//! Parse the contents of Typst files into a format parseable by the
//! LanguageTool API.

use crate::api::check::{Data, DataAnnotation};

/// Parse the contents of a Typst file into a text format to be sent to the
/// LanguageTool API.
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
                    // This pattern is ignored by LanguageTool, and allows us to avoid whitespace
                    // issues. The following sentence would give an error for
                    // repeated whitespace otherwise: This has ``` `backticks`
                    // ``` in it
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
