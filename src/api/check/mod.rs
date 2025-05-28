//! Structures for `check` requests and responses.

mod data_annotations;
mod requests;
mod responses;

pub use data_annotations::*;
pub use requests::*;
pub use responses::*;
use serde::Serializer;

use crate::error::{Error, Result};

/// Parse `v` is valid language code.
///
/// A valid language code is usually
/// - a two character string matching pattern `[a-z]{2}
/// - a five character string matching pattern `[a-z]{2}-[A-Z]{2}
/// - or some more complex ascii string (see below)
///
/// Language code is case-insensitive.
///
/// Therefore, a valid language code must match the following:
///
/// - `[a-zA-Z]{2,3}(-[a-zA-Z]{2}(-[a-zA-Z]+)*)?`
///
/// or
///
/// - "auto"
///
/// > Note: a valid language code does not mean that it exists.
///
/// # Examples
///
/// ```
/// # use languagetool_rust::api::check::parse_language_code;
/// assert!(parse_language_code("en").is_ok());
///
/// assert!(parse_language_code("en-US").is_ok());
///
/// assert!(parse_language_code("en-us").is_ok());
///
/// assert!(parse_language_code("ca-ES-valencia").is_ok());
///
/// assert!(parse_language_code("abcd").is_err());
///
/// assert!(parse_language_code("en_US").is_err());
///
/// assert!(parse_language_code("fr-french").is_err());
///
/// assert!(parse_language_code("some random text").is_err());
/// ```
#[cfg(feature = "cli")]
pub fn parse_language_code(v: &str) -> Result<String> {
    #[inline]
    fn is_match(v: &str) -> bool {
        let mut splits = v.split('-');

        match splits.next() {
            Some(s)
                if (s.len() == 2 || s.len() == 3) && s.chars().all(|c| c.is_ascii_alphabetic()) => {
            },
            _ => return false,
        }

        match splits.next() {
            Some(s) if s.len() != 2 || s.chars().any(|c| !c.is_ascii_alphabetic()) => return false,
            Some(_) => (),
            None => return true,
        }
        for s in splits {
            if !s.chars().all(|c| c.is_ascii_alphabetic()) {
                return false;
            }
        }
        true
    }

    if v == "auto" || is_match(v) {
        Ok(v.to_string())
    } else {
        Err(Error::InvalidValue(
            "The value should be `\"auto\"` or match regex pattern: \
             ^[a-zA-Z]{2,3}(-[a-zA-Z]{2}(-[a-zA-Z]+)*)?$"
                .to_string(),
        ))
    }
}

/// Utility function to serialize a optional vector a strings
/// into a comma separated list of strings.
///
/// This is required by reqwest's RequestBuilder, otherwise it
/// will not work.
pub(crate) fn serialize_option_vec_string<S>(
    v: &Option<Vec<String>>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match v {
        Some(v) if v.len() == 1 => serializer.serialize_str(&v[0]),
        Some(v) if v.len() > 1 => {
            let size = v.iter().map(|s| s.len()).sum::<usize>() + v.len() - 1;
            let mut string = String::with_capacity(size);

            string.push_str(&v[0]);

            for s in &v[1..] {
                string.push(',');
                string.push_str(s);
            }

            serializer.serialize_str(string.as_ref())
        },
        _ => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_option_vec_string() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct Foo {
            #[serde(serialize_with = "serialize_option_vec_string")]
            values: Option<Vec<String>>,
        }

        impl Foo {
            fn new<I, T>(values: I) -> Self
            where
                I: IntoIterator<Item = T>,
                T: ToString,
            {
                Self {
                    values: Some(values.into_iter().map(|v| v.to_string()).collect()),
                }
            }
            fn none() -> Self {
                Self { values: None }
            }
        }

        let got = serde_json::to_string(&Foo::new(vec!["en-US", "de-DE"])).unwrap();
        assert_eq!(got, r#"{"values":"en-US,de-DE"}"#);

        let got = serde_json::to_string(&Foo::new(vec!["en-US"])).unwrap();
        assert_eq!(got, r#"{"values":"en-US"}"#);

        let got = serde_json::to_string(&Foo::new(Vec::<String>::new())).unwrap();
        assert_eq!(got, r#"{"values":null}"#);

        let got = serde_json::to_string(&Foo::none()).unwrap();
        assert_eq!(got, r#"{"values":null}"#);
    }
}
