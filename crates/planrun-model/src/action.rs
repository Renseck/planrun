//! Actions: the closed set of things a plan can make planrun do (decision D4).

use serde::{Deserialize, Serialize};

use crate::expect::Expectation;
use crate::ids::{AbbrevBlobId, RecognizerId};
use crate::path::{PathOrigin, RepoPath};
use crate::span::{Span, TextRef};

/// One action of a step, with where it came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    /// What to do.
    pub kind: ActionKind,
    /// The plan text this action was recognized from (lead sentence through the last block).
    pub span: Span,
    /// The recognizer instance that produced it.
    pub origin: RecognizerId,
    /// Where the action's path came from, when the plan did not write it in full (decision
    /// D32). `Written` for actions without a path.
    #[serde(default)]
    pub path_origin: PathOrigin,
}

/// What to do. Adding a variant is a cross-cutting change: a handler, a policy class, a journal
/// record, dry-run support and a coverage label (docs/extending.md §4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    /// Write a whole file.
    WriteFile {
        /// Target.
        path: RepoPath,
        /// The exact contents (fence body).
        body: TextRef,
        /// Create (must not exist) or overwrite (must exist).
        mode: WriteMode,
    },
    /// Apply a git-style patch with `git apply` semantics: exact context, no fuzz.
    ApplyPatch {
        /// Target, as named by the plan; must equal the paths inside the patch.
        path: RepoPath,
        /// The patch text.
        patch: TextRef,
        /// Preimage blob id from the `index` line, when present.
        pre: Option<AbbrevBlobId>,
        /// Postimage blob id from the `index` line, when present.
        post: Option<AbbrevBlobId>,
    },
    /// Replace text that occurs exactly once (`before:`/`After:`, `old → new`).
    ReplaceExact {
        /// Target.
        path: RepoPath,
        /// The text to find; must occur exactly once (line endings compared as LF).
        before: TextRef,
        /// Its replacement.
        after: TextRef,
    },
    /// Delete a file.
    Delete {
        /// Target.
        path: RepoPath,
        /// `git rm` (index too) rather than a plain delete.
        via_git: bool,
    },
    /// A file the plan says a command generates. Checked to exist after the task's last
    /// non-commit action, before the commit; a path the task names as generated more than once
    /// (an intro notice and a step notice) is one check.
    Generated {
        /// The generated file.
        path: RepoPath,
    },
    /// Run a script in a persistent shell session.
    RunScript(Script),
    /// Run a commit script, as written, only when the task's verification passed.
    Commit(Script),
    /// Plan text with no recognized instruction that is not an instruction either: shown in
    /// place, never blocks.
    Note {
        /// The text.
        text: TextRef,
    },
    /// Something the user must do or confirm by hand. Always blocks until acknowledged.
    Manual {
        /// Why planrun does not do it itself.
        reason: ManualReason,
        /// The plan text to show.
        text: TextRef,
    },
}

/// Create or overwrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WriteMode {
    /// The file must not exist.
    Create,
    /// The file must exist.
    Overwrite,
    /// The plan does not say (`(full contents)` with no Files entry): the manifest decides;
    /// a check error when it cannot.
    FromManifest,
}

/// Why an instruction is manual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManualReason {
    /// A fenced block no recognizer consumed.
    Unrecognized,
    /// An edit described in prose with no exact anchor ("add these members to the class").
    ProseEdit,
    /// A fenced block in a language planrun does not run (`sql`, `text` without a script).
    ForeignLanguage(Box<str>),
    /// An edit that depends on earlier output ("replace the hashes with the Actual values").
    OutputDependent,
    /// A check by eye, in a browser, on another machine.
    HumanCheck,
    /// Text that matched the format's `manual_markers` ("Rens runs it by hand").
    Marked,
}

/// A script and what is expected of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Script {
    /// The shell that runs it, from the fence's info string.
    pub shell: Shell,
    /// The script, as written.
    pub body: TextRef,
    /// What the plan expects, when it says.
    pub expect: Option<Expectation>,
    /// Why the plan runs it.
    pub intent: ScriptIntent,
}

/// The shells planrun runs. Any other fence language becomes `Manual(ForeignLanguage)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Shell {
    /// PowerShell 7 (`powershell`, `pwsh`, `ps1`).
    Pwsh,
    /// bash (`bash`, `sh`, `shell`).
    Bash,
}

/// Why a script runs; drives policy defaults and the commit gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptIntent {
    /// A test, build or check whose result matters.
    Verify,
    /// The gate of a task (its result decides the commit).
    Gate,
    /// The preview half of a preview/apply pair (`-DryRun`, a count).
    Preview,
    /// The apply half: always stops for confirmation after its preview.
    Apply,
    /// Anything else (setup, generation).
    Other,
}
