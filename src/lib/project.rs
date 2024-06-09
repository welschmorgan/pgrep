use std::{
  collections::HashMap,
  path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use log::{debug, trace};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct FolderScan {
  path: PathBuf,
  files: Vec<PathBuf>,
  last_scanned: DateTime<Local>,
}

impl FolderScan {
  pub const DIR_EXCLUSIONS: [&'static str; 4] = [".git", "node_modules", "target", "vendor"];

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

  pub fn path(&self) -> &PathBuf {
    &self.path
  }

  pub fn files(&self) -> &Vec<PathBuf> {
    &self.files
  }

  pub fn last_scanned(&self) -> &DateTime<Local> {
    &self.last_scanned
  }
}

#[derive(
  Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter, Clone, Copy, Hash,
)]
pub enum ProjectKind {
  Rust,
  Node,
  Maven,
  Other,
}

impl ProjectKind {
  pub fn project_files(&self) -> Vec<&str> {
    match self {
      Self::Rust => vec!["Cargo.toml", "Cargo.lock"],
      Self::Node => vec!["package.json", "package.lock"],
      Self::Maven => vec!["pom.xml"],
      Self::Other => vec!["README.md", "LICENSE.md", "CONTRIBUTING.md"],
    }
  }

  pub fn language_extensions(&self) -> Vec<&str> {
    match self {
      Self::Rust => vec!["rs"],
      Self::Node => vec!["js", "ts"],
      Self::Maven => vec!["java"],
      Self::Other => vec![],
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
  path: PathBuf,
  kinds: Vec<ProjectKind>,
  source_files: Vec<PathBuf>,
  project_files: Vec<PathBuf>,
}

impl Project {
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

  pub fn name(&self) -> Option<String> {
    self
      .path
      .file_name()
      .unwrap()
      .to_str()
      .map(|s| s.to_string())
  }

  pub fn path(&self) -> &PathBuf {
    &self.path
  }
  pub fn path_mut(&mut self) -> &mut PathBuf {
    &mut self.path
  }

  pub fn kinds(&self) -> &Vec<ProjectKind> {
    &self.kinds
  }
  pub fn kinds_mut(&mut self) -> &mut Vec<ProjectKind> {
    &mut self.kinds
  }

  pub fn source_files(&self) -> &Vec<PathBuf> {
    &self.source_files
  }
  pub fn source_files_mut(&mut self) -> &mut Vec<PathBuf> {
    &mut self.source_files
  }

  pub fn project_files(&self) -> &Vec<PathBuf> {
    &self.project_files
  }
  pub fn project_files_mut(&mut self) -> &mut Vec<PathBuf> {
    &mut self.project_files
  }
}

pub trait ProjectWriter {
  fn write(&self, to: &mut dyn std::io::Write, project: &Project) -> std::io::Result<()>;
}

pub type BoxedProjectWriter = Box<dyn ProjectWriter>;

pub struct TextProjectWriter {}

impl ProjectWriter for TextProjectWriter {
  fn write(&self, to: &mut dyn std::io::Write, project: &Project) -> std::io::Result<()> {
    write!(
      to,
      "{:?} {} - {}",
      project.kinds(),
      project.name().unwrap(),
      project.path().display()
    )
  }
}

pub fn default_project_writer() -> BoxedProjectWriter {
  Box::new(TextProjectWriter{})
}

pub fn detect_projects(scan: &FolderScan) -> Vec<Project> {
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
      for kind in ProjectKind::iter() {
        for project_file in kind.project_files() {
          if let Some(fname) = file.file_name() {
            if fname.eq_ignore_ascii_case(project_file) {
              let project_dir = file.parent().unwrap().to_path_buf();
              project_roots
                .entry(project_dir.clone())
                .or_insert_with(|| vec![])
                .push(kind);
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
