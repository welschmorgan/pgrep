use std::{
  collections::HashMap,
  fmt::Display,
  path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use log::trace;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

/// Simple recursive folder scanning.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct FolderScan {
  path: PathBuf,
  files: Vec<PathBuf>,
  last_scanned: DateTime<Local>,
}

impl FolderScan {
  /// The common dirs to be excluded
  pub const DIR_EXCLUSIONS: [&'static str; 4] = [".git", "node_modules", "target", "vendor"];

  /// Create a new folder scanner
  ///
  /// # Examples
  ///
  /// ```
  /// use pgrep::project::FolderScan;
  /// use chrono::Local;
  /// use std::path::PathBuf;
  ///
  /// let now = Local::now();
  /// let scan = FolderScan::new(".").unwrap();
  /// for file in scan.files() {
  ///   println!("{}", file.display());
  /// }
  /// ```
  pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
    let files = Self::scan_folder(path.as_ref())?;
    Ok(Self {
      path: path.as_ref().to_path_buf(),
      files,
      last_scanned: Local::now(),
    })
  }

  fn scan_folder<P: AsRef<Path>>(path: P) -> crate::Result<Vec<PathBuf>> {
    let dir = std::fs::read_dir(path.as_ref())?;
    let mut ret = vec![];
    trace!("scanning '{}'", path.as_ref().display());
    for e in dir {
      let e = e?;
      if e.file_type()?.is_dir() {
        if let Some(fname) = e.file_name().to_str() {
          if Self::DIR_EXCLUSIONS.contains(&fname) || fname.starts_with(".") {
            continue;
          }
        }
        ret.append(&mut Self::scan_folder(&e.path())?);
      } else {
        ret.push(e.path());
      }
    }
    Ok(ret)
  }

  /// Retrieve the scanned folder path
  pub fn path(&self) -> &PathBuf {
    &self.path
  }

  /// Retrieve the discovered files
  pub fn files(&self) -> &Vec<PathBuf> {
    &self.files
  }

  /// Retrieve the last time the folder was scanned
  pub fn last_scanned(&self) -> &DateTime<Local> {
    &self.last_scanned
  }
}

/// A known project kind
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter, Clone, Hash)]
#[serde(tag = "type")]
pub enum ProjectKind {
  Rust,
  Go,
  C,
  Node,
  Maven,
  Other,
  Custom {
    name: String,
    language_exts: Vec<String>,
    project_files: Vec<String>,
  },
}

impl ProjectKind {
  pub fn name(&self) -> String {
    match self {
      ProjectKind::Rust => "Rust".to_string(),
      ProjectKind::Go => "Go".to_string(),
      ProjectKind::C => "C".to_string(),
      ProjectKind::Node => "Node".to_string(),
      ProjectKind::Maven => "Maven".to_string(),
      ProjectKind::Other => "Other".to_string(),
      ProjectKind::Custom { name, .. } => name.clone(),
    }
  }

  /// Retrieve the known project files
  pub fn project_files(&self) -> Vec<String> {
    match self {
      Self::Rust => vec!["Cargo.toml", "Cargo.lock"],
      Self::Go => vec!["go.mod"],
      Self::C => vec!["Makefile", "CMakeLists.txt"],
      Self::Node => vec!["package.json", "package.lock"],
      Self::Maven => vec!["pom.xml"],
      Self::Other => vec!["README.md", "LICENSE.md", "CONTRIBUTING.md"],
      Self::Custom { project_files, .. } => project_files
        .iter()
        .map(|ext| ext.as_str())
        .collect::<Vec<_>>(),
    }
    .iter()
    .map(|f| f.to_string())
    .collect::<Vec<_>>()
  }

  /// Retrieve the common source code extensions
  pub fn language_extensions(&self) -> Vec<String> {
    match self {
      Self::Rust => vec!["rs"],
      Self::Go => vec!["go"],
      Self::C => vec!["c", "h", "cc", "cpp", "cxx", "hh", "hxx", "hpp"],
      Self::Node => vec!["js", "ts"],
      Self::Maven => vec!["java"],
      Self::Other => vec![],
      Self::Custom { language_exts, .. } => language_exts
        .iter()
        .map(|ext| ext.as_str())
        .collect::<Vec<_>>(),
    }
    .iter()
    .map(|s| s.to_string())
    .collect::<Vec<_>>()
  }
}

impl Display for ProjectKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name())
  }
}

/// Represent a discovered project
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
  path: PathBuf,
  kinds: Vec<ProjectKind>,
  source_files: Vec<PathBuf>,
  project_files: Vec<PathBuf>,
}

impl Project {
  /// Create a new [`Project`]
  pub fn new<P: AsRef<Path>>(
    path: P,
    kinds: Vec<ProjectKind>,
    source_files: Vec<PathBuf>,
    project_files: Vec<PathBuf>,
  ) -> Self {
    Self {
      path: path.as_ref().to_path_buf(),
      kinds,
      source_files,
      project_files,
    }
  }

  /// Retrieve the project name from it's path
  pub fn name(&self) -> Option<String> {
    self
      .path
      .file_name()
      .unwrap()
      .to_str()
      .map(|s| s.to_string())
  }

  /// Retrieve the project path (folder)
  pub fn path(&self) -> &PathBuf {
    &self.path
  }

  /// Retrieve the project path (folder) as a mutable reference
  pub fn path_mut(&mut self) -> &mut PathBuf {
    &mut self.path
  }

  /// Retrieve the project kinds that were discovered using [`ProjectKind::project_files`]
  pub fn kinds(&self) -> &Vec<ProjectKind> {
    &self.kinds
  }
  /// Retrieve the project kinds that were discovered using [`ProjectKind::project_files`]
  /// as a mutable reference
  pub fn kinds_mut(&mut self) -> &mut Vec<ProjectKind> {
    &mut self.kinds
  }

  /// Retrieve the source code files that were discovered using [`ProjectKind::language_extensions`]
  pub fn source_files(&self) -> &Vec<PathBuf> {
    &self.source_files
  }
  /// Retrieve the source code files that were discovered using [`ProjectKind::language_extensions`]
  /// as a mutable reference
  pub fn source_files_mut(&mut self) -> &mut Vec<PathBuf> {
    &mut self.source_files
  }

  /// Retrieve the project files files that were discovered using [`ProjectKind::project_files`]
  pub fn project_files(&self) -> &Vec<PathBuf> {
    &self.project_files
  }
  /// Retrieve the project files files that were discovered using [`ProjectKind::project_files`]
  /// as a mutable reference
  pub fn project_files_mut(&mut self) -> &mut Vec<PathBuf> {
    &mut self.project_files
  }
}

/// Detect all the discovered [`Project`] roots from a given folder scan
pub fn detect_projects(scan: &FolderScan, mut custom_kinds: Vec<ProjectKind>) -> Vec<Project> {
  let mut ret = vec![];
  let mut project_roots: HashMap<PathBuf, Vec<ProjectKind>> = HashMap::new();
  let mut project_files: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
  let mut project_source_files: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
  fn find_project_root<'a, V>(
    path: &'a PathBuf,
    roots: &'a HashMap<PathBuf, V>,
  ) -> Option<&'a PathBuf> {
    let path = format!("{}", path.display());
    for (root, _) in roots {
      if path.len() >= root.as_os_str().len() {
        let path = &path[0..root.as_os_str().len()];
        if root.as_os_str().eq_ignore_ascii_case(path) {
          return Some(root);
        }
      }
    }
    None
  }
  for file in &scan.files {
    if let Some(_) = find_project_root(file, &project_roots) {
      project_source_files
        .entry(file.clone())
        .or_insert_with(|| vec![])
        .push(file.clone());
    } else {
      let mut kinds = ProjectKind::iter().collect::<Vec<_>>();
      kinds.append(&mut custom_kinds);
      for kind in &kinds {
        for project_file in kind.project_files() {
          if let Some(fname) = file.file_name() {
            if fname.eq_ignore_ascii_case(project_file) {
              let project_dir = file.parent().unwrap().to_path_buf();
              project_roots
                .entry(project_dir.clone())
                .or_insert_with(|| vec![])
                .push(kind.clone());
              project_files
                .entry(project_dir.clone())
                .or_insert_with(|| vec![])
                .push(file.clone());
              project_source_files
                .entry(project_dir.clone())
                .or_insert_with(|| vec![]);
            }
          }
        }
      }
    }
  }
  for (path, kinds) in project_roots {
    let source_files = project_source_files.remove(&path).unwrap();
    let project_files = project_files.remove(&path).unwrap();
    ret.push(Project::new(&path, kinds, source_files, project_files));
  }
  ret
}
