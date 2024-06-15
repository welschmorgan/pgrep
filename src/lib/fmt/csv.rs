use crate::{Project, ProjectMatchesFormatter};

/// The most basic project writer: a human readable list on stdout
pub struct CsvProjectMatchesWriter {}

impl ProjectMatchesFormatter for CsvProjectMatchesWriter {
  fn write(
    &self,
    to: &mut dyn std::io::Write,
    matches: &Vec<Project>,
  ) -> crate::Result<()> {
    let mut rows = vec![
      vec!["Language".to_string(), "Name".to_string(), "Path".to_string()]
    ];
    for prj in matches {
      rows.push(vec![
        prj.kinds().iter().map(|k| format!("{}", k.name())).collect::<Vec<_>>().join("+"), 
        prj.name().unwrap_or_default(), 
        format!("{}", prj.path().display())
      ]);
    }
    for row in rows {
      writeln!(
        to,
        "{}",
        row
          .iter()
          .map(|v| format!("\"{}\"", v))
          .collect::<Vec<_>>()
          .join(",")
      )?;
    }
    Ok(())
  }
}
