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

## Author

Morgan Welsch <welschmorgan@gmail.com>