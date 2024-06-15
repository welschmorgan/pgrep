use crate::{Error, Project, ProjectMatchesWriter};

/// The most basic project writer: a human readable list on stdout
pub struct XmlProjectMatchesWriter {}

impl ProjectMatchesWriter for XmlProjectMatchesWriter {
  fn write<'a>(
    &'a self,
    to: &'a mut dyn std::io::Write,
    matches: &'a Vec<&'a Project>,
  ) -> crate::Result<()> {
    writeln!(to, "<?xml version = \"1.0\" encoding = \"UTF-8\" standalone = \"yes\" ?>")?;
    writeln!(to, "<projects>")?;
    for prj in matches {
      if prj.kinds().len() == 1 {
        writeln!(to, "\t<project name=\"{}\" path=\"{}\" kind=\"{}\"/>", prj.name().unwrap_or_default(), prj.path().display(), prj.kinds()[0].name())?;
      } else {
        writeln!(to, "\t<project name=\"{}\" path=\"{}\">", prj.name().unwrap_or_default(), prj.path().display())?;
        for k in prj.kinds() {
          writeln!(to, "\t\t<kind>{}</kind>", k.name())?;
        }
        writeln!(to, "\t</project>")?;
      }
    }
    writeln!(to, "</projects>")?;
    Ok(())
  }
}
