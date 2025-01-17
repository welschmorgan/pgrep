use std::{
  collections::HashMap,
  io::stdout,
  path::PathBuf,
  sync::{Arc, Mutex},
};

use crate::{
  cache, detect_projects, AppOptions, BoxedProjectMatchesFormatter, BoxedUI, Cache, Config, Error,
  FolderScan, Project, Query,
};
use clap::Parser;
use directories::ProjectDirs;
use log::{debug, warn};

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
  /// The project formatter to use
  formatter: BoxedProjectMatchesFormatter,
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
    let config = Config::load(options.config.as_ref(), options.folders.clone())?;
    if config.general.folders.is_empty() {
      return Err(Error::Init(format!(
        "No source code folders configured. use -F/--folder to specify one or more."
      )));
    }
    let cache = cache().clone();
    if options.no_cache {
      cache.lock().unwrap().disable();
    }
    let query = options.query.clone();
    Ok(Self {
      formatter: options.format.formatter()?,
      options,
      config,
      cache,
      query,
    })
  }

  /// Run the application, scanning the code folders and filtering projects.
  pub fn run(self) -> crate::Result<()> {
    if self.options.list && self.options.query != Default::default() {
      return Err(Error::Init(format!(
        "Query given with --list but the two options are mutually exclusive!"
      )));
    } else if self.options.dump_config {
      println!("{:#?}", self.config);
      return Ok(());
    } else if self.options.clean_cache {
      let path = self.cache.lock().unwrap().clean()?;
      warn!("removed '{}'", path.display());
      return Ok(());
    }
    if !self.options.list {
      debug!(
        "Looking for '{}' in the following paths: {:?}",
        self.options.query, self.config.general.folders
      );
    }
    // get list of projects
    let projects = self.list_projects()?;
    if projects.is_empty() {
      return Err(Error::Unknown(format!(
        "no project root discovered for {} dirs:\n{:#?}",
        self.config.general.folders.len(),
        self.config.general.folders
      )));
    } else {
      // match discovered projects with user query
      let projects = projects
        .iter()
        .flat_map(|(_, projects)| projects)
        .collect::<Vec<_>>();
      debug!("found {} projects", projects.len());
      let matches = match self.options.list {
        false => {
          let matches = Self::match_projects(&self.query, &projects);
          if matches.is_empty() {
            return Err(Error::Unknown(format!(
              "no match found for query '{}' in {} projects",
              self.query,
              projects.len()
            )));
          }
          matches
        }
        true => projects,
      }
      .iter()
      .map(|proj| (*proj).clone())
      .collect::<Vec<_>>();

      #[cfg(feature = "tui")]
      let has_tui = self.options.tui;
      #[cfg(not(feature = "tui"))]
      let has_tui = false;
      let mut ui: BoxedUI = match has_tui {
        true => {
          #[cfg(not(feature = "tui"))]
          panic!("Feature 'tui' not available");
          #[cfg(feature = "tui")]
          {
            use crate::Terminal;
            Box::new(Terminal::new(self.options.editor)?)
          }
        }
        false => {
          #[cfg(not(feature = "console"))]
          panic!("Feature 'console' not available");
          #[cfg(feature = "console")]
          {
            use crate::Console;
            Box::new(Console::new())
          }
        }
      };
      ui.write_matches(&matches, &self.formatter)?;
      ui.render_loop()?;
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
        cache.load_store(&folder.join(".projects"), || {
          Ok(detect_projects(
            &scan,
            self.config.general.project_kinds.clone(),
          ))
        })?,
      );
    }
    Ok(projects)
  }

  /// Filter discovered project using the command-line query
  pub fn match_projects<'a>(query: &'a Query, projects: &'a Vec<&'a Project>) -> Vec<&'a Project> {
    projects
      .iter()
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
      .map(|proj| *proj)
      .collect::<Vec<_>>()
  }

  /// Write the report to the configured writer
  pub fn write_report(&self, matches: &Vec<Project>) -> crate::Result<()> {
    self.formatter.write(&mut stdout(), matches)?;
    Ok(())
  }
}
