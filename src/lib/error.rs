
#[derive(Debug)]
/// This crate's error type
pub enum Error {
  Init(String),
  IO(String, Option<Box<dyn std::error::Error>>),
  Unknown(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}{}{}",
      self.kind(),
      match self.message() {
        Some(m) => format!(": {}", m),
        None => String::new(),
      },
      match self.cause() {
        Some(c) => format!("\nCaused by: {}", c),
        None => String::new(),
      }
    )
  }
}

impl Error {
  /// Modify the message, prepending `prefix` to the current message
  pub fn with_context(mut self, prefix: String) -> Self {
    match &mut self {
      Self::Init(m) => *m = format!("{}, {}", prefix, m),
      Self::IO(m, ..) => *m = format!("{}, {}", prefix, m),
      Self::Unknown(m) => *m = format!("{}, {}", prefix, m),
    };
    self
  }

  /// Retrieve the error kind
  pub fn kind<'a>(&self) -> &'a str {
    match self {
      Self::Init(..) => "Initialization",
      Self::IO(..) => "I/O",
      Self::Unknown(..) => "Unknown",
    }
  }

  /// Retrieve the stored message
  pub fn message(&self) -> Option<&String> {
    match self {
      Self::Init(m) => Some(m),
      Self::IO(m, ..) => Some(m),
      Self::Unknown(m) => Some(m),
    }
  }

  /// Retrieve the `caused by` field
  pub fn cause(&self) -> Option<&Box<dyn std::error::Error>> {
    match self {
      Self::Init(..) => None,
      Self::IO(_, c) => c.as_ref(),
      Self::Unknown(..) => None,
    }
  }
}

/// This crate's result type
pub type Result<T> = std::result::Result<T, Error>;

impl From<clap::error::Error> for Error {
  fn from(value: clap::error::Error) -> Self {
    Error::Init(value.to_string())
  }
}

impl From<log::SetLoggerError> for Error {
  fn from(value: log::SetLoggerError) -> Self {
    Error::Init(value.to_string())
  }
}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
    Error::IO(value.to_string(), None)
  }
}

impl From<toml::de::Error> for Error {
  fn from(value: toml::de::Error) -> Self {
    Error::IO(
      "failed to deserialize entity".to_string(),
      Some(Box::new(value)),
    )
  }
}

impl From<toml::ser::Error> for Error {
  fn from(value: toml::ser::Error) -> Self {
    Error::IO(
      "failed to serialize entity".to_string(),
      Some(Box::new(value)),
    )
  }
}

impl From<rmp_serde::decode::Error> for Error {
  fn from(value: rmp_serde::decode::Error) -> Self {
    Error::IO(
      "failed to deserialize entity".to_string(),
      Some(Box::new(value)),
    )
  }
}

impl From<rmp_serde::encode::Error> for Error {
  fn from(value: rmp_serde::encode::Error) -> Self {
    Error::IO(
      "failed to serialize entity".to_string(),
      Some(Box::new(value)),
    )
  }
}

impl From<chrono::OutOfRangeError> for Error {
  fn from(value: chrono::OutOfRangeError) -> Self {
    Error::Unknown(value.to_string())
  }
}
