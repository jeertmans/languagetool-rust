//! Utilities for parsing the contents of different file types into a text
//! representation that can be sent to the LanguageTool API.

use crate::api::check::{Data, DataAnnotation};

/// Parse the contents of an HTML file into a text format to be sent to the
/// LanguageTool API.
#[cfg(feature = "html")]
pub fn parse_html(file_content: impl AsRef<str>) -> String {
    use html_parser::Node;

    let mut txt = String::new();

    let html = html_parser::Dom::parse(file_content.as_ref()).unwrap_or_default();
    let mut children: Vec<Node> = html.children.into_iter().rev().collect();

    fn handle_node(txt: &mut String, node: Node) {
        if let Some(e) = node.element() {
            match e.name.as_str() {
                "head" | "script" | "style" => {
                    return;
                },
                "code" => {
                    txt.push_str("_code_");
                    return;
                },
                "a" => {
                    txt.push_str("_link_");
                    return;
                },
                "pre" => {
                    txt.push_str("_pre_");
                    txt.push_str("\n\n");
                    return;
                },
                s => {
                    let add_children = |txt: &mut String| {
                        if !e.children.is_empty() {
                            // Recursively handle children
                            e.children.clone().into_iter().for_each(|n| {
                                handle_node(txt, n);
                            });
                        };
                    };

                    match s {
                        "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" | "td" | "th" => {
                            while txt.chars().last().is_some_and(|c| c.is_whitespace()) {
                                txt.pop();
                            }
                            txt.push_str("\n\n");
                            add_children(txt);
                            txt.push_str("\n\n");
                        },
                        _ => {
                            add_children(txt);
                        },
                    }
                },
            }
        }

        if let Some(t) = node.text() {
            let mut text = t.trim().to_owned();
            if !text.is_empty() {
                let mut chars = t.chars();

                // Maintain leading/trailing white space, but only a single space
                if chars.next().is_some_and(|c| c.is_whitespace()) {
                    while txt.chars().last().is_some_and(|c| c.is_whitespace()) {
                        txt.pop();
                    }
                    text.insert(0, ' ');
                }
                if chars.last().is_some_and(|c| c.is_whitespace()) {
                    text.push(' ');
                }

                txt.push_str(&text);
            }
        }
    }

    while let Some(node) = children.pop() {
        handle_node(&mut txt, node);
    }

    txt
}

/// Parse the contents of a Markdown file into a text format to be sent to the
/// LanguageTool API.
#[cfg(feature = "markdown")]
pub fn parse_markdown(file_content: impl AsRef<str>) -> String {
    use pulldown_cmark::{html, Options, Parser};

    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(file_content.as_ref(), options);
    let mut html = String::new();
    html::push_html(&mut html, parser);

    parse_html(html)
}

/// Parse the contents of a Typst file into a text format to be sent to the
/// LanguageTool API.
#[cfg(feature = "typst")]
pub fn parse_typst(file_content: impl AsRef<str>) -> Data<'static> {
    use typst_syntax::{parse, SyntaxKind, SyntaxNode};

    let mut annotations: Vec<DataAnnotation> = vec![];

    let parent = parse(file_content.as_ref());
    let mut nodes: Vec<&SyntaxNode> = parent.children().rev().collect();

    while let Some(node) = nodes.pop() {
        let kind = node.kind();

        // MARKUP NODES
        match kind {
            // Pure markup
            SyntaxKind::SetRule
            | SyntaxKind::Ident
            | SyntaxKind::ShowRule
            | SyntaxKind::Raw
            | SyntaxKind::Code
            | SyntaxKind::CodeBlock
            | SyntaxKind::Math
            | SyntaxKind::Equation
            | SyntaxKind::Ref
            | SyntaxKind::LetBinding
            | SyntaxKind::FieldAccess
            | SyntaxKind::FuncCall
            | SyntaxKind::Args => {
                let mut markup = node.text().to_string();
                if markup.is_empty() {
                    let mut stack: Vec<&SyntaxNode> = node.children().rev().collect();
                    while let Some(n) = stack.pop() {
                        if n.text().is_empty() {
                            stack.extend(n.children().rev());
                        } else {
                            markup += n.text();
                        }
                    }
                }

                annotations.push(DataAnnotation::new_interpreted_markup(
                    markup,
                    // This pattern is ignored by LanguageTool, and allows us to avoid whitespace issues.
                    // The following sentence would give an error for repeated whitespace
                    // otherwise: This has ``` `backticks` ``` in it
                    "_ignore_".to_string(),
                ));
                continue;
            },
            // Markup with valid text interpretations
            SyntaxKind::Heading
            | SyntaxKind::Markup
            | SyntaxKind::EnumItem
            | SyntaxKind::ListItem
            | SyntaxKind::Emph
            | SyntaxKind::Strong => {
                let (mut full_text, mut interpreted_as) = (String::new(), String::new());
                let mut stack: Vec<&SyntaxNode> = node.children().rev().collect();

                while let Some(n) = stack.pop() {
                    if n.text().is_empty() {
                        stack.extend(n.children().rev());
                    } else {
                        if matches!(n.kind(), SyntaxKind::Text | SyntaxKind::Space) {
                            interpreted_as += n.text();
                        }
                        full_text += n.text();
                    }
                }

                annotations.push(DataAnnotation::new_interpreted_markup(
                    full_text,
                    interpreted_as,
                ));
                continue;
            },
            _ => {},
        }

        // NESTED NODES
        if node.children().count() > 0 && !matches!(kind, SyntaxKind::Args | SyntaxKind::FuncCall) {
            nodes.extend(node.children().rev());
            continue;
        }

        // TEXT
        if matches!(
            kind,
            SyntaxKind::Text
                | SyntaxKind::SmartQuote
                | SyntaxKind::BlockComment
                | SyntaxKind::LineComment
                | SyntaxKind::Space
                | SyntaxKind::Parbreak
        ) {
            annotations.push(DataAnnotation::new_text(node.text().to_string()));
        };
    }

    Data::from_iter(annotations)
}
