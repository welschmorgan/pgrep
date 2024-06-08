use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use clap::Parser;
use log::{debug, error, trace};

use crate::{cache, Cache, Config, FolderScan, ProjectType};

#[derive(Debug, Parser)]
#[command(about, long_about = None, version)]
pub struct AppOptions {
  /// The query used to find the project.
  query: String,

  /// Specify a custom config file to load.
  #[arg(short, long)]
  config: Option<PathBuf>,
}

pub struct App {
  options: AppOptions,
  config: Config,
  cache: Arc<Mutex<Cache>>,
}

impl App {
  pub fn new() -> crate::Result<Self> {
    pretty_env_logger::try_init()?;
    let options = AppOptions::parse();
    let dflt_config = Config::default();
    let config = if let Some(path) = &options.config {
      if !path.exists() {
        let mut f = std::fs::File::create_new(path)?;
        dflt_config.write(&mut f)?;
      }
      Config::parse(path)?
    } else {
      dflt_config
    };
    trace!("Config: {:#?}", config);
    Ok(Self {
      options,
      config,
      cache: cache().clone(),
    })
  }

  pub fn run(self) -> crate::Result<()> {
    debug!(
      "Looking for '{}' in the following paths: {:?}",
      self.options.query, self.config.general.folders
    );
    let mut matches = HashMap::new();
    for folder in &self.config.general.folders {
      let scan = FolderScan::new(folder)?;
      for typ in ProjectType::detect(&scan)? {
        matches
          .entry(typ.path.clone())
          .or_insert_with(|| vec![])
          .push(typ);
      }
    }
    if matches.is_empty() {
      error!("failed to find '{}'", self.options.query);
    }
    Ok(())
  }
}
