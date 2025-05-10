//! Structures for handling data annotations.

use crate::error::{Error, Result};

use std::{borrow::Cow, mem};

use lifetime::IntoStatic;
use serde::{Deserialize, Serialize};

/// A portion of text to be checked.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize, Hash, IntoStatic)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct DataAnnotation<'source> {
    /// Text that should be treated as normal text.
    ///
    /// This or `markup` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Cow<'source, str>>,
    /// Text that should be treated as markup.
    ///
    /// This or `text` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markup: Option<Cow<'source, str>>,
    /// If set, the markup will be interpreted as this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpret_as: Option<Cow<'source, str>>,
}

impl<'source> DataAnnotation<'source> {
    /// Instantiate a new `DataAnnotation` with text only.
    #[inline]
    #[must_use]
    pub fn new_text<T: Into<Cow<'source, str>>>(text: T) -> Self {
        Self {
            text: Some(text.into()),
            markup: None,
            interpret_as: None,
        }
    }

    /// Instantiate a new `DataAnnotation` with markup only.
    #[inline]
    #[must_use]
    pub fn new_markup<M: Into<Cow<'source, str>>>(markup: M) -> Self {
        Self {
            text: None,
            markup: Some(markup.into()),
            interpret_as: None,
        }
    }

    /// Instantiate a new `DataAnnotation` with markup and its interpretation.
    #[inline]
    #[must_use]
    pub fn new_interpreted_markup<M: Into<Cow<'source, str>>, I: Into<Cow<'source, str>>>(
        markup: M,
        interpret_as: I,
    ) -> Self {
        Self {
            interpret_as: Some(interpret_as.into()),
            markup: Some(markup.into()),
            text: None,
        }
    }

    /// Return the text or markup within the data annotation.
    ///
    /// # Errors
    ///
    /// If this data annotation does not contain text or markup.
    pub fn try_get_text(&self) -> Result<Cow<'source, str>> {
        if let Some(ref text) = self.text {
            Ok(text.clone())
        } else if let Some(ref markup) = self.markup {
            Ok(markup.clone())
        } else {
            Err(Error::InvalidDataAnnotation(format!(
                "missing either text or markup field in {self:?}"
            )))
        }
    }
}

#[cfg(test)]
mod data_annotation_tests {

    use super::DataAnnotation;

    #[test]
    fn test_text() {
        let da = DataAnnotation::new_text("Hello");

        assert_eq!(da.text.unwrap(), "Hello");
        assert!(da.markup.is_none());
        assert!(da.interpret_as.is_none());
    }

    #[test]
    fn test_markup() {
        let da = DataAnnotation::new_markup("<a>Hello</a>");

        assert!(da.text.is_none());
        assert_eq!(da.markup.unwrap(), "<a>Hello</a>");
        assert!(da.interpret_as.is_none());
    }

    #[test]
    fn test_interpreted_markup() {
        let da = DataAnnotation::new_interpreted_markup("<a>Hello</a>", "Hello");

        assert!(da.text.is_none());
        assert_eq!(da.markup.unwrap(), "<a>Hello</a>");
        assert_eq!(da.interpret_as.unwrap(), "Hello");
    }
}

/// Alternative text to be checked.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Data<'source> {
    /// Vector of markup text, see [`DataAnnotation`].
    pub annotation: Vec<DataAnnotation<'source>>,
}

impl Data<'_> {
    /// Split data into as few fragments as possible, where each fragment
    /// contains (if possible) a maximum of `n` characters in it's
    /// annotations' markup and text fields.
    ///
    /// Pattern str `pat` is used for splitting.
    #[must_use]
    pub fn split(self, n: usize, pat: &str) -> Vec<Self> {
        // Build vec of breakpoints and the length of the text + markup at that
        // potential breakpoint
        let mut break_point_lengths = vec![];
        let mut len = 0;
        for (i, ann) in self.annotation.iter().enumerate() {
            len +=
                ann.text.as_deref().unwrap_or("").len() + ann.markup.as_deref().unwrap_or("").len();
            if ann.text.as_ref().is_some_and(|t| t.contains(pat)) {
                break_point_lengths.push((i, len));
            }
        }

        // Decide which breakpoints to split the annotations at
        let mut break_points: Vec<usize> = vec![];
        if break_point_lengths.len() > 1 {
            let (mut i, mut ii) = (0, 1);
            let (mut base, mut curr) = (0, 0);
            while ii < break_point_lengths.len() {
                curr += break_point_lengths[i].1 - base;

                if break_point_lengths[ii].1 - base + curr > n {
                    break_points.push(break_point_lengths[i].0);
                    base = break_point_lengths[i].1;
                    curr = 0;
                }

                i += 1;
                ii += 1;
            }
        }

        // Split annotations based on calculated break points
        let mut split = Vec::with_capacity(break_points.len());
        let mut iter = self.into_iter();
        let mut taken = 0;
        let mut annotations = vec![];
        for break_point in break_points {
            while taken != break_point + 1 {
                annotations.push(iter.next().unwrap());
                taken += 1;
            }
            split.push(Data::from_iter(mem::take(&mut annotations)));
        }

        split
    }
}

impl IntoStatic for Data<'_> {
    type Static = Data<'static>;
    fn into_static(self) -> Self::Static {
        Data {
            annotation: self
                .annotation
                .into_iter()
                .map(IntoStatic::into_static)
                .collect(),
        }
    }
}

impl<'source, T: Into<DataAnnotation<'source>>> FromIterator<T> for Data<'source> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let annotation = iter.into_iter().map(std::convert::Into::into).collect();
        Data { annotation }
    }
}

impl<'source> IntoIterator for Data<'source> {
    type Item = DataAnnotation<'source>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.annotation.into_iter()
    }
}

impl Serialize for Data<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = std::collections::HashMap::new();
        map.insert("annotation", &self.annotation);

        serializer.serialize_str(&serde_json::to_string(&map).unwrap())
    }
}

#[cfg(feature = "cli")]
impl std::str::FromStr for Data<'_> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let v: Self = serde_json::from_str(s)?;
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::super::{Data, DataAnnotation};

    #[derive(Debug)]
    enum Token<'source> {
        Text(&'source str),
        Skip(&'source str),
    }

    impl<'source> From<&'source str> for Token<'source> {
        fn from(s: &'source str) -> Self {
            if s.chars().all(|c| c.is_ascii_alphabetic()) {
                Token::Text(s)
            } else {
                Token::Skip(s)
            }
        }
    }

    impl<'source> From<Token<'source>> for DataAnnotation<'source> {
        fn from(token: Token<'source>) -> Self {
            match token {
                Token::Text(s) => DataAnnotation::new_text(s),
                Token::Skip(s) => DataAnnotation::new_markup(s),
            }
        }
    }

    #[test]
    fn test_data_annotation() {
        let words: Vec<&str> = "My name is Q34XY".split(' ').collect();
        let data: Data = words.iter().map(|w| Token::from(*w)).collect();

        let expected_data = Data {
            annotation: vec![
                DataAnnotation::new_text("My"),
                DataAnnotation::new_text("name"),
                DataAnnotation::new_text("is"),
                DataAnnotation::new_markup("Q34XY"),
            ],
        };

        assert_eq!(data, expected_data);
    }

    #[test]
    fn test_try_get_text() {
        const TEXT: &str = "Lorem Ipsum";
        assert_eq!(
            DataAnnotation::new_text(TEXT).try_get_text().unwrap(),
            Cow::from(TEXT)
        );
        assert_eq!(
            DataAnnotation::new_markup(TEXT).try_get_text().unwrap(),
            Cow::from(TEXT)
        );
        assert!((DataAnnotation {
            text: None,
            markup: None,
            interpret_as: None
        })
        .try_get_text()
        .is_err());
    }
}
