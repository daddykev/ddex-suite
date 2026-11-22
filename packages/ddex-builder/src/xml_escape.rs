//! XML escaping and character checks shared by the XML writer and the canonicalizer.
//!
//! Output must be well-formed for any input string and must read back as exactly the
//! input text:
//! - text: `&`, `<`, `>` are escaped, and `\r` is written as `&#13;` (a parser would
//!   otherwise normalize it to `\n`);
//! - attribute values: additionally `"` and `'`, and `\t`, `\n`, `\r` as character
//!   references (attribute-value normalization would otherwise turn them into spaces).
//!
//! Characters that XML 1.0 does not allow at all (C0 controls other than tab, LF and CR,
//! U+FFFE and U+FFFF; unpaired surrogates cannot occur in a Rust `String`) cannot be
//! escaped either, so the builder rejects them up front ([`check_request_chars`]).

use crate::error::BuildError;

/// Escape character data.
pub(crate) fn escape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\r' => out.push_str("&#13;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escape an attribute value (for either quote style).
pub(crate) fn escape_attribute(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '\t' => out.push_str("&#9;"),
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            _ => out.push(c),
        }
    }
    out
}

/// True for characters XML 1.0 does not allow anywhere in a document.
pub(crate) fn is_illegal_xml_char(c: char) -> bool {
    matches!(c, '\u{0}'..='\u{8}' | '\u{B}' | '\u{C}' | '\u{E}'..='\u{1F}' | '\u{FFFE}' | '\u{FFFF}')
}

/// Reject a build request that contains a character XML 1.0 does not allow, naming the
/// field (e.g. `releases[0].tracks[2].title`) and the character.
pub(crate) fn check_request_chars(
    request: &crate::builder::BuildRequest,
) -> Result<(), BuildError> {
    let value =
        serde_json::to_value(request).map_err(|e| BuildError::Serialization(e.to_string()))?;
    check_value(&value, &mut String::new())
}

fn check_value(value: &serde_json::Value, path: &mut String) -> Result<(), BuildError> {
    use serde_json::Value;
    match value {
        Value::String(s) => check_str(s, path),
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                let len = path.len();
                path.push_str(&format!("[{i}]"));
                check_value(item, path)?;
                path.truncate(len);
            }
            Ok(())
        }
        Value::Object(map) => {
            for (key, item) in map {
                let len = path.len();
                if !path.is_empty() {
                    path.push('.');
                }
                path.push_str(key);
                check_str(key, path)?;
                check_value(item, path)?;
                path.truncate(len);
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn check_str(s: &str, path: &str) -> Result<(), BuildError> {
    match s.chars().find(|c| is_illegal_xml_char(*c)) {
        Some(c) => Err(BuildError::InvalidFormat {
            field: path.to_string(),
            message: format!(
                "contains U+{:04X}, which XML 1.0 does not allow (control characters other \
                 than tab, LF and CR, U+FFFE and U+FFFF); remove it from the input",
                c as u32
            ),
        }),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_escaping() {
        assert_eq!(
            escape_text("a & <b> \"q\" 'a'\r\n\t"),
            "a &amp; &lt;b&gt; \"q\" 'a'&#13;\n\t"
        );
    }

    #[test]
    fn attribute_escaping() {
        assert_eq!(
            escape_attribute("a & <b> \"q\" 'a'\r\n\t"),
            "a &amp; &lt;b&gt; &quot;q&quot; &apos;a&apos;&#13;&#10;&#9;"
        );
    }

    #[test]
    fn illegal_characters() {
        for c in [
            '\u{0}', '\u{1}', '\u{8}', '\u{B}', '\u{C}', '\u{E}', '\u{1F}', '\u{FFFE}', '\u{FFFF}',
        ] {
            assert!(is_illegal_xml_char(c), "{:?}", c);
        }
        for c in [
            '\t',
            '\n',
            '\r',
            ' ',
            '\u{7F}',
            '\u{85}',
            '\u{FFFD}',
            '\u{10FFFF}',
            '🎵',
        ] {
            assert!(!is_illegal_xml_char(c), "{:?}", c);
        }
    }
}
