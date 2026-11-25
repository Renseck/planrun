//! What happens to an action before it runs (decisions D13, D50, D67, D69). P4 computes the
//! decision; P5's runner acts on it and prompts.

use planrun_model::{Action, ActionKind, ExitExpect, PlanSource, Script, ScriptIntent, Task};

use super::{CommandClassifier, PolicyClass};

/// What the runner does with an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// It runs without asking.
    Auto,
    /// It asks first; the class that made it ask.
    Ask(PolicyClass),
    /// A file action outside the task's declared files (D50): it always asks.
    AskUnscoped,
    /// A Destructive script: shown whole, it needs a typed `yes`.
    TypedYes,
    /// A Manual: it waits until acknowledged.
    Wait,
    /// A Note: shown in place.
    Show,
    /// It does not run (D67).
    Skip(Skip),
}

/// Why an action does not run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    /// A verification expected to fail, without `--run-expected-failures` (D12, D67).
    ExpectedFailure,
}

/// A refused `--auto` value.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyError {
    /// Not a class name.
    #[error("`--auto` takes read, build and modify; `{0}` is not one of them")]
    Unknown(String),
    /// `destructive`: a Destructive script always needs a typed `yes` (D69).
    #[error("`--auto destructive` is refused: a Destructive script always needs a typed `yes`")]
    Destructive,
}

/// The run's policy: which classes run without asking, and whether expected failures run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Policy {
    auto: Vec<PolicyClass>,
    run_expected_failures: bool,
}

impl Policy {
    /// A policy; `auto` may hold Read, Build and Modify, never Destructive.
    pub fn new(auto: &[PolicyClass], run_expected_failures: bool) -> Result<Policy, PolicyError> {
        if auto.contains(&PolicyClass::Destructive) {
            return Err(PolicyError::Destructive);
        }
        Ok(Policy {
            auto: auto.to_vec(),
            run_expected_failures,
        })
    }

    /// Parses `--auto`'s value: `read,build,modify`, any of them, in any case.
    pub fn parse_auto(text: &str) -> Result<Vec<PolicyClass>, PolicyError> {
        text.split(',')
            .map(|part| match part.trim().to_ascii_lowercase().as_str() {
                "read" => Ok(PolicyClass::Read),
                "build" => Ok(PolicyClass::Build),
                "modify" => Ok(PolicyClass::Modify),
                "destructive" => Err(PolicyError::Destructive),
                _ => Err(PolicyError::Unknown(part.trim().to_owned())),
            })
            .collect()
    }

    /// The decision for `action` of `task`; scripts are classified by `classifier`.
    pub fn decide(
        &self,
        task: &Task,
        action: &Action,
        source: &PlanSource,
        classifier: &dyn CommandClassifier,
    ) -> Decision {
        match &action.kind {
            ActionKind::Note { .. } => Decision::Show,
            ActionKind::Manual { .. } => Decision::Wait,
            ActionKind::Generated { .. } => Decision::Auto,
            ActionKind::WriteFile { path, .. }
            | ActionKind::ApplyPatch { path, .. }
            | ActionKind::ReplaceExact { path, .. }
            | ActionKind::Delete { path, .. } => {
                if task.manifest.scopes(path) {
                    Decision::Auto
                } else {
                    Decision::AskUnscoped
                }
            }
            ActionKind::RunScript(script) if self.skips(script) => {
                Decision::Skip(Skip::ExpectedFailure)
            }
            ActionKind::RunScript(script) | ActionKind::Commit(script) => {
                let class = classifier.classify(script.shell, source.resolve(&script.body));
                self.script(script, class)
            }
        }
    }

    /// D67: a Verify script that expects failure, unless expected failures run. A Gate, a
    /// Preview or an Apply runs (a skipped Gate leaves `$gate` unset for the commit), and a
    /// `Code(n)` is never an expected failure.
    fn skips(&self, script: &Script) -> bool {
        let failure = script
            .expect
            .as_ref()
            .is_some_and(|e| e.exit == ExitExpect::Failure);
        failure && script.intent == ScriptIntent::Verify && !self.run_expected_failures
    }

    fn script(&self, script: &Script, class: PolicyClass) -> Decision {
        if class == PolicyClass::Destructive {
            return Decision::TypedYes;
        }
        if script.intent == ScriptIntent::Apply || !self.auto.contains(&class) {
            return Decision::Ask(class);
        }
        Decision::Auto
    }
}

#[cfg(test)]
mod tests {
    use planrun_model::{ActionKind, ChangeKind, Expectation, PathOrigin, WriteMode};

    use super::*;
    use crate::ir::Ir;
    use crate::policy::DefaultClassifier;

    fn expecting(mut action: Action, exit: ExitExpect, intent: ScriptIntent) -> Action {
        if let ActionKind::RunScript(script) | ActionKind::Commit(script) = &mut action.kind {
            script.intent = intent;
            script.expect = Some(Expectation {
                exit,
                lines: None,
                prose: None,
                comments: None,
            });
        }
        action
    }

    #[test]
    fn decisions() {
        use Decision::{Ask, AskUnscoped, Auto, Show, TypedYes, Wait};
        use PolicyClass::{Build, Modify, Read};
        use ScriptIntent::{Apply, Gate, Preview, Verify};
        let mut ir = Ir::default();
        let manifest = ir.manifest(&[
            ("src/lib.rs", ChangeKind::Modify, PathOrigin::Written),
            ("README.md", ChangeKind::Modify, PathOrigin::Bare),
        ]);
        let none = Policy::default();
        let build = Policy::new(&[Read, Build], false).unwrap_or_else(|e| panic!("{e}"));
        let all = Policy::new(&[Read, Build, Modify], true).unwrap_or_else(|e| panic!("{e}"));
        let fails =
            |ir: &mut Ir, intent| expecting(ir.script("cargo test"), ExitExpect::Failure, intent);
        #[rustfmt::skip]
        let rows: Vec<(Action, &Policy, Decision)> = vec![
            (ir.note("a note"), &none, Show),
            (ir.manual("by hand"), &all, Wait),
            (ir.generated("Cargo.lock"), &none, Auto),
            (ir.write("src/lib.rs", "x", WriteMode::Overwrite), &none, Auto),
            (ir.write("src/new.rs", "x", WriteMode::Create), &all, AskUnscoped),
            (ir.replace("README.md", "a", "b"), &all, AskUnscoped),
            (ir.delete("src/lib.rs", true), &none, Auto),
            (ir.script("git status"), &none, Ask(Read)),
            (ir.script("git status"), &build, Auto),
            (ir.script("cargo test"), &build, Auto),
            (ir.script("git add -A"), &build, Ask(Modify)),
            (ir.script("git add -A"), &all, Auto),
            (ir.script("git push"), &all, TypedYes),
            (ir.commit("git add -A; git commit -m x"), &all, Auto),
            (expecting(ir.script("replace a b -All"), ExitExpect::Success, Apply), &all, Ask(Modify)),
            (fails(&mut ir, Verify), &none, Decision::Skip(Skip::ExpectedFailure)),
            (fails(&mut ir, Verify), &build, Decision::Skip(Skip::ExpectedFailure)),
            (fails(&mut ir, Verify), &all, Auto),
            (fails(&mut ir, Gate), &build, Auto),
            (fails(&mut ir, Preview), &none, Ask(Build)),
            (expecting(ir.script("rg x"), ExitExpect::Code(1), Verify), &none, Ask(Read)),
            (expecting(ir.commit("git commit -m x"), ExitExpect::Failure, Verify), &all, Auto),
        ];
        let task = ir.task("P1-T1", manifest, vec![]);
        let source = ir.source();
        for (action, policy, expected) in rows {
            let got = policy.decide(&task, &action, &source, &DefaultClassifier);
            assert_eq!(got, expected, "{:?}", source.slice(action.span));
        }
    }

    #[test]
    fn auto_values() {
        use PolicyClass::{Build, Modify, Read};
        assert_eq!(
            Policy::parse_auto("read,Build, MODIFY"),
            Ok(vec![Read, Build, Modify])
        );
        assert_eq!(
            Policy::parse_auto("read,destructive"),
            Err(PolicyError::Destructive)
        );
        let unknown = PolicyError::Unknown("all".to_owned());
        assert_eq!(Policy::parse_auto("all"), Err(unknown));
        let refused = Policy::new(&[PolicyClass::Destructive], false);
        assert_eq!(refused, Err(PolicyError::Destructive));
    }
}
