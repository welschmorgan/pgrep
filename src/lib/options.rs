use std::{path::PathBuf, str::FromStr};

use clap::Parser;

use crate::Query;

#[derive(Debug, Parser)]
#[command(version)]
#[command(author)]
#[command(about, long_about = None)]
pub struct AppOptions {
  #[arg(required_unless_present("clean_cache"), default_value("*"), next_line_help(true), help("The query used to find the project. It supports the following wildcards:\n\
\t- '?': an optional character\n\
\t- '_': a required character\n\
\t- '#': a required digit\n\
\t- '*': any string\n"), value_parser = parse_query)]
  pub query: Query,

  /// Specify a custom config file to load.
  #[arg(short, long)]
  pub config: Option<PathBuf>,

  /// Clean the cache folder and exit.
  #[arg(long, exclusive(true))]
  pub clean_cache: bool,

  /// Disable cache usage.
  #[arg(long)]
  pub no_cache: bool,
}

fn parse_query(s: &str) -> Result<Query, String> {
  Query::from_str(s).map_err(|e| format!("`{s}` isn't a valid query, {}", e))
}
