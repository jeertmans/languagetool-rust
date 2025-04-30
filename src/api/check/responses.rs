//! Structures for `check` responses.

use std::{borrow::Cow, marker::PhantomData, ops::Deref};

#[cfg(feature = "annotate")]
use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use lifetime::IntoStatic;
use serde::{Deserialize, Serialize};

/// Detected language from check request.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DetectedLanguage {
    /// Language code, e.g., `"sk-SK"` for Slovak.
    pub code: String,
    /// Confidence level, from 0 to 1.
    #[cfg(feature = "unstable")]
    pub confidence: Option<f64>,
    /// Language name, e.g., `"Slovak"`.
    pub name: String,
    /// Source (file) for the language detection.
    #[cfg(feature = "unstable")]
    pub source: Option<String>,
}

/// Language information in check response.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LanguageResponse {
    /// Language code, e.g., `"sk-SK"` for Slovak.
    pub code: String,
    /// Detected language from provided request.
    pub detected_language: DetectedLanguage,
    /// Language name, e.g., `"Slovak"`.
    pub name: String,
}

/// Match context in check response.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Context {
    /// Length of the match.
    pub length: usize,
    /// Char index at which the match starts.
    pub offset: usize,
    /// Contextual text around the match.
    pub text: String,
}

/// More context, post-processed in check response.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct MoreContext {
    /// Line number where match occurred.
    pub line_number: usize,
    /// Char index at which the match starts on the current line.
    pub line_offset: usize,
}

/// Possible replacement for a given match in check response.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Replacement {
    /// Possible replacement value.
    pub value: String,
}

impl From<String> for Replacement {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl From<&str> for Replacement {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

/// A rule category.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Category {
    /// Category id.
    pub id: String,
    /// Category name.
    pub name: String,
}

/// A possible url of a rule in a check response.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Url {
    /// Url value.
    pub value: String,
}

/// The rule that was not satisfied in a given match.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Rule {
    /// Rule category.
    pub category: Category,
    /// Rule description.
    pub description: String,
    /// Rule id.
    pub id: String,
    /// Indicate if the rule is from the premium API.
    #[cfg(feature = "unstable")]
    pub is_premium: Option<bool>,
    /// Issue type.
    pub issue_type: String,
    /// Rule source file.
    #[cfg(feature = "unstable")]
    pub source_file: Option<String>,
    /// Rule sub id.
    pub sub_id: Option<String>,
    /// Rule list of urls.
    pub urls: Option<Vec<Url>>,
}

/// Type of given match.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Type {
    /// Type name.
    pub type_name: String,
}

/// Grammatical error match.
#[derive(PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Match {
    /// Match context.
    pub context: Context,
    /// Unknown: please fill a [PR](https://github.com/jeertmans/languagetool-rust/pulls) of your
    /// know that this attribute is used for.
    #[cfg(feature = "unstable")]
    pub context_for_sure_match: isize,
    /// Unknown: please fill a [PR](https://github.com/jeertmans/languagetool-rust/pulls) of your
    /// know that this attribute is used for.
    #[cfg(feature = "unstable")]
    pub ignore_for_incomplete_sentence: bool,
    /// Match length.
    pub length: usize,
    /// Error message.
    pub message: String,
    /// More context to match, post-processed using original text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_context: Option<MoreContext>,
    /// Char index at which the match starts.
    pub offset: usize,
    /// List of possible replacements (if applies).
    pub replacements: Vec<Replacement>,
    /// Match rule that was not satisfied.
    pub rule: Rule,
    /// Sentence in which the error was found.
    pub sentence: String,
    /// Short message about the error.
    pub short_message: String,
    /// Match type.
    #[cfg(feature = "unstable")]
    #[serde(rename = "type")]
    pub type_: Type,
}

/// LanguageTool software details.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Software {
    /// LanguageTool API version.
    pub api_version: usize,
    /// Some information about build date.
    pub build_date: String,
    /// Name (should be `"LanguageTool"`).
    pub name: String,
    /// Tell whether the server uses premium API or not.
    pub premium: bool,
    /// Sentence that indicates if using premium API would find more errors.
    #[cfg(feature = "unstable")]
    pub premium_hint: Option<String>,
    /// Unknown: please fill a [PR](https://github.com/jeertmans/languagetool-rust/pulls) of your
    /// know that this attribute is used for.
    pub status: String,
    /// LanguageTool version.
    pub version: String,
}

/// Warnings about check response.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Warnings {
    /// Indicate if results are incomplete.
    pub incomplete_results: bool,
}

/// LanguageTool POST check response.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Response {
    /// Language information.
    pub language: LanguageResponse,
    /// List of error matches.
    pub matches: Vec<Match>,
    /// Ranges ([start, end]) of sentences.
    #[cfg(feature = "unstable")]
    pub sentence_ranges: Option<Vec<[usize; 2]>>,
    /// LanguageTool software information.
    pub software: Software,
    /// Possible warnings.
    #[cfg(feature = "unstable")]
    pub warnings: Option<Warnings>,
}

impl Response {
    /// Return an iterator over matches.
    pub fn iter_matches(&self) -> std::slice::Iter<'_, Match> {
        self.matches.iter()
    }

    /// Return an iterator over mutable matches.
    pub fn iter_matches_mut(&mut self) -> std::slice::IterMut<'_, Match> {
        self.matches.iter_mut()
    }

    /// Creates an annotated string from current response.
    #[cfg(feature = "annotate")]
    #[must_use]
    pub fn annotate(&self, text: &str, origin: Option<&str>, color: bool) -> String {
        if self.matches.is_empty() {
            return "No errors were found in provided text".to_string();
        }
        let replacements: Vec<_> = self
            .matches
            .iter()
            .map(|m| {
                m.replacements.iter().fold(String::new(), |mut acc, r| {
                    if !acc.is_empty() {
                        acc.push_str(", ");
                    }
                    acc.push_str(&r.value);
                    acc
                })
            })
            .collect();

        let snippets = self.matches.iter().zip(replacements.iter()).map(|(m, r)| {
            Snippet {
                title: Some(Annotation {
                    label: Some(&m.message),
                    id: Some(&m.rule.id),
                    annotation_type: AnnotationType::Error,
                }),
                footer: vec![],
                slices: vec![Slice {
                    source: &m.context.text,
                    line_start: 1 + text.chars().take(m.offset).filter(|c| *c == '\n').count(),
                    origin,
                    fold: true,
                    annotations: vec![
                        SourceAnnotation {
                            label: &m.rule.description,
                            annotation_type: AnnotationType::Error,
                            range: (m.context.offset, m.context.offset + m.context.length),
                        },
                        SourceAnnotation {
                            label: r,
                            annotation_type: AnnotationType::Help,
                            range: (m.context.offset, m.context.offset + m.context.length),
                        },
                    ],
                }],
                opt: FormatOptions {
                    color,
                    ..Default::default()
                },
            }
        });

        let mut annotation = String::new();

        for snippet in snippets {
            if !annotation.is_empty() {
                annotation.push('\n');
            }
            annotation.push_str(&DisplayList::from(snippet).to_string());
        }
        annotation
    }

    /// Joins the given [`super::Request`] to the current one.
    ///
    /// This is especially useful when a request was split into multiple
    /// requests.
    #[must_use]
    pub fn append(mut self, mut other: Self) -> Self {
        #[cfg(feature = "unstable")]
        if let Some(ref mut sr_other) = other.sentence_ranges {
            match self.sentence_ranges {
                Some(ref mut sr_self) => {
                    sr_self.append(sr_other);
                },
                None => {
                    std::mem::swap(&mut self.sentence_ranges, &mut other.sentence_ranges);
                },
            }
        }

        self.matches.append(&mut other.matches);

        self
    }
}

/// Check response with additional context.
///
/// This structure exists to keep a link between a check response
/// and the original text that was checked.
#[derive(Debug, Clone, PartialEq, IntoStatic)]
pub struct ResponseWithContext<'source> {
    /// Original text that was checked by LT.
    pub text: Cow<'source, str>,
    /// Check response.
    pub response: Response,
    /// Text's length.
    pub text_length: usize,
}

impl Deref for ResponseWithContext<'_> {
    type Target = Response;
    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

impl<'source> ResponseWithContext<'source> {
    /// Bind a check response with its original text.
    #[must_use]
    pub fn new(text: Cow<'source, str>, response: Response) -> Self {
        let text_length = text.chars().count();

        // Add more context to response
        Self {
            text,
            response,
            text_length,
        }
    }

    /// Return an iterator over matches.
    pub fn iter_matches(&'source self) -> std::slice::Iter<'source, Match> {
        self.response.iter_matches()
    }

    /// Return an iterator over mutable matches.
    pub fn iter_matches_mut(&mut self) -> std::slice::IterMut<'_, Match> {
        self.response.iter_matches_mut()
    }

    /// Return an iterator over matches and corresponding line number and line
    /// offset.
    #[must_use]
    pub fn iter_match_positions(&self) -> MatchPositions<'_, '_, std::slice::Iter<'_, Match>> {
        self.into()
    }

    /// Append a check response to the current while
    /// adjusting the matches' offsets.
    ///
    /// This is especially useful when a text was split in multiple requests.
    #[must_use]
    pub fn append(mut self, mut other: Self) -> Self {
        let offset = self.text_length;
        for m in other.iter_matches_mut() {
            m.offset += offset;
        }

        #[cfg(feature = "unstable")]
        if let Some(ref mut sr_other) = other.response.sentence_ranges {
            match self.response.sentence_ranges {
                Some(ref mut sr_self) => {
                    sr_self.append(sr_other);
                },
                None => {
                    std::mem::swap(
                        &mut self.response.sentence_ranges,
                        &mut other.response.sentence_ranges,
                    );
                },
            }
        }

        self.response.matches.append(&mut other.response.matches);

        self.text.to_mut().push_str(&other.text);
        self.text_length += other.text_length;

        self
    }
}

impl<'source> From<ResponseWithContext<'source>> for Response {
    fn from(mut resp: ResponseWithContext<'source>) -> Self {
        for (line_number, line_offset, m) in MatchPositions::new(&resp.text, &mut resp.response) {
            m.more_context = Some(MoreContext {
                line_number,
                line_offset,
            });
        }

        resp.response
    }
}

/// Iterator over matches and their corresponding line number and line offset.
#[derive(Clone, Debug)]
pub struct MatchPositions<'source, 'response, T: Iterator + 'response> {
    text_chars: std::str::Chars<'source>,
    matches: T,
    line_number: usize,
    line_offset: usize,
    offset: usize,
    _marker: PhantomData<&'response ()>,
}

impl<'source, 'response> MatchPositions<'source, 'response, std::slice::IterMut<'response, Match>> {
    fn new(text: &'source str, response: &'response mut Response) -> Self {
        MatchPositions {
            _marker: Default::default(),
            text_chars: text.chars(),
            matches: response.iter_matches_mut(),
            line_number: 1,
            line_offset: 0,
            offset: 0,
        }
    }
}

impl<'source, 'response> From<&'source ResponseWithContext<'source>>
    for MatchPositions<'source, 'response, std::slice::Iter<'response, Match>>
where
    'source: 'response,
{
    fn from(response: &'source ResponseWithContext) -> Self {
        MatchPositions {
            _marker: Default::default(),
            text_chars: response.text.chars(),
            matches: response.iter_matches(),
            line_number: 1,
            line_offset: 0,
            offset: 0,
        }
    }
}

impl<'source, 'response> From<&'source mut ResponseWithContext<'source>>
    for MatchPositions<'source, 'response, std::slice::IterMut<'response, Match>>
where
    'source: 'response,
{
    fn from(response: &'source mut ResponseWithContext) -> Self {
        MatchPositions {
            _marker: Default::default(),
            text_chars: response.text.chars(),
            matches: response.response.iter_matches_mut(),
            line_number: 1,
            line_offset: 0,
            offset: 0,
        }
    }
}

impl<'response, T: Iterator + 'response> MatchPositions<'_, 'response, T> {
    /// Set the line number to a given value.
    ///
    /// By default, the first line number is 1.
    pub fn set_line_number(mut self, line_number: usize) -> Self {
        self.line_number = line_number;
        self
    }

    fn update_line_number_and_offset(&mut self, m: &Match) {
        let n = m.offset - self.offset;
        for _ in 0..n {
            match self.text_chars.next() {
                Some('\n') => {
                    self.line_number += 1;
                    self.line_offset = 0;
                },
                None => {
                    panic!(
                        "text is shorter than expected, are you sure this text was the one used \
                         for the check request?"
                    )
                },
                _ => self.line_offset += 1,
            }
        }
        self.offset = m.offset;
    }
}

impl<'source, 'response> Iterator
    for MatchPositions<'source, 'response, std::slice::Iter<'response, Match>>
where
    'response: 'source,
{
    type Item = (usize, usize, &'source Match);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(m) = self.matches.next() {
            self.update_line_number_and_offset(m);
            Some((self.line_number, self.line_offset, m))
        } else {
            None
        }
    }
}

impl<'source, 'response> Iterator
    for MatchPositions<'source, 'response, std::slice::IterMut<'response, Match>>
where
    'response: 'source,
{
    type Item = (usize, usize, &'source mut Match);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(m) = self.matches.next() {
            self.update_line_number_and_offset(m);
            Some((self.line_number, self.line_offset, m))
        } else {
            None
        }
    }
}
