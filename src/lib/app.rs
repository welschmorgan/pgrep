use std::{
  collections::HashMap,
  io::stdout,
  path::PathBuf,
  sync::{Arc, Mutex},
};

use crate::{
  cache, default_project_writer, detect_projects, AppOptions, BoxedProjectWriter, Cache, Config, FolderScan, Project, Query
};
use clap::Parser;
use log::{debug, error, warn};

pub struct App {
  options: AppOptions,
  config: Config,
  cache: Arc<Mutex<Cache>>,
  query: Query,
  writer: BoxedProjectWriter,
}

impl App {
  pub fn new() -> crate::Result<Self> {
    pretty_env_logger::try_init()?;
    let options = AppOptions::parse();
    let config = Config::init(options.config.as_ref())?;
    let cache = cache().clone();
    if options.no_cache {
      cache.lock().unwrap().disable();
    }
    let query = options.query.clone();
    Ok(Self {
      writer: default_project_writer(),
      options,
      config,
      cache,
      query,
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
    let projects = self.list_projects()?;
    if projects.is_empty() {
      error!("failed to find '{}'", self.options.query);
    } else {
      let matches = Self::match_projects(&self.query, &projects);
      self.write_report(&matches)?;
    }
    self.cache.lock().unwrap().shutdown()?;
    Ok(())
  }

  pub fn list_projects(&self) -> crate::Result<HashMap<PathBuf, Vec<Project>>> {
    let mut projects = HashMap::new();
    for folder in &self.config.general.folders {
      let mut cache = self.cache.lock().unwrap();
      let scan = cache.load_store(folder, || FolderScan::new(folder))?;
      projects.insert(
        folder.clone(),
        cache.load_store(&folder.join(".projects"), || Ok(detect_projects(&scan)))?,
      );
    }
    Ok(projects)
  }

  pub fn match_projects<'a>(
    query: &'a Query,
    projects: &'a HashMap<PathBuf, Vec<Project>>,
  ) -> Vec<&'a Project> {
    projects
      .iter()
      .flat_map(|(_, projects)| projects)
      .filter(|project| {
        if let Some(name) = project.name() {
          if query.matches(&name) {
            return true;
          }
        }
        project
          .path()
          .components()
          .find(|part| {
            if let Some(part_str) = part.as_os_str().to_str() {
              return query.matches(part_str);
            }
            return false;
          })
          .is_some()
      })
      .collect::<Vec<_>>()
  }

  pub fn write_report<'a>(&'a self, matches: &'a Vec<&'a Project>) -> crate::Result<()> {
    use std::io::Write;
    let stdout = &mut stdout();
    for proj in matches {
      self.writer.write(stdout, proj)?;
      write!(stdout, "\n")?;
    }
    Ok(())
  }
}
