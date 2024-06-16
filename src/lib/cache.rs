use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use chrono::{DateTime, Duration, Local};
use lazy_static::lazy_static;
use log::debug;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use crate::{get_project_dirs, Error};

/// An index for cache files which allows storing last write times and paths.
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Index {
  paths: Vec<PathBuf>,
  write_times: HashMap<PathBuf, DateTime<Local>>,
  written_at: Option<DateTime<Local>>,
}

/// The cache store holding the caching state of the whole app.
/// 
/// It will write the index on shutdown to persist state.
pub struct Cache {
  base_dir: PathBuf,
  index: Index,
  enabled: bool,
}

impl Cache {
  /// The threshold after which to bust the cache, effectively rescanning project roots
  pub const CACHE_BUST_THRESHOLD: Duration = Duration::minutes(5);
  /// The stored files extension
  pub const CACHE_EXT: &'static str = ".bin";
  /// The key under which to find the index
  pub const CACHE_INDEX_KEY: &'static str = "index";

  /// Create a new cache store
  fn new() -> crate::Result<Self> {
    let cache_dir = match get_project_dirs() {
      Some(proj_dir) => proj_dir.cache_dir().to_path_buf(),
      None => PathBuf::from(".cache"),
    };
    if !cache_dir.exists() {
      std::fs::create_dir_all(&cache_dir)?;
    }
    let mut ret = Self {
      index: Index::default(),
      base_dir: cache_dir,
      enabled: true,
    };
    let index_path = ret.path(Self::CACHE_INDEX_KEY);
    if index_path.exists() {
      if let Err(e) = ret.load_index() {
        debug!("{}", e);
      }
    }
    for (key, write_time) in &ret.index.write_times {
      let expires_at: DateTime<Local> = *write_time + Self::CACHE_BUST_THRESHOLD;
      let now = Local::now();
      let is_expired = now > expires_at;
      debug!(
        "{} created at {}{}",
        key.display(),
        write_time.format("%Y-%m-%d %H:%M:%S"),
        match is_expired {
          true => format!(
            " \x1b[0;31mexpired at\x1b[0m {} ({} minutes ago)",
            expires_at.format("%Y-%m-%d %H:%M:%S"),
            (now - expires_at).num_minutes()
          ),
          false => format!(
            " \x1b[0;32mwill expire at\x1b[0m {}",
            expires_at.format("%Y-%m-%d %H:%M:%S")
          ),
        }
      );
    }
    Ok(ret)
  }

  /// If disabled, caching will never occur
  pub fn set_enabled(&mut self, state: bool) {
    self.enabled = state
  }

  /// Enable caching
  pub fn enable(&mut self) {
    self.set_enabled(true)
  }
  
  /// Disable caching
  pub fn disable(&mut self) {
    self.set_enabled(false)
  }

  /// Save the index
  pub fn save_index(&mut self) -> crate::Result<()> {
    if !self.enabled {
      return Ok(());
    }
    let mut buf = vec![];
    self.index.written_at = Some(Local::now());
    self
      .index
      .serialize(&mut Serializer::new(&mut buf))
      .map_err(|e| Error::IO(format!("failed to serialize index"), Some(Box::new(e))))?;
    let path = self.path(Self::CACHE_INDEX_KEY);
    std::fs::write(&path, buf).map_err(|e| {
      Error::IO(
        format!("failed to save index to '{}'", path.display()),
        Some(Box::new(e)),
      )
    })?;
    debug!(
      "Saved '{}': {} entries",
      path.display(),
      self.index.paths.len()
    );
    Ok(())
  }

  /// Load the index
  pub fn load_index(&mut self) -> crate::Result<()> {
    if !self.enabled {
      return Ok(());
    }
    let path = self.path(Self::CACHE_INDEX_KEY);
    let buf = std::fs::read(&path).map_err(|e| {
      Error::IO(
        format!("failed to load index from '{}'", path.display()),
        Some(Box::new(e)),
      )
    })?;
    let mut de = Deserializer::new(buf.as_slice());
    self.index = Deserialize::deserialize(&mut de)?;
    debug!(
      "Loaded '{}': {} entries",
      path.display(),
      self.index.paths.len()
    );
    Ok(())
  }

  /// Shutdown the cache store, saving its index
  pub fn shutdown(&mut self) -> crate::Result<()> {
    self.save_index()?;
    Ok(())
  }

  /// Will delete everything in the cache folder!
  pub fn clean(&self) -> crate::Result<PathBuf> {
    std::fs::remove_dir_all(&self.base_dir)?;
    Ok(self.base_dir.clone())
  }

  /// Retrieve the on-disk path for a given key.
  /// This will replace non-alnum characters with '_'
  /// 
  /// # Examples
  /// 
  /// ```
  /// use std::path::PathBuf;
  /// use pgrep::cache;
  /// 
  /// assert_eq!(cache().lock().unwrap().path("/home/user/myfile.txt").file_name().unwrap(), PathBuf::from("_home_user_myfile_txt.bin"));
  /// assert_eq!(cache().lock().unwrap().path("myfile.txt").file_name().unwrap(), PathBuf::from("myfile_txt.bin"));
  /// ```
  pub fn path<K: AsRef<Path>>(&self, key: K) -> PathBuf {
    let sub = format!("{}", key.as_ref().display())
      .replace("\\", "/")
      .chars()
      .map(|ch| {
        if ch.is_ascii_alphanumeric() {
          return ch;
        }
        '_'
      })
      .collect::<String>()
      + Self::CACHE_EXT;
    self.base_dir.join(&sub)
  }

  /// Load a cached entity from the store
  /// 
  /// # Examples
  /// 
  /// ```
  /// use std::path::PathBuf;
  /// use pgrep::{cache, Project, Result};
  /// 
  /// let res: Result<Option<Project>> = cache().lock().unwrap().load("C:/dev/project/my_project");
  /// ```
  pub fn load<'a, K: AsRef<Path>, E: Deserialize<'a>>(&self, key: K) -> crate::Result<Option<E>> {
    if !self.enabled {
      return Ok(None);
    }
    debug!("loading '{}' from cache", key.as_ref().display());
    let write_time = match self.index.write_times.get(key.as_ref()) {
      Some(write_time) => write_time,
      None => {
        debug!("cache entry '{}' not in index", key.as_ref().display());
        return Ok(None);
      }
    }
    .clone();
    let expires_at = write_time + Self::CACHE_BUST_THRESHOLD;
    if Local::now() >= expires_at {
      debug!("cache is stale for '{}'", key.as_ref().display());
      return Ok(None);
    }
    let path = self.path(&key);
    if !path.exists() {
      return Ok(None);
    }
    let content = std::fs::read(path).map_err(|e| {
      Error::IO(
        format!("cannot load '{}' from cache", key.as_ref().display()),
        Some(Box::new(e)),
      )
    })?;
    let mut de = Deserializer::new(content.as_slice());
    let ret: E = Deserialize::deserialize(&mut de).map_err(|e| {
      Error::IO(
        format!("cannot deserialize '{}' from cache", key.as_ref().display()),
        Some(Box::new(e)),
      )
    })?;
    Ok(Some(ret))
  }

  /// Save an entity to the cache store
  /// 
  /// # Examples
  /// 
  /// ```
  /// use std::path::PathBuf;
  /// use pgrep::{cache, Project, Result, ProjectKind};
  /// 
  /// let project = Project::new(
  ///   "C:/dev/project/my_project", 
  ///   vec![ProjectKind::Rust], 
  ///   vec![PathBuf::from("C:/dev/project/my_project/src/main.rs")],
  ///   vec![PathBuf::from("C:/dev/project/my_project/Cargo.toml")]
  /// );
  /// let res: Result<PathBuf> = cache().lock().unwrap().store(project.path(), &project);
  /// ```
  pub fn store<K: AsRef<Path>, E: Serialize>(
    &mut self,
    key: &K,
    value: &E,
  ) -> crate::Result<PathBuf> {
    let path = self.path(key);
    if !self.enabled {
      return Ok(path);
    }
    debug!("saving '{}' to cache", key.as_ref().display());
    let mut buf = vec![];
    value
      .serialize(&mut Serializer::new(&mut buf))
      .map_err(|e| {
        Error::IO(
          format!("cannot serialize '{}'", key.as_ref().display()),
          Some(Box::new(e)),
        )
      })?;
    std::fs::write(&path, buf).map_err(|e| {
      Error::IO(
        format!("cannot save '{}' to cache", key.as_ref().display()),
        Some(Box::new(e)),
      )
    })?;
    let key_path = key.as_ref().to_path_buf();
    if !self.index.paths.contains(&key_path) {
      self.index.paths.push(key_path.clone());
    }
    self.index.write_times.insert(key_path, Local::now());
    Ok(path)
  }

  /// Load the entity from cache if it was found in the store and the [`Self::CACHE_BUST_THRESHOLD`]
  /// has not been reached yet.
  /// 
  /// Otherwise store the entity provided by the `action` parameter.
  /// 
  /// # Examples
  /// 
  /// ```
  /// use std::path::PathBuf;
  /// use pgrep::{cache, Project, Result, ProjectKind};
  /// 
  /// let path = PathBuf::from("C:/dev/project/my_project");
  /// let project: Result<Project> = cache().lock().unwrap().load_store(&path, || Ok(Project::new(
  ///   path.clone(), 
  ///   vec![ProjectKind::Rust], 
  ///   vec![PathBuf::from("C:/dev/project/my_project/src/main.rs")],
  ///   vec![PathBuf::from("C:/dev/project/my_project/Cargo.toml")]
  /// )));
  /// ```
  pub fn load_store<
    'a,
    K: AsRef<Path>,
    E: Deserialize<'a> + Serialize,
    F: Fn() -> crate::Result<E>,
  >(
    &mut self,
    key: &K,
    action: F,
  ) -> crate::Result<E> {
    if let Some(entity) = self.load::<_, E>(key.as_ref())? {
      return Ok(entity);
    }
    let entity = action()?;
    self.store(key, &entity)?;
    Ok(entity)
  }
}

lazy_static! {
  /// The global cache instance as a mutexed [`std::sync::Arc`]
  static ref _INST: Arc<Mutex<Cache>> =
    Arc::new(Mutex::new(Cache::new().expect("failed to create cache")));
}

/// Retrieve the global cache instance
pub fn cache() -> &'static Arc<Mutex<Cache>> {
  &_INST
}
