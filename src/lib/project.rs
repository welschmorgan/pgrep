use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Local};
use log::trace;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Serialize, Deserialize, Clone)]
pub struct FolderScan {
  path: PathBuf,
  files: Vec<PathBuf>,
  last_scanned: DateTime<Local>,
}

impl FolderScan {
  pub const DIR_EXCLUSIONS: [&'static str; 3] = [".git", "node_modules", "target"];

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter, Clone, Copy)]
pub enum ProjectType {
  Rust,
  Node,
  Maven,
  Other,
}

pub struct ProjectTypeMatch {
  pub typ: ProjectType,
  pub path: PathBuf,
  pub files: Vec<PathBuf>,
}

impl ProjectType {
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
      Self::Rust => vec![".rs"],
      Self::Node => vec![".js", ".ts"],
      Self::Maven => vec![".java"],
      Self::Other => vec![],
    }
  }

  pub fn detect(scan: &FolderScan) -> crate::Result<Vec<ProjectTypeMatch>> {
    let mut ret = vec![];
    for typ in Self::iter() {
      let mut matching_files = Self::detect_extensions(scan, typ)?;
      matching_files.append(&mut Self::detect_project_files(scan, typ)?);
      matching_files.sort();
      matching_files.dedup();
      if !matching_files.is_empty() {
        ret.push(ProjectTypeMatch {
          typ,
          path: scan.path.clone(),
          files: matching_files,
        });
      }
    }
    Ok(ret)
  }

  fn detect_extensions(scan: &FolderScan, typ: ProjectType) -> crate::Result<Vec<PathBuf>> {
    let mut ret = vec![];
    for ext in typ.language_extensions() {
      ret.append(
        &mut scan
          .files
          .iter()
          .filter(|file| {
            if let Some(file_ext) = file.extension() {
              return file_ext.eq_ignore_ascii_case(ext);
            }
            false
          })
          .map(|file| file.clone())
          .collect::<Vec<_>>(),
      );
    }
    Ok(ret)
  }

  fn detect_project_files(scan: &FolderScan, typ: ProjectType) -> crate::Result<Vec<PathBuf>> {
    let mut ret = vec![];
    for prj_file in typ.project_files() {
      let abs_path = scan.path.join(prj_file);
      if abs_path.exists() {
        ret.push(abs_path);
      }
    }
    Ok(ret)
  }
}
