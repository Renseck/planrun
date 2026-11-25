//! Line endings of written files (decisions D10, D54). Pure: the bytes, the path's attributes and
//! git's checkout convention for the path go in, the bytes to write come out.

use crate::git::Attributes;

/// A line ending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eol {
    /// `\n`.
    Lf,
    /// `\r\n`.
    Crlf,
}

/// The line endings a file has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endings {
    /// No line ending at all (empty, or one line).
    None,
    /// Only LF.
    Lf,
    /// Only CRLF.
    Crlf,
    /// Both.
    Mixed,
}

/// How a target's new contents are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Convention {
    /// `-text` or `binary`: the bytes as they are, matched as they are.
    Bytes,
    /// Text, with this line ending.
    Text(Eol),
}

/// A ReplaceExact on a file with both LF and CRLF (`eol-mixed`, D54).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mixed;

/// The line endings of `bytes`.
pub fn endings(bytes: &[u8]) -> Endings {
    let mut lf = false;
    let mut crlf = false;
    for (i, &b) in bytes.iter().enumerate() {
        if b != b'\n' {
            continue;
        }
        if i > 0 && bytes[i - 1] == b'\r' {
            crlf = true;
        } else {
            lf = true;
        }
    }
    match (lf, crlf) {
        (false, false) => Endings::None,
        (true, false) => Endings::Lf,
        (false, true) => Endings::Crlf,
        (true, true) => Endings::Mixed,
    }
}

/// `bytes` with every CRLF turned into LF (a lone CR stays).
pub fn to_lf(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    for (i, &b) in bytes.iter().enumerate() {
        let crlf = b == b'\r' && bytes.get(i + 1) == Some(&b'\n');
        if !crlf {
            out.push(b);
        }
    }
    out
}

/// LF text `lf` with the line ending `eol`.
pub fn with_eol(lf: &[u8], eol: Eol) -> Vec<u8> {
    if eol == Eol::Lf {
        return lf.to_vec();
    }
    let mut out = Vec::with_capacity(lf.len() + lf.len() / 32);
    for &b in lf {
        if b == b'\n' {
            out.push(b'\r');
        }
        out.push(b);
    }
    out
}

/// The attribute's decision, when it makes one: `-text` → bytes, `eol` → that ending.
fn by_attributes(attrs: &Attributes) -> Option<Convention> {
    if !attrs.text {
        return Some(Convention::Bytes);
    }
    match attrs.eol.as_deref() {
        Some("crlf") => Some(Convention::Text(Eol::Crlf)),
        Some("lf") => Some(Convention::Text(Eol::Lf)),
        _ => None,
    }
}

/// The convention of an Overwrite of a file whose current contents are `existing`.
pub fn for_overwrite(attrs: &Attributes, existing: &[u8], checkout: Eol) -> Convention {
    if let Some(decided) = by_attributes(attrs) {
        return decided;
    }
    match endings(existing) {
        Endings::Lf => Convention::Text(Eol::Lf),
        Endings::Crlf => Convention::Text(Eol::Crlf),
        Endings::None | Endings::Mixed => Convention::Text(checkout),
    }
}

/// The convention of a ReplaceExact in `existing`; a mixed file is refused.
pub fn for_replace(
    attrs: &Attributes,
    existing: &[u8],
    checkout: Eol,
) -> Result<Convention, Mixed> {
    if let Some(decided) = by_attributes(attrs) {
        return Ok(decided);
    }
    match endings(existing) {
        Endings::Mixed => Err(Mixed),
        Endings::Lf => Ok(Convention::Text(Eol::Lf)),
        Endings::Crlf => Ok(Convention::Text(Eol::Crlf)),
        Endings::None => Ok(Convention::Text(checkout)),
    }
}

/// The plan's LF text `body` as written under `convention`.
pub fn render(body: &[u8], convention: Convention) -> Vec<u8> {
    match convention {
        Convention::Bytes => body.to_vec(),
        Convention::Text(eol) => with_eol(body, eol),
    }
}

/// The text a ReplaceExact searches in: LF space for text (D10), the bytes for `-text`.
pub fn search_space(existing: &[u8], convention: Convention) -> Vec<u8> {
    match convention {
        Convention::Bytes => existing.to_vec(),
        Convention::Text(_) => to_lf(existing),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(eol: Option<&str>, text: bool) -> Attributes {
        Attributes {
            eol: eol.map(str::to_owned),
            text,
        }
    }

    #[test]
    fn endings_are_told_apart() {
        assert_eq!(endings(b""), Endings::None);
        assert_eq!(endings(b"one line"), Endings::None);
        assert_eq!(endings(b"a\nb\n"), Endings::Lf);
        assert_eq!(endings(b"a\r\nb\r\n"), Endings::Crlf);
        assert_eq!(endings(b"a\r\nb\n"), Endings::Mixed);
        assert_eq!(endings(b"\n"), Endings::Lf);
    }

    #[test]
    fn conversions_round_trip() {
        assert_eq!(to_lf(b"a\r\nb\rc\r\n"), b"a\nb\rc\n");
        assert_eq!(with_eol(b"a\nb\n", Eol::Crlf), b"a\r\nb\r\n");
        assert_eq!(with_eol(b"a\nb\n", Eol::Lf), b"a\nb\n");
    }

    #[test]
    fn attributes_win_and_binary_keeps_bytes() {
        let crlf_file = b"a\r\nb\r\n";
        assert_eq!(
            for_overwrite(&attrs(Some("lf"), true), crlf_file, Eol::Crlf),
            Convention::Text(Eol::Lf)
        );
        assert_eq!(
            for_overwrite(&attrs(Some("crlf"), true), b"a\n", Eol::Lf),
            Convention::Text(Eol::Crlf)
        );
        assert_eq!(
            for_replace(&attrs(Some("crlf"), false), b"a\r\nb\n", Eol::Lf),
            Ok(Convention::Bytes)
        );
    }

    #[test]
    fn the_file_decides_without_attributes() {
        let plain = attrs(None, true);
        assert_eq!(
            for_overwrite(&plain, b"a\r\n", Eol::Lf),
            Convention::Text(Eol::Crlf)
        );
        assert_eq!(
            for_overwrite(&plain, b"a\n", Eol::Crlf),
            Convention::Text(Eol::Lf)
        );
        assert_eq!(
            for_replace(&plain, b"a\r\n", Eol::Lf),
            Ok(Convention::Text(Eol::Crlf))
        );
    }

    #[test]
    fn no_line_endings_take_the_checkout_convention() {
        let plain = attrs(None, true);
        for checkout in [Eol::Lf, Eol::Crlf] {
            assert_eq!(
                for_overwrite(&plain, b"", checkout),
                Convention::Text(checkout)
            );
            assert_eq!(
                for_replace(&plain, b"one", checkout),
                Ok(Convention::Text(checkout))
            );
        }
    }

    #[test]
    fn a_mixed_file_is_overwritten_by_checkout_and_never_edited() {
        let plain = attrs(None, true);
        let mixed = b"a\r\nb\n";
        assert_eq!(
            for_overwrite(&plain, mixed, Eol::Crlf),
            Convention::Text(Eol::Crlf)
        );
        assert_eq!(for_replace(&plain, mixed, Eol::Lf), Err(Mixed));
    }

    #[test]
    fn search_space_is_lf_for_text_and_raw_for_bytes() {
        let file = b"a\r\nb\r\n";
        assert_eq!(search_space(file, Convention::Text(Eol::Crlf)), b"a\nb\n");
        assert_eq!(search_space(file, Convention::Bytes), file);
        assert_eq!(render(b"x\n", Convention::Text(Eol::Crlf)), b"x\r\n");
        assert_eq!(render(b"x\n", Convention::Bytes), b"x\n");
    }
}
