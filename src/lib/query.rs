use std::{fmt::Display, str::FromStr};

use crate::Error;

/// Some part of a [`Query`] expression
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Part {
  /// An optional character. Corresponds to `?`
  OptionalChar,
  /// A required character. Corresponds to `_`
  RequiredChar,
  /// Any string. Corresponds to `*`
  AnyStr,
  /// A contiguous stream of integers. Corresponds to `#`
  Integer,
  /// A fixed-length string
  Fixed(String),
}

/// Represents a match against a string and a [`Query`]. This is an [`Option`] equivalent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PartMatch {
  /// The part was matched successfully, and the cursor must be advanced by as much chars
  Success(usize),
  /// The part failed to match
  Failure,
}

impl PartMatch {
  /// Helper to find if the part matched successfully
  pub fn is_success(&self) -> bool {
    match self {
      Self::Success(..) => true,
      Self::Failure => false,
    }
  }

  /// Helper to find if the part did NOT match
  pub fn is_failure(&self) -> bool {
    match self {
      Self::Success(..) => false,
      Self::Failure => true,
    }
  }
}

/// A glob-like pattern for filtering [`crate::project::Project`]s
///
/// It supports the following wildcards:
///   - '?': an optional character
///   - '_': a required character
///   - '#': a required digit
///   - '*': any string
///
/// # Examples
///
/// ```
/// use pgrep::Query;
///
/// let q = "abc*".parse::<Query>().unwrap(); // accepts 'abcdefg' and 'abc' but not 'zabc'
/// let q = "*abc".parse::<Query>().unwrap(); // accepts '123abc' and 'abc' but not 'abcz'
/// let q = "abc#".parse::<Query>().unwrap(); // accepts 'abc1' and 'abc2345' but not 'abcz' or 'abc'
/// let q = "abc?".parse::<Query>().unwrap(); // accepts 'abc' and 'abca' but not 'abczd'
/// let q = "abc_".parse::<Query>().unwrap(); // accepts 'abc1' and 'abcz' but not 'abc' or 'abczz'
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Query {
  /// The original expression
  expr: String,
  /// The parsed parts
  parts: Vec<Part>,
}

impl Query {
  /// Internal part-matching logic implementation.
  ///
  /// # Arguments
  ///
  /// `p` - The current part to match
  /// `part_it` - The part iterator, used to detect next part when matching [`Part::AnyStr`]
  /// `expr` - The expression to match against
  /// `ch_id` - The current expression character being matched against
  fn match_part(
    p: &Part,
    part_it: &mut std::slice::Iter<Part>,
    expr: &str,
    mut ch_id: usize,
  ) -> PartMatch {
    // println!("  match {:?} on {:?} | {} / {}", p, expr.chars().nth(ch_id), ch_id, expr.len());
    match p {
      Part::OptionalChar => {
        ch_id += 1;
      }
      Part::RequiredChar => {
        if ch_id >= expr.len() {
          return PartMatch::Failure;
        }
        ch_id += 1;
      }
      Part::AnyStr => {
        if let Some(next) = part_it.next() {
          let mut found = false;
          while ch_id < expr.len() {
            if let PartMatch::Success(next_ch_id) = Self::match_part(next, part_it, expr, ch_id) {
              ch_id = next_ch_id;
              found = true;
              break;
            }
            ch_id += 1
          }
          if !found {
            return PartMatch::Failure;
          }
        } else {
          ch_id = expr.len()
        }
      }
      Part::Integer => {
        let orig_ch_id = ch_id;
        while ch_id < expr.len() {
          if !expr.chars().nth(ch_id).unwrap().is_numeric() {
            break;
          }
          ch_id += 1;
        }
        if orig_ch_id == ch_id {
          return PartMatch::Failure;
        }
      }
      Part::Fixed(s) => {
        let mut s_id = 0;
        while ch_id < expr.len() && s_id < s.len() {
          let s_ch = s.chars().nth(s_id);
          let e_ch = expr.chars().nth(ch_id);
          if s_ch.unwrap().to_ascii_lowercase() != e_ch.unwrap().to_ascii_lowercase() {
            return PartMatch::Failure;
          }
          s_id += 1;
          ch_id += 1;
        }
        if s_id < s.len() {
          return PartMatch::Failure;
        }
      }
    }
    PartMatch::Success(ch_id)
  }

  /// Check if this [`Query`] matches the given expression
  ///
  /// # Arguments
  ///
  /// `expr` - Anything that can be considered a string ref. In the form `*abc?`
  pub fn matches<S: AsRef<str>>(&self, expr: S) -> bool {
    // println!("matches {} on '{}'", self.expr, expr.as_ref());
    let mut ch_id = 0;
    let mut part_it = self.parts.iter();
    let mut last_match = PartMatch::Failure;
    while let Some(part) = part_it.next() {
      last_match = Self::match_part(part, &mut part_it, expr.as_ref(), ch_id);
      match last_match {
        PartMatch::Success(next_ch_id) => ch_id = next_ch_id,
        _ => break,
      }
    }
    let next_part = part_it.next();
    next_part.is_none() && ch_id >= expr.as_ref().len() && last_match.is_success()
  }
}

impl FromStr for Query {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let expr = s.trim().to_string();
    if expr.is_empty() {
      return Err(Error::IO(format!("cannot parse empty query"), None));
    }
    let mut parts = vec![];
    for ch in expr.chars() {
      match ch {
        '?' => parts.push(Part::OptionalChar),
        '_' => parts.push(Part::RequiredChar),
        '*' => parts.push(Part::AnyStr),
        '#' => parts.push(Part::Integer),
        ch => {
          let mut done = false;
          if !parts.is_empty() {
            if let Part::Fixed(s) = parts.last_mut().unwrap() {
              s.push(ch);
              done = true;
            }
          }
          if !done {
            parts.push(Part::Fixed(ch.to_string()));
          }
        }
      }
    }
    Ok(Self { expr, parts })
  }
}

impl Display for Query {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.expr)
  }
}

impl Default for Query {
  fn default() -> Self {
    Self {
      expr: "*".to_string(),
      parts: vec![Part::AnyStr],
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::Query;

  fn run_cases(cases: &[(&str, &str, bool)]) {
    for (query, subject, expected) in cases {
      let query = query.parse::<Query>().unwrap();
      assert_eq!(
        query.matches(subject),
        *expected,
        "\nquery = {}, subject = {}",
        query.expr,
        subject
      );
    }
  }

  #[test]
  fn fixed() {
    run_cases(&[("myTest", "MYTEST", true)])
  }

  #[test]
  fn star() {
    run_cases(&[
      ("*test*", "mytestproject", true),
      ("*test*", "project", false),
      ("mytest*", "mytestproject", true),
      ("nytest*", "mytestproject", false),
    ]);
  }

  #[test]
  fn optional_char() {
    run_cases(&[
      ("test?", "test", true),
      ("test?", "test2", true),
      ("test", "test2", false),
    ]);
  }

  #[test]
  fn required_char() {
    run_cases(&[
      ("test_", "test", false),
      ("test_", "test2", true),
      ("test", "test2", false),
    ]);
  }

  #[test]
  fn digit() {
    run_cases(&[("test#", "test", false), ("test#", "test2", true)]);
  }
}
