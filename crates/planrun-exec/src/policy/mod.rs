//! Policy classes and the command classifier (docs/architecture.md §6.5; decisions D13, D68).
//!
//! The classifier is a built-in allowlist: a script's class is the highest class of any command
//! in it, a command the tables do not know is Destructive, and so is a script the splitter
//! cannot take apart with confidence. No format spec, `[checks]` entry or flag changes a class,
//! and a runner has one classifier, not one per format.

mod bash;
mod decide;
mod heads;
mod pwsh;
mod sed;

pub use decide::{Decision, Policy, PolicyError, Skip};
pub use heads::DEVREADY_VERSION;

use planrun_model::Shell;

/// The command classes of decision D13, least to most dangerous.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PolicyClass {
    /// Reads only (`git status`, `rg`).
    #[default]
    Read,
    /// Builds or tests (`cargo test`, `dotnet build`, `npm ci`).
    Build,
    /// Changes the repository (`git add`, `git commit`, `replace … -All`, file actions).
    Modify,
    /// Destroys or reaches outside (`Remove-Item -Recurse`, `git reset --hard`, `git push`,
    /// anything unclassified).
    Destructive,
}

/// Classifies a script: the most dangerous class of any command in it.
pub trait CommandClassifier: Send + Sync {
    /// The class of `script` run by `shell`.
    fn classify(&self, shell: Shell, script: &str) -> PolicyClass;
}

/// A script's class with the reasons that made it Destructive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Classification {
    /// The highest class of any command.
    pub class: PolicyClass,
    /// What made it Destructive (`git push`, `unknown command \`foo\``, `unparsed: …`), in
    /// script order; empty below Destructive.
    pub reasons: Vec<String>,
}

/// The built-in classifier (D68), with the tables of `heads.rs`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultClassifier;

impl DefaultClassifier {
    /// The class of `script` and why.
    pub fn explain(&self, shell: Shell, script: &str) -> Classification {
        let mut acc = Acc::default();
        let parsed = match shell {
            Shell::Pwsh => pwsh::classify(script, &mut acc),
            Shell::Bash => bash::classify(script, &mut acc),
        };
        if let Err(why) = parsed {
            acc.add(PolicyClass::Destructive, Some(format!("unparsed: {why}")));
        }
        Classification {
            class: acc.class,
            reasons: acc.reasons,
        }
    }
}

impl CommandClassifier for DefaultClassifier {
    fn classify(&self, shell: Shell, script: &str) -> PolicyClass {
        self.explain(shell, script).class
    }
}

/// One word of a command as the splitter saw it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Word {
    /// Its text: the value without quotes when `literal`, the raw spelling otherwise.
    pub text: String,
    /// No variable, subexpression or other expansion: `text` is exactly the value.
    pub literal: bool,
    /// Some part was quoted (a quoted word is never a parameter or a keyword).
    pub quoted: bool,
}

impl Word {
    /// A literal word that was not quoted.
    pub fn bare(&self) -> Option<&str> {
        (self.literal && !self.quoted).then_some(self.text.as_str())
    }
}

/// What the splitter found so far: the highest class, and the reasons for Destructive.
#[derive(Debug, Default)]
pub(crate) struct Acc {
    pub class: PolicyClass,
    pub reasons: Vec<String>,
}

impl Acc {
    /// Raises the class to `class`; a reason is kept only for Destructive.
    pub fn add(&mut self, class: PolicyClass, reason: Option<String>) {
        self.class = self.class.max(class);
        if class == PolicyClass::Destructive {
            self.reasons
                .push(reason.unwrap_or_else(|| "destructive".to_owned()));
        }
    }
}

/// A redirection target's class (D68): nothing for the null device, Modify for a literal
/// relative path without `..`, Destructive for anything else.
pub(crate) fn redirect_target(word: &Word, acc: &mut Acc) {
    let text = word.text.as_str();
    let null = ["$null", "/dev/null"].contains(&text) || text.eq_ignore_ascii_case("nul");
    if null && (word.literal || text == "$null") {
        return;
    }
    let relative = word.literal
        && !text.is_empty()
        && !text.starts_with(['/', '\\', '~'])
        && !text.contains(':')
        && !text.split(['/', '\\']).any(|s| s == "..");
    if relative {
        acc.add(PolicyClass::Modify, None);
        return;
    }
    acc.add(
        PolicyClass::Destructive,
        Some(format!("a redirection to `{text}`")),
    );
}

#[cfg(test)]
mod tests;
