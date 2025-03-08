//! Parse the contents of Markdown files into a format parseable by the
//! LanguageTool API.

use crate::{
    api::check::{Data, DataAnnotation},
    parsers::IGNORE,
};

/// Parse the contents of a Markdown file into a text format to be sent to the
/// LanguageTool API.
#[must_use]
pub fn parse_markdown(file_content: &str) -> Data<'_> {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let mut annotations: Vec<DataAnnotation> = vec![];

    // Stack to keep track of the current "tag" context
    let mut tags = vec![];

    Parser::new_ext(file_content, Options::all()).for_each(|event| {
        match event {
            Event::Start(tag) => {
                match tag {
                    // Start list items
                    Tag::List(_) | Tag::Item => {
                        annotations.push(DataAnnotation::new_text("- "));
                    },
                    _ => {},
                }

                tags.push(tag);
            },
            Event::End(tag) => {
                match tag {
                    // Separate list items and table cells
                    TagEnd::List(_) | TagEnd::Item | TagEnd::TableRow | TagEnd::TableHead => {
                        annotations.push(DataAnnotation::new_text("\n"));
                    },
                    TagEnd::TableCell => {
                        annotations.push(DataAnnotation::new_text(" | "));
                    },
                    _ => {},
                };

                if tags
                    .last()
                    .is_some_and(|t| TagEnd::from(t.to_owned()) == tag)
                {
                    tags.pop();
                };
            },

            Event::Html(s) | Event::InlineHtml(s) => {
                let data = super::html::parse_html(s.into_string());
                annotations.push(DataAnnotation::new_text(data));
            },

            Event::Text(mut s) => {
                // Add space between sentences
                if s.chars()
                    .last()
                    .is_some_and(|c| matches!(c, '.' | '!' | '?'))
                {
                    s = pulldown_cmark::CowStr::from(s.to_string() + " ");
                }

                let Some(tag) = tags.last() else {
                    annotations.push(DataAnnotation::new_text(s.to_owned()));
                    return;
                };

                match tag {
                    Tag::Heading { level, .. } => {
                        let s = format!("{s}\n");
                        annotations.push(DataAnnotation::new_text(format!(
                            "{} {s}\n",
                            "#".repeat(*level as usize)
                        )));
                    },

                    Tag::Emphasis => {
                        annotations
                            .push(DataAnnotation::new_interpreted_markup(format!("_{s}_"), s))
                    },
                    Tag::Strong => {
                        annotations.push(DataAnnotation::new_interpreted_markup(
                            format!("**{s}**"),
                            s,
                        ))
                    },
                    Tag::Strikethrough => {
                        annotations
                            .push(DataAnnotation::new_interpreted_markup(format!("~{s}~"), s))
                    },

                    // No changes necessary
                    Tag::Paragraph
                    | Tag::List(_)
                    | Tag::Item
                    | Tag::BlockQuote
                    | Tag::TableCell => {
                        annotations.push(DataAnnotation::new_text(s));
                    },

                    // Ignored
                    Tag::CodeBlock(_) | Tag::Link { .. } | Tag::Image { .. } => {
                        annotations.push(DataAnnotation::new_interpreted_markup(s, IGNORE));
                    },
                    _ => {},
                }
            },
            Event::Code(s) => {
                annotations.push(DataAnnotation::new_interpreted_markup(s, IGNORE));
            },

            Event::HardBreak => {
                annotations.push(DataAnnotation::new_text("\n\n"));
            },
            Event::SoftBreak => {
                if let Some(last) = annotations.last() {
                    // Don't add space if the last text already ends with a space
                    if last
                        .text
                        .as_ref()
                        .is_some_and(|t| t.chars().last().is_some_and(|c| c.is_ascii_whitespace()))
                        || last.interpret_as.as_ref().is_some_and(|t| {
                            t.chars().last().is_some_and(|c| c.is_ascii_whitespace())
                        })
                    {
                        return;
                    };
                }

                annotations.push(DataAnnotation::new_text(" "));
            },

            Event::FootnoteReference(_) | Event::TaskListMarker(_) | Event::Rule => {},
        };
    });

    Data::from_iter(annotations)
}
