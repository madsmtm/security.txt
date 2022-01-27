// https://tools.ietf.org/html/draft-foudil-securitytxt-09

use chrono::prelude::*;
use core::str::FromStr;
use language_tags::LanguageTag;
use std::error::Error;
use std::fmt;
use url::Url;

/// The conventional name of the file.
pub const FILENAME: &str = "security.txt";

/// The path under which security.txt MUST be placed, when served over HTTP
pub const WELL_KNOWN_PATH: &str = "/.well-known/security.txt";

/// The required file format of the "security.txt" file (MUST be plain text).
pub const MIMETYPE: &str = "text/plain";

#[derive(Debug, PartialEq)]
pub enum Field {
    Acknowledgments(Url), // Required HTTPS?
    Canonical(Url),       // Required HTTPS?
    Contact(Url),
    Encryption(Url),
    Expires(DateTime<FixedOffset>), // Must appear only once
    Hiring(Url),                    // Required HTTPS?
    Policy(Url),
    PreferredLanguages(Vec<LanguageTag>), // Must appear only once
    Extension(String, String),
}

fn split_at_str(string: &str, pattern: char) -> Option<(&str, &str)> {
    let mut split = string.splitn(2, pattern);
    let first = split.next().unwrap();
    match split.next() {
        Some(second) => Some((first, second)),
        None => None,
    }
}

fn parse_rfc5322_datetime(string: &str) -> chrono::ParseResult<DateTime<FixedOffset>> {
    // TODO: See https://tools.ietf.org/html/rfc5322#section-3.3
    DateTime::parse_from_str(string, "")
}

impl FromStr for Field {
    type Err = ParseError;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some((name, value)) = split_at_str(string, ':') {
            return Ok(match &*name.to_lowercase() {
                "acknowledgments" => Self::Acknowledgments(Url::parse(value)?),
                "canonical" => Self::Canonical(Url::parse(value)?),
                "contact" => Self::Contact(Url::parse(value)?),
                "encryption" => Self::Encryption(Url::parse(value)?),
                "expires" => Self::Expires(parse_rfc5322_datetime(value)?),
                "hiring" => Self::Hiring(Url::parse(value)?),
                "policy" => Self::Policy(Url::parse(value)?),
                "preferred-languages" => {
                    let languages = value
                        .split(',')
                        .map(|s| LanguageTag::from_str(s))
                        .collect::<Result<_, _>>()?;
                    Self::PreferredLanguages(languages)
                }
                _ => Self::Extension(name.into(), value.into()),
            });
        }
        Err(ParseError("Missing `:`".into()))
    }
}

/// Signifies an error in the specification
#[derive(Debug, PartialEq)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseError {}

impl From<language_tags::Error> for ParseError {
    fn from(error: language_tags::Error) -> Self {
        ParseError(error.to_string())
    }
}

impl From<url::ParseError> for ParseError {
    fn from(error: url::ParseError) -> Self {
        ParseError(error.to_string())
    }
}

impl From<chrono::format::ParseError> for ParseError {
    fn from(error: chrono::format::ParseError) -> Self {
        ParseError(error.to_string())
    }
}

pub enum Line {
    Field(Field),
    Comment(String),
}

impl FromStr for Line {
    type Err = ParseError;
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if let Some(string) = string.strip_prefix("#") {
            Ok(Self::Comment(string.into()))
        } else {
            Ok(Self::Field(Field::from_str(string)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(
            Ok(Field::Acknowledgments(
                Url::parse("https://abc.com").unwrap()
            )),
            Field::from_str("Acknowledgments:https://abc.com")
        );
    }
}
