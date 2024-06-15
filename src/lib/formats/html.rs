use crate::{Error, Project, ProjectMatchesWriter};

/// The most basic project writer: a human readable list on stdout
pub struct HtmlProjectMatchesWriter {}

pub const HTML_TEMPLATE: &'static str = "
<!DOCTYPE html>
<html lang=\"en\">
  <head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <meta http-equiv=\"X-UA-Compatible\" content=\"ie=edge\">
    <title>Discovered projects</title>
    <!--<link rel=\"stylesheet\" href=\"style.css\">-->
  </head>
  <body>
    <form onsubmit=\"filter(); return false\">
      <input type=\"text\" id=\"query\" autofocus> <button type='button' onclick='clearFilter()'>Clear</button>
    </form>
    <table>
      <thead>
{{HEADER}}
      </thead>
      <tbody>
{{BODY}}
      </tbody>
    </table>
  <script type=\"text/javascript\">
    function filter() {
      let qelem = document.getElementById('query');
      let elems = document.querySelectorAll('tr[path]');
      let res = [];
      let q = qelem.value.toLowerCase();
      for (const elem of elems) {
        const path = elem.getAttribute('path');
        const kinds = elem.getAttribute('kinds');
        const name = elem.getAttribute('name');
        if (path && path.toLowerCase().includes(q) || kinds && kinds.toLowerCase().includes(q) || name && name.toLowerCase().includes(q)) {
          res.push(elem);
          elem.style.display = 'table-row';
        } else {
          elem.style.display = 'none';
        }
      }
    }

    function clearFilter() {
      let qelem = document.getElementById('query');
      let elems = document.getElementsByTagName('TR');
      let res = [];
      let q = qelem.value;
      for (const elem of elems) {
        elem.style.display = 'table-row';
      }
    }
  </script>
  </body>
</html>";

impl ProjectMatchesWriter for HtmlProjectMatchesWriter {
  fn write<'a>(
    &'a self,
    to: &'a mut dyn std::io::Write,
    matches: &'a Vec<&'a Project>,
  ) -> crate::Result<()> {
    let header = vec!["Language", "Name", "Path"]
      .iter()
      .map(|val| format!("<th>{}</th>", val))
      .fold(String::new(), |mut prev, cur| {
        if !prev.is_empty() {
          prev.push('\n');
        }
        prev.push_str(&cur);
        prev
      });
    let body = matches
      .iter()
      .map(|proj| {
        let kinds = proj
        .kinds()
        .iter()
        .map(|k| k.name())
        .collect::<Vec<_>>()
        .join(",");
      let name = proj.name().unwrap_or_default();
      let path = format!("{}", proj.path().display());
        format!(
          "<tr path=\"{path}\" name=\"{name}\" kinds=\"{kinds}\"><td>{}</td><td>{}</td><td>{}</td></tr>",
          kinds, name, path
        )
      })
      .fold(String::new(), |mut prev, cur| {
        if !prev.is_empty() {
          prev.push('\n');
        }
        prev.push_str(&cur);
        prev
      });
    writeln!(
      to,
      "{}",
      HTML_TEMPLATE
        .replace("{{BODY}}", &body)
        .replace("{{HEADER}}", &header)
    )?;
    Ok(())
  }
}
