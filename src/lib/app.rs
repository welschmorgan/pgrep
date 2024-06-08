use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  str::FromStr,
  sync::{Arc, Mutex},
};

use clap::{Parser, ValueHint};
use log::{debug, error, info, trace, warn};

use crate::{cache, detect_projects, Cache, Config, Error, FolderScan, ProjectKind, Query};

#[derive(Debug, Parser)]
#[command(about, long_about = None, version)]
pub struct AppOptions {
  #[arg(required_unless_present("clean_cache"), default_value("*"), next_line_help(true), help("The query used to find the project. It supports the following wildcards:\n\
\t- '?': an optional character\n\
\t- '_': a required character\n\
\t- '#': a required digit\n\
\t- '*': any string\n"))]
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
  query: Query,
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
    let query = options.query.parse::<Query>()?;
    Ok(Self {
      options,
      config,
      cache,
      query
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
    let mut projects = HashMap::new();
    for folder in &self.config.general.folders {
      let mut cache = self.cache.lock().unwrap();
      let scan = cache.load_store(folder, || FolderScan::new(folder))?;
      projects.insert(
        folder.clone(),
        cache.load_store(&folder.join(".projects"), || Ok(detect_projects(&scan)))?,
      );
    }
    if projects.is_empty() {
      error!("failed to find '{}'", self.options.query);
    } else {
      // for (path, projects) in &matches {
      //   println!("{}:", path.display());
      //   for project in projects {
      //     println!("  - {:?} {}", project.kinds(), project.path().display());
      //   }
      // }
      let matches = projects
        .iter()
        .flat_map(|(_, projects)| projects)
        .filter(|project| {
          if let Some(name) = project.name() {
            if self.query.matches(&name) {
              return true;
            }
          }
          project.path().components().find(|part| {
            if let Some(part_str) = part.as_os_str().to_str() {
              return self.query.matches(part_str);
            }
            return false;
          }).is_some()
        })
        .collect::<Vec<_>>();
      for proj in matches {
        println!(
          "{:?} {} - {}",
          proj.kinds(),
          proj.name().unwrap(),
          proj.path().display()
        )
      }
    }
    self.cache.lock().unwrap().shutdown()?;
    Ok(())
  }
}
