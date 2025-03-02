//! Parse the contents of HTML files into a format parseable by the LanguageTool
//! API.

/// Parse the contents of an HTML file into a text format to be sent to the
/// LanguageTool API.
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
