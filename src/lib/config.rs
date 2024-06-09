use std::path::{Path, PathBuf};

use directories::UserDirs;
use log::trace;
use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeneralConfig {
  pub folders: Vec<PathBuf>,
}

impl Default for GeneralConfig {
  fn default() -> Self {
    #[cfg(target_os = "windows")]
    return Self {
      folders: vec![PathBuf::from("C:/")],
    };
    #[cfg(not(target_os = "windows"))]
    return Self {
      folders: vec![PathBuf::from("/")],
    };
  }
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Config {
  pub general: GeneralConfig,
}

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
    if let Some(end) = ret[start + 2..].find(markers.1) {
      let env_key = &ret[start + 2..end];
      if let Ok(env_val) = std::env::var(env_key) {
        ret = format!("{}{}{}", &ret[0..start], env_val, &ret[end + 1..]);
      } else {
        return Err(Error::Init(format!("{}: invalid configuration value, environment variable '{}' is undefined", path.as_ref().display(), env_key)));
      }
    }
  }
  Ok(ret.into())
}

impl Config {
  pub fn init(path: Option<&PathBuf>) -> crate::Result<Self> {
    let dflt_config = Config::default();
    let mut config = if let Some(path) = path {
      if !path.exists() {
        let mut f = std::fs::File::create_new(path).map_err(|e| {
          Error::IO(
            format!("failed to create config file '{}'", path.display()),
            Some(Box::new(e)),
          )
        })?;
        dflt_config
          .write(&mut f)
          .map_err(|e| e.with_context("failed to serialize default config".to_string()))?;
      }
      Config::parse(path)?
    } else {
      dflt_config
    };
    let mut new_folders = vec![];
    for folder in config.general.folders {
      new_folders.push(expand_path(&folder)?);
    }
    config.general.folders = new_folders;
    trace!("Config: {:#?}", config);
    Ok(config)
  }
  
  pub fn parse<P: AsRef<Path>>(path: P) -> crate::Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let ret = toml::from_str(&content)?;
    Ok(ret)
  }

  pub fn write<W: std::io::Write>(&self, mut w: W) -> crate::Result<()> {
    let mut data = toml::to_string_pretty(self)?;
    w.write(unsafe { data.as_bytes_mut() })?;
    Ok(())
  }

  pub fn read<R: std::io::Read>(&mut self, mut r: R) -> crate::Result<()> {
    let mut buf = String::new();
    r.read_to_string(&mut buf)?;
    *self = toml::from_str(&buf)?;
    Ok(())
  }
}
