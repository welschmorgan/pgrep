use crate::{Project, ProjectMatchesFormatter};

/// The most basic project writer: a human readable list on stdout
pub struct TextProjectMatchesWriter {}

impl ProjectMatchesFormatter for TextProjectMatchesWriter {
  fn write(
    &self,
    to: &mut dyn std::io::Write,
    matches: &Vec<Project>,
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
