//! Decoding invoice XML to text and formatting it for display (spec 0008).

use std::borrow::Cow;

use crate::detect::DetectError;

const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

/// Decodes invoice XML according to its declared encoding: UTF-8 (also when
/// nothing is declared), ISO-8859-1 or US-ASCII. A UTF-8 byte order mark is
/// removed.
pub fn decode(xml: &[u8]) -> Result<Cow<'_, str>, DetectError> {
    let bytes = xml.strip_prefix(UTF8_BOM).unwrap_or(xml);
    let encoding = declared_encoding(bytes).map(|name| name.to_ascii_lowercase());
    match encoding.as_deref() {
        None | Some("utf-8" | "utf8") => std::str::from_utf8(bytes)
            .map(Cow::Borrowed)
            .map_err(|error| not_well_formed(bytes, error.valid_up_to())),
        Some("iso-8859-1" | "iso8859-1" | "iso_8859-1" | "latin1" | "latin-1" | "l1") => {
            // ISO-8859-1 maps every byte to the Unicode code point of the
            // same value.
            Ok(Cow::Owned(
                bytes.iter().map(|&byte| char::from(byte)).collect(),
            ))
        }
        Some("us-ascii" | "ascii") => match bytes.iter().position(|byte| !byte.is_ascii()) {
            Some(offset) => Err(not_well_formed(bytes, offset)),
            None => std::str::from_utf8(bytes)
                .map(Cow::Borrowed)
                .map_err(|error| not_well_formed(bytes, error.valid_up_to())),
        },
        Some(other) => Err(DetectError::UnsupportedEncoding {
            encoding: other.to_string(),
        }),
    }
}

/// The `encoding` of the XML declaration, if there is one.
fn declared_encoding(bytes: &[u8]) -> Option<&str> {
    let rest = bytes.strip_prefix(b"<?xml")?;
    let end = rest.windows(2).position(|pair| pair == b"?>")?;
    let declaration = std::str::from_utf8(&rest[..end]).ok()?;
    let after_name = &declaration[declaration.find("encoding")? + "encoding".len()..];
    let after_equals = after_name.trim_start().strip_prefix('=')?.trim_start();
    let quote = after_equals
        .chars()
        .next()
        .filter(|c| *c == '"' || *c == '\'')?;
    let value = &after_equals[1..];
    Some(&value[..value.find(quote)?])
}

fn not_well_formed(bytes: &[u8], offset: usize) -> DetectError {
    let newlines = bytes[..offset]
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count();
    DetectError::NotWellFormed {
        line: u32::try_from(newlines + 1).unwrap_or(u32::MAX),
    }
}

/// Formats XML text for display: one element per line, indented by two
/// spaces per level; elements with only text stay on one line. Tags,
/// attributes, comments and CDATA sections are kept exactly as written.
pub fn pretty_print(xml: &str) -> String {
    pretty_print_mapped(xml).0
}

/// A piece of the input copied verbatim into the reformatted text.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Piece {
    /// Byte offset in the input.
    pub input: usize,
    /// Byte offset in the reformatted text.
    pub output: usize,
    pub len: usize,
}

/// `pretty_print`, plus where each piece of the input ended up, in input
/// order.
pub(crate) fn pretty_print_mapped(xml: &str) -> (String, Vec<Piece>) {
    let tokens = tokenize(xml);
    let mut pieces = Vec::with_capacity(tokens.len());
    let mut out = String::with_capacity(xml.len() * 2);
    let mut push_line = |out: &mut String, depth: usize, parts: &[&str]| {
        if !out.is_empty() {
            out.push('\n');
        }
        for _ in 0..depth {
            out.push_str("  ");
        }
        for part in parts {
            pieces.push(Piece {
                // Every part is a slice of `xml`.
                input: part.as_ptr() as usize - xml.as_ptr() as usize,
                output: out.len(),
                len: part.len(),
            });
            out.push_str(part);
        }
    };
    let mut depth = 0usize;
    let mut index = 0;
    while index < tokens.len() {
        match tokens[index] {
            Token::Start(start) => match (tokens.get(index + 1), tokens.get(index + 2)) {
                (Some(Token::Text(text)), Some(Token::End(end))) => {
                    push_line(&mut out, depth, &[start, text, end]);
                    index += 3;
                }
                (Some(Token::End(end)), _) => {
                    push_line(&mut out, depth, &[start, end]);
                    index += 2;
                }
                _ => {
                    push_line(&mut out, depth, &[start]);
                    depth += 1;
                    index += 1;
                }
            },
            Token::End(end) => {
                depth = depth.saturating_sub(1);
                push_line(&mut out, depth, &[end]);
                index += 1;
            }
            Token::Empty(text) | Token::Text(text) | Token::Other(text) => {
                push_line(&mut out, depth, &[text]);
                index += 1;
            }
        }
    }
    (out, pieces)
}

/// A piece of XML text, borrowed from the input.
#[derive(Debug, Clone, Copy)]
enum Token<'a> {
    Start(&'a str),
    End(&'a str),
    Empty(&'a str),
    /// Character data (trimmed) or a CDATA section.
    Text(&'a str),
    /// XML declaration, processing instruction, comment or DOCTYPE.
    Other(&'a str),
}

fn tokenize(xml: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut position = 0;
    while position < xml.len() {
        let rest = &xml[position..];
        if !rest.starts_with('<') {
            let end = rest.find('<').unwrap_or(rest.len());
            let text = rest[..end].trim();
            if !text.is_empty() {
                tokens.push(Token::Text(text));
            }
            position += end;
            continue;
        }
        let (length, token) = if let Some(length) = delimited(rest, "<!--", "-->") {
            (length, Token::Other(&rest[..length]))
        } else if let Some(length) = delimited(rest, "<![CDATA[", "]]>") {
            (length, Token::Text(&rest[..length]))
        } else if let Some(length) = delimited(rest, "<?", "?>") {
            (length, Token::Other(&rest[..length]))
        } else if rest.starts_with("<!") {
            let length = rest.find('>').map_or(rest.len(), |end| end + 1);
            (length, Token::Other(&rest[..length]))
        } else {
            let length = tag_length(rest);
            let tag = &rest[..length];
            let token = if tag.starts_with("</") {
                Token::End(tag)
            } else if tag.ends_with("/>") {
                Token::Empty(tag)
            } else {
                Token::Start(tag)
            };
            (length, token)
        };
        tokens.push(token);
        position += length;
    }
    tokens
}

/// Length of `text` up to and including `close`, if it starts with `open`.
/// Unterminated constructs extend to the end of the text.
fn delimited(text: &str, open: &str, close: &str) -> Option<usize> {
    let body = text.strip_prefix(open)?;
    Some(
        body.find(close)
            .map_or(text.len(), |end| open.len() + end + close.len()),
    )
}

/// Length of the tag at the start of `text`, honouring quoted attribute
/// values that may contain `>`.
fn tag_length(text: &str) -> usize {
    let mut quote = None;
    for (index, c) in text.char_indices() {
        match (quote, c) {
            (None, '"' | '\'') => quote = Some(c),
            (Some(open), _) if c == open => quote = None,
            (None, '>') => return index + 1,
            _ => {}
        }
    }
    text.len()
}
