use std::{path::PathBuf, str::FromStr};

use clap::{ArgAction, Parser};
use strum::VariantNames;

use crate::{OutputFormat, Query};

/// The query format description for command-line use
pub const QUERY_FORMAT: &'static str = "The query used to find the project. It supports the following wildcards:\n\
\t- '?': an optional character\n\
\t- '_': a required character\n\
\t- '#': a required digit\n\
\t- '*': any string\n";

#[derive(Debug, Parser)]
#[command(version)]
#[command(author)]
#[command(about, long_about = None)]
/// The AppOptions structure represents the command-line options and values
pub struct AppOptions {
  /// The query used to filter projects
  #[arg(required_unless_present("dump_config"))]
  #[arg(required_unless_present("clean_cache"))]
  #[arg(required_unless_present("list"))]
  #[arg(default_value("*"))]
  #[arg(next_line_help(true))]
  #[arg(help(QUERY_FORMAT))]
  #[arg(value_parser = parse_query)]
  pub query: Query,

  /// Specify a custom config file to load.
  #[arg(short, long)]
  pub config: Option<PathBuf>,

  /// Dump the config to stdout then exit
  #[arg(long)]
  pub dump_config: bool,

  /// Clean the cache folder and exit.
  #[arg(long, exclusive(true))]
  pub clean_cache: bool,

  /// Disable cache usage.
  #[arg(long)]
  pub no_cache: bool,

  /// Register a new entry to the searchable folders list
  #[arg(short = 'F', long = "folder", action = ArgAction::Append)]
  pub folders: Vec<PathBuf>,

  /// Set the output format
  #[arg(long = "format", default_value = OutputFormat::VARIANTS.get(0).unwrap_or(&"text"))]
  pub format: OutputFormat,

  /// Activate terminal ui
  #[cfg(feature = "tui")]
  #[arg(long = "tui")]
  pub tui: bool,
  
  /// List project without filtering them
  #[arg(short = 'l', long = "list")]
  pub list: bool
}

/// ValueParser helper for [`clap`]
fn parse_query(s: &str) -> Result<Query, String> {
  Query::from_str(s).map_err(|e| format!("`{s}` isn't a valid query, {}", e))
}
