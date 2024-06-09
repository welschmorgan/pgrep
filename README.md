# pgrep

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-brightgreen.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/rust-1.78.0-blue.svg)](https://blog.rust-lang.org/2024/05/02/Rust-1.78.0.html)
[![Rust](https://github.com/welschmorgan/pgrep/actions/workflows/rust.yml/badge.svg)](https://github.com/welschmorgan/pgrep/actions/workflows/rust.yml)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](http://makeapullrequest.com)
[![Maturity badge - level 2](https://img.shields.io/badge/Maturity-Level%202%20--%20First%20Release-yellowgreen.svg)](https://github.com/tophat/getting-started/blob/master/scorecard.md)

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

## Features

The following list represents the planned or implemented features:

- [x] Scanning code folders
- [x] Project type matching
- [x] Caching of results with duration bound busting
- [ ] Per project actions
  - [ ] build
  - [ ] clean
  - [ ] backup
- [ ] Search in source code

## Prerequisites

A working rust 1.7+ toolchain (`1.78.0`).

## Installation

To build and install the binaries to `~/.cargo/bin` use:

```shell
cargo install --path .
```

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