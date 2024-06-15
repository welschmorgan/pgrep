use crate::{Project, ProjectMatchesWriter};

/// The most basic project writer: a human readable list on stdout
pub struct TextProjectMatchesWriter {}

impl ProjectMatchesWriter for TextProjectMatchesWriter {
  fn write<'a>(
    &'a self,
    to: &'a mut dyn std::io::Write,
    matches: &'a Vec<&'a Project>,
  ) -> crate::Result<()> {
    for prj in matches {
      writeln!(
        to,
        "[{}] {} - {}",
        prj
          .kinds()
          .iter()
          .map(|k| k.name())
          .collect::<Vec<_>>()
          .join(", "),
        prj.name().unwrap(),
        prj.path().display()
      )?;
    }
    return Ok(());
  }
}
