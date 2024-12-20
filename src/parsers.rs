//! Utilities for parsing the contents of different file types into a text representation that can
//! be sent to the LanguageTool API.

/// Parse the contents of an HTML file into a text format to be sent to the LanguageTool API.
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

/// Parse the contents of a Markdown file into a text format to be sent to the LanguageTool API.
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

/// Parse the contents of a Typst file into a text format to be sent to the LanguageTool API.
#[cfg(feature = "typst")]
pub fn parse_typst(file_content: impl AsRef<str>) -> String {
    use typst_syntax::{parse, SyntaxKind, SyntaxNode};

    let mut txt = String::new();
    let parent = parse(file_content.as_ref());
    let mut nodes: Vec<&SyntaxNode> = parent.children().rev().collect();

    while let Some(node) = nodes.pop() {
        let kind = node.kind();

        // PLACEHOLDERS / REPLACEMENTS
        match kind {
            SyntaxKind::Code | SyntaxKind::CodeBlock | SyntaxKind::Raw => {
                txt.push_str("_raw_");
                continue;
            },
            SyntaxKind::Math => {
                txt.push_str("_math_");
                continue;
            },
            SyntaxKind::Ref => {
                txt.push_str("_ref_");
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
        match kind {
            SyntaxKind::Text
            | SyntaxKind::SmartQuote
            | SyntaxKind::BlockComment
            | SyntaxKind::LineComment => {
                let text = node.text();
                // if
                txt.push_str(text);
            },
            SyntaxKind::Space => {
                if let Some(c) = txt.chars().last() {
                    if !c.is_whitespace() {
                        txt.push(' ');
                    }
                }
            },
            SyntaxKind::Parbreak | SyntaxKind::EnumMarker | SyntaxKind::ListMarker => {
                // Clear any preceding white space, for a clean paragraph break
                while txt.chars().last().is_some_and(|c| c.is_whitespace()) {
                    txt.pop();
                }
                txt.push_str("\n\n");
            },
            _ => {},
        }
    }

    txt.trim().to_owned()
}
