use clap::ValueEnum;
use strum::{Display, EnumIter, IntoEnumIterator, VariantNames};

use crate::{Error, Project};

#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "text")]
pub mod text;
#[cfg(feature = "csv")]
pub mod csv;
#[cfg(feature = "xml")]
pub mod xml;
#[cfg(feature = "html")]
pub mod html;
#[cfg(feature = "markdown")]
pub mod markdown;

/// A project writer to support multiple output formats
pub trait ProjectMatchesFormatter {
  /// Write the given project to the output stream
  ///
  /// # Arguments
  ///
  /// * `to` - The output stream to write to
  /// * `matches` - The project to be written
  fn write<'a>(
    &'a self,
    to: &'a mut dyn std::io::Write,
    matches: &'a Vec<&'a Project>,
  ) -> crate::Result<()>;
}

/// A boxed [`ProjectWriter`]
pub type BoxedProjectMatchesFormatter = Box<dyn ProjectMatchesFormatter>;

#[derive(ValueEnum, EnumIter, VariantNames, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display, Copy, Clone)]
pub enum OutputFormat {
  #[cfg(feature = "text")]
  #[strum(serialize = "text")]
  Text,
  #[cfg(feature = "json")]
  #[strum(serialize = "json")]
  Json,
  #[cfg(feature = "csv")]
  #[strum(serialize = "csv")]
  Csv,
  #[cfg(feature = "xml")]
  #[strum(serialize = "xml")]
  Xml,
  #[cfg(feature = "html")]
  #[strum(serialize = "html")]
  Html,
  #[cfg(feature = "markdown")]
  #[strum(serialize = "markdown")]
  Markdown,
}

impl OutputFormat {
  pub fn formatter(&self) -> crate::Result<BoxedProjectMatchesFormatter> {
    match self {
      #[cfg(feature = "text")]
      Self::Text => Ok(Box::new(text::TextProjectMatchesWriter {})),
      #[cfg(feature = "json")]
      Self::Json => Ok(Box::new(json::JsonProjectMatchesWriter {})),
      #[cfg(feature = "csv")]
      Self::Csv => Ok(Box::new(csv::CsvProjectMatchesWriter {})),
      #[cfg(feature = "xml")]
      Self::Xml => Ok(Box::new(xml::XmlProjectMatchesWriter {})),
      #[cfg(feature = "html")]
      Self::Html => Ok(Box::new(html::HtmlProjectMatchesWriter {})),
      #[cfg(feature = "markdown")]
      Self::Markdown => Ok(Box::new(markdown::MarkdownProjectMatchesWriter {})),
      #[allow(unreachable_patterns)]
      _ => Err(Error::Unknown(format!("No supported output formats")))
    }
  }
}

pub fn supported_formats() -> Vec<(String, BoxedProjectMatchesFormatter)> {
  OutputFormat::iter()
    .map(|fmt| (format!("{:?}", fmt), fmt.formatter().unwrap()))
    .collect::<Vec<_>>()
}

/// Retrieve the default [`ProjectWriter`]
pub fn default_format() -> BoxedProjectMatchesFormatter {
  let mut formats = supported_formats();
  if formats.is_empty() {
    panic!("no output formats supported, enable at least one feature")
  }
  let (_, writer) = formats.remove(0);
  writer
}

/// Retrieve the corresponding format or the default one if not found
pub fn get_format_or_default<N: AsRef<str>>(name: N) -> Option<BoxedProjectMatchesFormatter> {
  let mut formats = supported_formats();
  let wanted_idx = formats.iter().enumerate().find_map(|(idx, (fmt_name, _))| {
    if fmt_name.eq_ignore_ascii_case(name.as_ref()) {
      return Some(idx);
    }
    return None;
  });
  if let Some(idx) = wanted_idx {
    let (_, writer) = formats.remove(idx);
    return Some(writer);
  }
  if !formats.is_empty() {
    let (_, writer) = formats.remove(0);
    return Some(writer);
  }
  None
}

/// Retrieve all supported format names
pub fn supported_format_names() -> Vec<String> {
  supported_formats()
    .iter()
    .map(|(name, _)| name.clone())
    .collect::<Vec<_>>()
}
