use std::{
  collections::HashMap,
  io::ErrorKind,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, TimeDelta, TimeZone, Utc};
use directories::ProjectDirs;
use lazy_static::lazy_static;
use log::{debug, error};
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Index {
  paths: Vec<PathBuf>,
  write_times: HashMap<PathBuf, DateTime<Local>>,
  written_at: Option<DateTime<Local>>,
}

pub struct Cache {
  base_dir: PathBuf,
  index: Index,
  enabled: bool,
}

impl Cache {
  pub const CACHE_BUST_THRESHOLD: Duration = Duration::minutes(5);
  pub const CACHE_EXT: &'static str = ".bin";
  pub const CACHE_INDEX_KEY: &'static str = "index";

  fn new() -> crate::Result<Self> {
    let cache_dir = match ProjectDirs::from("com", "darksofts", "rust project grep") {
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

  pub fn set_enabled(&mut self, state: bool) {
    self.enabled = state
  }

  pub fn enable(&mut self) {
    self.set_enabled(true)
  }

  pub fn disable(&mut self) {
    self.set_enabled(false)
  }

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

  pub fn shutdown(&mut self) -> crate::Result<()> {
    self.save_index()?;
    Ok(())
  }

  /// Will delete everything in the cache folder!
  pub fn clean(&self) -> crate::Result<PathBuf> {
    std::fs::remove_dir_all(&self.base_dir)?;
    Ok(self.base_dir.clone())
  }

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
  static ref _INST: Arc<Mutex<Cache>> =
    Arc::new(Mutex::new(Cache::new().expect("failed to create cache")));
}

pub fn cache() -> &'static Arc<Mutex<Cache>> {
  &_INST
}