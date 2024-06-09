use std::path::{Path, PathBuf};

use directories::{ProjectDirs, UserDirs};
use log::{debug, trace};
use serde::{Deserialize, Serialize};

use crate::{get_project_dirs, Error};

/// Expand a path containing symbolic dirs into an absolute one.
///
/// # Examples
///
/// ```
/// use pgrep::config::expand_path;
/// use std::path::PathBuf;
///
/// let user = |path: &str| PathBuf::from(path.replace("<user>", &whoami::username()));
/// let expanded = expand_path("~/my_folder").unwrap();
///
/// #[cfg(target_os = "windows")]
/// assert_eq!(expanded, user("C:/Users/<user>/my_folder"));
///
/// #[cfg(target_os = "linux")]
/// assert_eq!(expanded, user("/home/<user>/my_folder"));
///
/// #[cfg(target_os = "macos")]
/// assert_eq!(expanded, user("/Users/<user>/my_folder"));
/// ```
///
/// ```
/// use pgrep::config::expand_path;
/// use std::path::PathBuf;
///
/// std::env::set_var("MY_VAR", "a_value");
///
/// let expanded = expand_path("/var/${MY_VAR}/my_folder").unwrap();
/// assert_eq!(expanded, PathBuf::from("/var/a_value/my_folder"));
/// ```
///
pub fn expand_path<P: AsRef<Path>>(path: P) -> crate::Result<PathBuf> {
  let home = if let Some(user_dirs) = UserDirs::new() {
    user_dirs.home_dir().to_path_buf()
  } else if let Ok(val) = std::env::var("HOME") {
    PathBuf::from(val)
  } else {
    PathBuf::from(".")
  };
  let mut ret = format!("{}", path.as_ref().display());
  if ret.starts_with("~") {
    let mut sub = ret[1..].trim();
    if sub.starts_with("/") {
      sub = sub[1..].trim();
    }
    ret = format!("{}/{}", home.display(), sub);
  }
  let markers = ("${", "}");
  while let Some(start) = ret.find(markers.0) {
    if let Some(mut end) = ret[start + 2..].find(markers.1) {
      end += start + 2;
      let env_key = &ret[start + 2..end];
      if let Ok(env_val) = std::env::var(env_key) {
        ret = format!("{}{}{}", &ret[0..start], env_val, &ret[end + 1..]);
      } else {
        return Err(Error::Init(format!(
          "{}: invalid configuration value, environment variable '{}' is undefined",
          path.as_ref().display(),
          env_key
        )));
      }
    }
  }
  Ok(ret.into())
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneralConfig {
  pub folders: Vec<PathBuf>,
}

impl Default for GeneralConfig {
  fn default() -> Self {
    return Self {
      folders: vec![],
    };
  }
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Config {
  pub general: GeneralConfig,
}

impl Config {
  /// The default configuration file name to search in [`common_directories`]
  ///
  /// [`common_directories`]: Config::common_config_dirs
  pub const DEFAULT_CONFIG_NAME: &'static str = "pgrep.toml";

  /// Retrieve the list of common config directories.
  /// This is used to sequentially check for a config file in each folder.
  ///
  /// On linux, this will give:
  /// ```json
  /// [
  ///   "~/.config/pgrep",
  ///   "~/.local/share/pgrep",
  ///   "."
  /// ]
  /// ```
  pub fn common_config_dirs() -> Vec<PathBuf> {
    let mut ret = vec![];
    if let Some(proj_dirs) = get_project_dirs() {
      ret.push(proj_dirs.preference_dir().to_path_buf());
      ret.push(proj_dirs.config_dir().to_path_buf());
      ret.push(proj_dirs.config_local_dir().to_path_buf());
      ret.push(proj_dirs.data_dir().to_path_buf());
      ret.push(proj_dirs.data_local_dir().to_path_buf());
    }
    ret.push(PathBuf::from("."));
    ret.dedup();
    ret
  }

  /// Retrieve the final configuration path.
  /// It will search through [`common directories`] if `path` is not given.
  pub fn path(path: Option<&PathBuf>) -> PathBuf {
    let common_dirs = Self::common_config_dirs();
    path.cloned()
      .or_else(|| {
        common_dirs
          .iter()
          .map(|config_dir| config_dir.join(Self::DEFAULT_CONFIG_NAME))
          .find(|config_file| config_file.exists())
      })
      .or_else(|| Some(common_dirs[0].join(Self::DEFAULT_CONFIG_NAME)))
      .unwrap()
  }

  /// Load the configuration values.
  ///
  /// If no config file path is specified, it will search the list of [`common directories`] for the [`Config::DEFAULT_CONFIG_NAME`] file
  /// and if found load it.
  ///
  /// If a config file path is specified, it doesn't even try to find the common config dir.
  ///
  /// If the config file doesn't exist, it write the default config to it.
  ///
  /// [`common directories`]: Config::common_config_dirs()
  pub fn load(user_path: Option<&PathBuf>, mut folders: Vec<PathBuf>) -> crate::Result<Self> {
    let dflt_config = Config::default();
    
    let path = Self::path(user_path);
    if !path.exists() {
      debug!("Creating default configuration at '{}'", path.display());
      // Create the config dir and write the default config file
      dflt_config
        .save(Some(&path))
        .map_err(|e| e.with_context("failed to serialize default config".to_string()))?;
    }
    debug!("Loading user configuration from '{}'", path.display());
    
    let mut config = Config::parse(&path)?;
    let len_before = config.general.folders.len();
    config.general.folders.append(&mut folders);
    config.general.folders.sort();
    config.general.folders.dedup();
    if config.general.folders.len() != len_before {
      config.save(Some(&path))?;
    }

    // expand folders
    let mut new_folders = vec![];
    for folder in config.general.folders {
      new_folders.push(expand_path(&folder)?);
    }
    config.general.folders = new_folders;
    trace!("Config: {:#?}", config);
    Ok(config)
  }

  /// Save the current configuration to disk.
  /// If path is supplied it will use this instead of guessing the correct path.
  pub fn save(&self, path: Option<&PathBuf>) -> crate::Result<()> {
    let path = Self::path(path);
    let mut f = std::fs::File::create(&path).map_err(|e| {
      Error::IO(
        format!("failed to create config file '{}'", path.display()),
        Some(Box::new(e)),
      )
    })?;
    if let Some(parent) = path.parent() {
      if !parent.exists() {
        std::fs::create_dir_all(parent)?;
      }
    }
    self.write(&mut f)
  }

  /// Parse the configuration from a file path
  pub fn parse<P: AsRef<Path>>(path: P) -> crate::Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let ret = toml::from_str(&content)?;
    Ok(ret)
  }

  /// Write the configuration to a [`std::io::Write`]
  pub fn write<W: std::io::Write>(&self, mut w: W) -> crate::Result<()> {
    let mut data = toml::to_string_pretty(self)?;
    w.write(unsafe { data.as_bytes_mut() })?;
    Ok(())
  }

  /// Read the configuration from a [`std::io::Write`]
  pub fn read<R: std::io::Read>(&mut self, mut r: R) -> crate::Result<()> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    *self = toml::from_str(&buf)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use crate::Config;

  #[test]
  fn common_dirs() {
    println!("{:#?}", Config::common_config_dirs());
  }
}
