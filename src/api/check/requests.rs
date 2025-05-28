//! Structures for `check` requests.

use super::{serialize_option_vec_string, Data};
use std::{borrow::Cow, mem, ops::Deref};

#[cfg(feature = "cli")]
use clap::ValueEnum;
use lifetime::IntoStatic;
use serde::{Serialize, Serializer};

use crate::error::{Error, Result};

/// Possible levels for additional rules.
///
/// Currently, `Level::Picky` adds additional rules
/// with respect to `Level::Default`.
#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Hash)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Level {
    /// Default level.
    #[default]
    Default,
    /// Picky level.
    Picky,
}

impl Level {
    /// Return `true` if current level is the default one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use languagetool_rust::api::check::Level;
    ///
    /// let level: Level = Default::default();
    ///
    /// assert!(level.is_default());
    /// ```
    #[must_use]
    pub fn is_default(&self) -> bool {
        *self == Level::default()
    }
}

/// Split a string into as few fragments as possible, where each fragment
/// contains (if possible) a maximum of `n` characters. Pattern str `pat` is
/// used for splitting.
///
/// # Examples
///
/// ```
/// # use languagetool_rust::api::check::split_len;
/// let s = "I have so many friends.
/// They are very funny.
/// I think I am very lucky to have them.
/// One day, I will write them a poem.
/// But, in the meantime, I write code.
/// ";
///
/// let split = split_len(&s, 40, "\n");
///
/// assert_eq!(split.join(""), s);
/// assert_eq!(
///     split,
///     vec![
///         "I have so many friends.\n",
///         "They are very funny.\n",
///         "I think I am very lucky to have them.\n",
///         "One day, I will write them a poem.\n",
///         "But, in the meantime, I write code.\n"
///     ]
/// );
///
/// let split = split_len(&s, 80, "\n");
///
/// assert_eq!(
///     split,
///     vec![
///         "I have so many friends.\nThey are very funny.\n",
///         "I think I am very lucky to have them.\nOne day, I will write them a poem.\n",
///         "But, in the meantime, I write code.\n"
///     ]
/// );
///
/// let s = "I have so many friends.
/// They are very funny.
/// I think I am very lucky to have them.
///
/// One day, I will write them a poem.
/// But, in the meantime, I write code.
/// ";
///
/// let split = split_len(&s, 80, "\n\n");
///
/// println!("{:?}", split);
///
/// assert_eq!(
///     split,
///     vec![
///         "I have so many friends.\nThey are very funny.\nI think I am very lucky to have \
///          them.\n\n",
///         "One day, I will write them a poem.\nBut, in the meantime, I write code.\n"
///     ]
/// );
/// ```
#[must_use]
pub fn split_len<'source>(s: &'source str, n: usize, pat: &str) -> Vec<&'source str> {
    let mut vec: Vec<&'source str> = Vec::with_capacity(s.len() / n);
    let mut splits = s.split_inclusive(pat);

    let mut start = 0;
    let mut i = 0;

    if let Some(split) = splits.next() {
        vec.push(split);
    } else {
        return Vec::new();
    }

    for split in splits {
        let new_len = vec[i].len() + split.len();
        if new_len < n {
            vec[i] = &s[start..start + new_len];
        } else {
            vec.push(split);
            start += vec[i].len();
            i += 1;
        }
    }

    vec
}

/// Default value for [`Request::language`].
pub const DEFAULT_LANGUAGE: &str = "auto";

/// Custom serialization for [`Request::language`].
fn serialize_language<S>(lang: &str, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(if lang.is_empty() {
        DEFAULT_LANGUAGE
    } else {
        lang
    })
}

/// LanguageTool POST check request.
///
/// The main feature - check a text with LanguageTool for possible style and
/// grammar issues.
///
/// The structure below tries to follow as closely as possible the JSON API
/// described [here](https://languagetool.org/http-api/swagger-ui/#!/default/post_check).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Hash, IntoStatic)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Request<'source> {
    /// The text to be checked. This or 'data' is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Cow<'source, str>>,
    /// The text to be checked, given as a JSON document that specifies what's
    /// text and what's markup. This or 'text' is required.
    ///
    /// Markup will be ignored when looking for errors. Example text:
    /// ```html
    /// A <b>test</b>
    /// ```
    /// JSON for the example text:
    /// ```json
    /// {"annotation":[
    ///  {"text": "A "},
    ///  {"markup": "<b>"},
    ///  {"text": "test"},
    ///  {"markup": "</b>"}
    /// ]}
    /// ```
    /// If you have markup that should be interpreted as whitespace, like `<p>`
    /// in HTML, you can have it interpreted like this:
    ///
    /// ```json
    /// {"markup": "<p>", "interpretAs": "\n\n"}
    /// ```
    /// The 'data' feature is not limited to HTML or XML, it can be used for any
    /// kind of markup. Entities will need to be expanded in this input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Data<'source>>,
    /// A language code like `en-US`, `de-DE`, `fr`, or `auto` to guess the
    /// language automatically (see `preferredVariants` below).
    ///
    /// For languages with variants (English, German, Portuguese) spell checking
    /// will only be activated when you specify the variant, e.g. `en-GB`
    /// instead of just `en`.
    #[serde(serialize_with = "serialize_language")]
    pub language: String,
    /// Set to get Premium API access: Your username/email as used to log in at
    /// languagetool.org.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Set to get Premium API access: your API key (see <https://languagetool.org/editor/settings/api>).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Comma-separated list of dictionaries to include words from; uses special
    /// default dictionary if this is unset.
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub dicts: Option<Vec<String>>,
    /// A language code of the user's native language, enabling false friends
    /// checks for some language pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mother_tongue: Option<String>,
    /// Comma-separated list of preferred language variants.
    ///
    /// The language detector used with `language=auto` can detect e.g. English,
    /// but it cannot decide whether British English or American English is
    /// used. Thus this parameter can be used to specify the preferred variants
    /// like `en-GB` and `de-AT`. Only available with `language=auto`. You
    /// should set variants for at least German and English, as otherwise the
    /// spell checking will not work for those, as no spelling dictionary can be
    /// selected for just `en` or `de`.
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub preferred_variants: Option<Vec<String>>,
    /// IDs of rules to be enabled, comma-separated.
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub enabled_rules: Option<Vec<String>>,
    /// IDs of rules to be disabled, comma-separated.
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub disabled_rules: Option<Vec<String>>,
    /// IDs of categories to be enabled, comma-separated.
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub enabled_categories: Option<Vec<String>>,
    /// IDs of categories to be disabled, comma-separated.
    #[serde(serialize_with = "serialize_option_vec_string")]
    pub disabled_categories: Option<Vec<String>>,
    /// If true, only the rules and categories whose IDs are specified with
    /// `enabledRules` or `enabledCategories` are enabled.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub enabled_only: bool,
    /// If set to `picky`, additional rules will be activated, i.e. rules that
    /// you might only find useful when checking formal text.
    #[serde(skip_serializing_if = "Level::is_default")]
    pub level: Level,
}

impl<'source> Request<'source> {
    /// Create a new empty request with language set to `"auto"`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            language: "auto".to_string(),
            ..Default::default()
        }
    }

    /// Set the text to be checked and remove potential data field.
    #[must_use]
    pub fn with_text<T: Into<Cow<'source, str>>>(mut self, text: T) -> Self {
        self.text = Some(text.into());
        self.data = None;
        self
    }

    /// Set the data to be checked and remove potential text field.
    #[must_use]
    pub fn with_data(mut self, data: Data<'source>) -> Self {
        self.data = Some(data);
        self.text = None;
        self
    }

    /// Set the data (obtained from string) to be checked and remove potential
    /// text field
    pub fn with_data_str(self, data: &str) -> serde_json::Result<Self> {
        serde_json::from_str(data).map(|data| self.with_data(data))
    }

    /// Set the language of the text / data.
    #[must_use]
    pub fn with_language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    /// Return the text within the request.
    ///
    /// # Errors
    ///
    /// If both `self.text` and `self.data` are [`None`].
    /// If any data annotation does not contain text or markup.
    pub fn try_get_text(&self) -> Result<Cow<'source, str>> {
        if let Some(ref text) = self.text {
            Ok(text.clone())
        } else if let Some(ref data) = self.data {
            match data.annotation.len() {
                0 => Ok(Default::default()),
                1 => data.annotation[0].try_get_text(),
                _ => {
                    let mut text = String::new();

                    for da in data.annotation.iter() {
                        text.push_str(da.try_get_text()?.deref());
                    }

                    Ok(Cow::Owned(text))
                },
            }
        } else {
            Err(Error::InvalidRequest(
                "missing either text or data field".to_string(),
            ))
        }
    }

    /// Return a copy of the text within the request.
    /// Call [`Request::try_get_text`] but panic on error.
    ///
    /// # Panics
    ///
    /// If both `self.text` and `self.data` are [`None`].
    /// If any data annotation does not contain text or markup.
    #[must_use]
    pub fn get_text(&self) -> Cow<'source, str> {
        self.try_get_text().unwrap()
    }

    /// Split this request into multiple, using [`split_len`] function to split
    /// text.
    ///
    /// # Errors
    ///
    /// If `self.text` is [`None`] and `self.data` is [`None`].
    pub fn try_split(mut self, n: usize, pat: &str) -> Result<Vec<Self>> {
        // DATA ANNOTATIONS
        if let Some(data) = mem::take(&mut self.data) {
            return Ok(data
                .split(n, pat)
                .into_iter()
                .map(|d| self.clone().with_data(d))
                .collect());
        }

        // TEXT
        let text = mem::take(&mut self.text)
            .ok_or_else(|| Error::InvalidRequest("missing text or data field".to_string()))?;
        let string: &str = match &text {
            Cow::Owned(s) => s.as_str(),
            Cow::Borrowed(s) => s,
        };

        Ok(split_len(string, n, pat)
            .iter()
            .map(|text_fragment| {
                self.clone()
                    .with_text(Cow::Owned(text_fragment.to_string()))
            })
            .collect())
    }

    /// Split this request into multiple, using [`split_len`] function to split
    /// text.
    /// Call [`Request::try_split`] but panic on error.
    ///
    /// # Panics
    ///
    /// If `self.text` is none.
    #[must_use]
    pub fn split(self, n: usize, pat: &str) -> Vec<Self> {
        self.try_split(n, pat).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::check::DataAnnotation;

    use super::*;

    #[test]
    fn test_with_text() {
        let req = Request::default().with_text("hello");

        assert_eq!(req.text.unwrap(), "hello");
        assert!(req.data.is_none());
    }

    #[test]
    fn test_with_data() {
        let req =
            Request::default().with_data([DataAnnotation::new_text("hello")].into_iter().collect());

        assert_eq!(
            req.data.unwrap().annotation[0],
            DataAnnotation::new_text("hello")
        );
    }

    #[test]
    fn test_with_data_str() {
        let req = Request::default()
            .with_data_str("{\"annotation\":[{\"text\": \"hello\"}]}")
            .unwrap();
        assert_eq!(
            req.data.unwrap().annotation[0],
            DataAnnotation::new_text("hello")
        );

        // Not a data annotation
        assert!(Request::default().with_data_str("hello").is_err());
    }

    #[test]
    fn test_with_language() {
        assert_eq!(
            Request::default().with_language("en-US".into()).language,
            "en-US".to_string()
        );
    }
}
