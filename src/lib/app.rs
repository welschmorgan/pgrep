use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
};

use clap::Parser;
use log::{debug, error, trace, warn};

use crate::{cache, Cache, Config, FolderScan, ProjectType};

#[derive(Debug, Parser)]
#[command(about, long_about = None, version)]
pub struct AppOptions {
  /// The query used to find the project.
  #[arg(required_unless_present("clean_cache"), default_value("*"))]
  query: String,

  /// Specify a custom config file to load.
  #[arg(short, long)]
  config: Option<PathBuf>,

  /// Clean the cache folder and exit.
  #[arg(long, exclusive(true))]
  clean_cache: bool,

  /// Disable cache usage.
  #[arg(long)]
  no_cache: bool,
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
    let cache = cache().clone();
    if options.no_cache {
      cache.lock().unwrap().disable();
    }
    Ok(Self {
      options,
      config,
      cache,
    })
  }

  pub fn run(self) -> crate::Result<()> {
    if self.options.clean_cache {
      let path = self.cache.lock().unwrap().clean()?;
      warn!("removed '{}'", path.display());
      return Ok(());
    }
    debug!(
      "Looking for '{}' in the following paths: {:?}",
      self.options.query, self.config.general.folders
    );
    let mut matches = HashMap::new();
    for folder in &self.config.general.folders {
      let scan = self
        .cache
        .lock()
        .unwrap()
        .load_store(folder, || FolderScan::new(folder))?;
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
    self.cache.lock().unwrap().shutdown()?;
    Ok(())
  }
}
