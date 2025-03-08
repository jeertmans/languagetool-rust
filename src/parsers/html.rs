//! Parse the contents of HTML files into a format parseable by the LanguageTool
//! API.

use ego_tree::NodeRef;
use scraper::{Html, Node};

use crate::{
    api::check::{Data, DataAnnotation},
    parsers::IGNORE,
};

/// Parse the contents of an HTML file into a text format to be sent to the
/// LanguageTool API.
#[must_use]
pub fn parse_html(file_content: &str) -> Data<'static> {
    let mut annotations: Vec<DataAnnotation> = vec![];

    fn handle_node(annotations: &mut Vec<DataAnnotation>, node: NodeRef<'_, Node>) {
        let n = node.value();
        match n {
            Node::Element(el) => {
                match el.name() {
                    "head" | "script" | "style" => {},

                    "code" => {
                        annotations.push(DataAnnotation::new_interpreted_markup(
                            "<code>...</code>",
                            IGNORE,
                        ));
                    },

                    "img" => {
                        annotations.push(DataAnnotation::new_interpreted_markup("<img />", IGNORE));
                    },

                    s => {
                        match s {
                            "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" | "td" | "th"
                            | "div" => {
                                annotations.push(DataAnnotation::new_interpreted_markup(
                                    format!("<{s}>"),
                                    "\n\n",
                                ));
                                for node in node.children() {
                                    handle_node(annotations, node);
                                }
                                annotations.push(DataAnnotation::new_interpreted_markup(
                                    format!("</{s}>"),
                                    "\n\n",
                                ));
                            },
                            _ => {
                                annotations.push(DataAnnotation::new_markup(format!("<{s}>")));
                                for node in node.children() {
                                    handle_node(annotations, node);
                                }
                                annotations.push(DataAnnotation::new_markup(format!("</{s}>")));
                            },
                        }
                    },
                }
            },

            Node::Text(t) => {
                let mut text = t.trim().to_owned();
                if !text.is_empty() {
                    let mut chars = t.chars();

                    // Maintain leading/trailing white space, but only a single space
                    if chars.next().is_some_and(|c| c.is_whitespace()) {
                        while text.chars().last().is_some_and(|c| c.is_whitespace()) {
                            text.pop();
                        }
                        text.insert(0, ' ');
                    }
                    if chars.last().is_some_and(|c| c.is_whitespace()) {
                        text.push(' ');
                    }

                    annotations.push(DataAnnotation::new_text(text))
                } else {
                    annotations.push(DataAnnotation::new_text("\n\n"));
                }
            },

            Node::Comment(c) => {
                let comment = c.to_string();

                annotations.push(DataAnnotation::new_interpreted_markup(
                    format!("<!-- {comment} -->",),
                    format!("\n\n{comment}\n\n"),
                ));
            },

            _ => {},
        }
    }

    let document = Html::parse_document(file_content);
    for node in document.root_element().children() {
        handle_node(&mut annotations, node);
    }

    Data::from_iter(annotations)
}
