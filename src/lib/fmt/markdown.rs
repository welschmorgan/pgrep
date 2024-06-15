use crate::{Project, ProjectMatchesFormatter};

/// The most basic project writer: a human readable list on stdout
pub struct MarkdownProjectMatchesWriter {}

impl ProjectMatchesFormatter for MarkdownProjectMatchesWriter {
  fn write(
    &self,
    to: &mut dyn std::io::Write,
    matches: &Vec<Project>,
  ) -> crate::Result<()> {
    writeln!(to, "# Projects")?;
    writeln!(to, "")?;
    struct Column(usize);
    let mut rows: Vec<[String; 3]> = vec![[
      "Language".to_string(),
      "Name".to_string(),
      "Path".to_string(),
    ]];
    let mut cols = vec![
      Column(rows[0][0].len()),
      Column(rows[0][1].len()),
      Column(rows[0][2].len()),
    ];
    for prj in matches {
      let row = [
        prj
          .kinds()
          .iter()
          .map(|k| k.name())
          .collect::<Vec<_>>()
          .join(","),
        prj.name().unwrap_or_default(),
        format!("{}", prj.path().display()),
      ];
      for i in 0..cols.len() {
        cols[i] = Column(cols[i].0.max(row[i].len()));
      }
      rows.push(row);
    }
    for (row_id, row) in rows.iter().enumerate() {
      let cells = row
        .iter()
        .enumerate()
        .map(|(cell_id, cell)| format!("{:0width$}", cell, width = cols[cell_id].0))
        .collect::<Vec<_>>();
      writeln!(to, "| {} |", cells.join(" | "))?;
      if row_id == 0 {
        writeln!(
          to,
          "| {} |",
          cells
            .iter()
            .map(|c| "-".repeat(c.len()))
            .collect::<Vec<_>>()
            .join(" | ")
        )?;
      }
    }
    Ok(())
  }
}
