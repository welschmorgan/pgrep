#[derive(Debug)]
pub enum Error {
  Init(String),
  IO(String),
  Unknown(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}{}",
      self.kind(),
      match self.message() {
        Some(m) => format!(": {}", m),
        None => String::new(),
      }
    )
  }
}

impl Error {
  pub fn kind<'a>(&self) -> &'a str {
    match self {
      Self::Init(_) => "Initialization",
      Self::IO(_) => "I/O",
      Self::Unknown(_) => "Unknown",
    }
  }

  pub fn message<'a>(&'a self) -> Option<&'a String> {
    match self {
      Self::Init(m) => Some(&m),
      Self::IO(m) => Some(&m),
      Self::Unknown(m) => Some(&m),
    }
  }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<clap::error::Error> for Error {
  fn from(value: clap::error::Error) -> Self {
    Error::Init(value.to_string())
  }
}
