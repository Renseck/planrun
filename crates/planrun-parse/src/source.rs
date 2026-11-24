//! Reading a plan: UTF-8 check, BOM, line-ending normalization, fingerprint (decision D6).

use planrun_model::PlanSource;
use sha2::{Digest, Sha256};

use crate::diagnostic::Diagnostic;

/// The largest plan accepted, in bytes. Spans are `u32`; this keeps every offset far below that.
pub const MAX_PLAN_BYTES: usize = 64 * 1024 * 1024;

/// Reads `bytes` as the plan named `name`: refuses input over [`MAX_PLAN_BYTES`] (`too-large`)
/// and input that is not UTF-8 (`not-utf8`), strips a leading BOM, turns CRLF and lone CR into
/// LF, and fingerprints the bytes as read (hex SHA-256).
pub fn load(name: &str, bytes: &[u8]) -> Result<PlanSource, Diagnostic> {
    if bytes.len() > MAX_PLAN_BYTES {
        return Err(Diagnostic::error(
            "too-large",
            None,
            format!(
                "the plan is {} bytes; the limit is {MAX_PLAN_BYTES}",
                bytes.len()
            ),
        ));
    }
    let text = std::str::from_utf8(bytes).map_err(|e| {
        Diagnostic::error(
            "not-utf8",
            None,
            format!("byte {} is not valid UTF-8", e.valid_up_to()),
        )
    })?;
    Ok(PlanSource::from_normalized(
        name,
        normalize(text),
        hex(&Sha256::digest(bytes)),
    ))
}

/// The text without a leading BOM, with CRLF and lone CR as LF. Nothing else changes.
pub fn normalize(text: &str) -> String {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    if !text.contains('\r') {
        return text.to_owned();
    }
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(char::from(DIGITS[usize::from(b >> 4)]));
        out.push(char::from(DIGITS[usize::from(b & 0x0f)]));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_endings_and_bom_are_normalized() {
        assert_eq!(normalize("\u{feff}a\r\nb\rc\n"), "a\nb\nc\n");
        assert_eq!(normalize("a\n\r\n"), "a\n\n");
        assert_eq!(normalize("plain\n"), "plain\n");
    }

    #[test]
    fn the_fingerprint_is_of_the_bytes_as_read() {
        let crlf = load("p", b"a\r\n").unwrap_or_else(|e| panic!("{e:?}"));
        let lf = load("p", b"a\n").unwrap_or_else(|e| panic!("{e:?}"));
        assert_eq!(crlf.text(), lf.text());
        assert_ne!(crlf.fingerprint, lf.fingerprint);
        assert_eq!(
            &*lf.fingerprint,
            "87428fc522803d31065e7bce3cf03fe475096631e5e07bbd7a0fde60c4cf25c7"
        );
    }

    #[test]
    fn invalid_utf8_is_refused() {
        let err = load("p", b"ok\xff").err().map(|d| (d.code, d.message));
        assert_eq!(
            err,
            Some(("not-utf8", "byte 2 is not valid UTF-8".to_owned()))
        );
    }
}
