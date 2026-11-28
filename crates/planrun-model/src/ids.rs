//! Identifiers: tasks, formats, recognizers, git blobs.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A task id as written (`P1-T3`, `P3c-1-T1`, `Task 4`), with the ordinal the format extracted
/// from it. Ordinals must strictly increase through a plan (check rule `task-order`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId {
    /// The id as written in the heading.
    pub raw: Box<str>,
    /// The number the format's task pattern captured as `ord`.
    pub ordinal: u32,
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

/// The id of a plan format (`assay`, `superpowers`, …): the `id` of its spec.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FormatId(pub Box<str>);

/// Where a format's spec came from (D95): built into planrun, the user's config directory, or
/// the repository's `.planrun/formats/`; the two files are paths as discovery found them.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FormatSource {
    /// `formats/*.toml`, embedded.
    BuiltIn,
    /// `<user config directory>/formats/<id>.toml`.
    User(PathBuf),
    /// `<repo>/.planrun/formats/<id>.toml`.
    Repository(PathBuf),
}

/// The id of the recognizer instance that produced an action (`file-block#full-contents`), for
/// `planrun show` and the coverage report.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecognizerId(pub Box<str>);

/// A git blob id as abbreviated in a diff's `index` line: 4 to 40 hex digits. Serialized as
/// its hex string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AbbrevBlobId {
    nibbles: [u8; 40],
    len: u8,
}

/// Why a blob id did not parse.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` is not a git blob id (4 to 40 hex digits)")]
pub struct BlobIdError(pub String);

impl AbbrevBlobId {
    /// Parses `hex` (case-insensitive).
    pub fn parse(hex: &str) -> Result<Self, BlobIdError> {
        if !(4..=40).contains(&hex.len()) {
            return Err(BlobIdError(hex.to_owned()));
        }
        let mut nibbles = [0u8; 40];
        for (slot, c) in nibbles.iter_mut().zip(hex.chars()) {
            *slot = c
                .to_digit(16)
                .and_then(|d| u8::try_from(d).ok())
                .ok_or_else(|| BlobIdError(hex.to_owned()))?;
        }
        Ok(Self {
            nibbles,
            len: u8::try_from(hex.len()).unwrap_or(40),
        })
    }

    /// The all-zero id git writes for a file that does not exist on that side of a diff.
    pub fn is_absent(&self) -> bool {
        self.nibbles[..usize::from(self.len)]
            .iter()
            .all(|&n| n == 0)
    }

    /// True when the full 20-byte `id` starts with this abbreviation.
    pub fn matches(&self, id: &[u8; 20]) -> bool {
        (0..usize::from(self.len)).all(|i| {
            let byte = id[i / 2];
            let nibble = if i % 2 == 0 { byte >> 4 } else { byte & 0x0f };
            nibble == self.nibbles[i]
        })
    }
}

impl TryFrom<String> for AbbrevBlobId {
    type Error = BlobIdError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<AbbrevBlobId> for String {
    fn from(value: AbbrevBlobId) -> Self {
        value.to_string()
    }
}

impl fmt::Display for AbbrevBlobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &n in &self.nibbles[..usize::from(self.len)] {
            write!(f, "{n:x}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_abbreviation_matches_its_full_id() {
        let id = AbbrevBlobId::parse("97bd92a").unwrap_or_else(|e| panic!("{e}"));
        let mut full = [0u8; 20];
        full[..4].copy_from_slice(&[0x97, 0xbd, 0x92, 0xa0]);
        assert!(id.matches(&full));
        full[3] = 0xb0;
        assert!(!id.matches(&full));
        assert_eq!(id.to_string(), "97bd92a");
    }

    #[test]
    fn zeros_mean_absent() {
        let id = AbbrevBlobId::parse("0000000").unwrap_or_else(|e| panic!("{e}"));
        assert!(id.is_absent());
        assert!(AbbrevBlobId::parse("xyz1").is_err());
    }
}
