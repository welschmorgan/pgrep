# pgrep

The rust project grepper, a developper tool to help manage tracking of source code projects.

It allows fast filtering, selection and quick actions on the found projects.

Ever wondered where a project you did ages ago lies ? Use `pgrep 'project'` to quickly find where it is.

## Showcase: Wildcard filtering

The following `test.toml` file:
```toml
[general]
folders = ["~/development"]
```

Yields the following results:

![showcase-simple-filter](img/showcase-simple-filter.gif)

## Creating custom project detection rules

You can customize recognized project using the user-configuration file (usually `~/.config/pgrep/pgrep.toml` on linux).

To do so, add the following definition:

```toml
[[general.project_kinds]]
[general.project_kinds.Custom]
name = "Rust"
language_exts = ["rs"]
project_files = ["Cargo.toml"]
```

## Author

Morgan Welsch <welschmorgan@gmail.com>