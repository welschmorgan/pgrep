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
use directories::ProjectDirs;
use log::{debug, error, warn};

/// The qualifier for windows and macOS config folders
pub const APP_QUALIFIER: &'static str = "com";
/// The organization for windows and macOS config folders
pub const APP_ORGANIZATION: &'static str = "darksofts";
/// The application for windows and macOS config folders
pub const APP_APPLICATION: &'static str = env!("CARGO_PKG_NAME");

/// Retrieve the platform-dependent project directories.
pub fn get_project_dirs() -> Option<ProjectDirs> {
  ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_APPLICATION)
}

/// The application structure
pub struct App {
  /// The command-line options
  options: AppOptions,
  /// The loaded configuration
  config: Config,
  /// The cache store
  cache: Arc<Mutex<Cache>>,
  /// The parsed query
  query: Query,
  /// The project writer to use
  writer: BoxedProjectWriter,
}

impl App {
  /// Create a new application instance.
  /// This will:
  ///   - configure the logger
  ///   - parse the command-line options
  ///   - load the user configuration
  ///   - parse the query string
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

  /// Run the application, scanning the code folders and filtering projects.
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

  /// Scan code folders and extract project roots
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

  /// Filter discovered project using the command-line query
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

  /// Write the report to the configured writer
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
