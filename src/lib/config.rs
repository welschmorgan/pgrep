use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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

impl Config {
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
