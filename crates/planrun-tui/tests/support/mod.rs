//! The TUI's test harness (decision D93): a greeter fixture copy, the synthetic plans in a
//! plans directory beside it, an app with a fixed clock and fixed labels, and a core over a
//! 120 × 36 `TestBackend`.

#![allow(dead_code)]

#[path = "../../../../tests/support/fixture.rs"]
pub mod fixture;

use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ratatui::Terminal;
use ratatui::backend::TestBackend;

use fixture::Fixture;
use planrun_tui::app::App;
use planrun_tui::driver::Core;
use planrun_tui::glyph::Glyphs;
use planrun_tui::grid::grid;
use planrun_tui::jobs::Bg;
use planrun_tui::terminal::context;

/// 2026-11-27 12:34:56 UTC.
pub const NOW: u64 = 1_795_782_896_000;

/// The synthetic plans, and the corpus README, which is not a plan.
pub const PLANS: [&str; 5] = [
    "README.md",
    "assay-style.md",
    "before-after-style.md",
    "devready-style.md",
    "superpowers-style.md",
];

/// A fixture with its plans directory.
pub struct Setup {
    /// The repository.
    pub fixture: Fixture,
    /// The plans directory, outside the repository.
    pub plans: PathBuf,
}

/// The greeter fixture and a plans directory with `plans`, each file's time set to [`NOW`].
pub fn setup(plans: &[&str]) -> Setup {
    let fixture = Fixture::greeter(false);
    let dir = fixture.outside().join("plans");
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    let corpus =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus/synthetic");
    for name in plans {
        let target = dir.join(name);
        std::fs::copy(corpus.join(name), &target).unwrap_or_else(|e| panic!("{name}: {e}"));
        let file = File::options()
            .write(true)
            .open(&target)
            .unwrap_or_else(|e| panic!("{e}"));
        file.set_modified(UNIX_EPOCH + Duration::from_millis(NOW))
            .unwrap_or_else(|e| panic!("{e}"));
    }
    Setup {
        fixture,
        plans: dir,
    }
}

impl Setup {
    /// The app on this setup, with the labels of the design's artboards.
    pub fn app(&self) -> App {
        self.app_with(None)
    }

    /// The app with `update` as its update check.
    pub fn app_with(&self, update: Option<planrun_tui::jobs::UpdateCheck>) -> App {
        let mut ctx = context(
            self.fixture.path(),
            Some(&self.plans),
            "0.1.0",
            self.fixture.env(),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        ctx.repo = "greeter".to_owned();
        ctx.target = r"C:\src\greeter".to_owned();
        ctx.plans_label = "./plans".to_owned();
        App::new(ctx, Glyphs::unicode(), Arc::new(|| NOW), update)
    }

    /// A started core whose plans are all checked.
    pub fn core(&self) -> Core<TestBackend> {
        let terminal = Terminal::new(TestBackend::new(120, 36)).unwrap_or_else(|e| panic!("{e}"));
        let mut core = Core::new(self.app(), terminal);
        core.start();
        let n = std::fs::read_dir(&self.plans).map_or(0, |d| d.count());
        let checked = core.wait(Duration::from_secs(60), |app| {
            let entries = &app.plans().entries;
            entries.len() == n && entries.iter().all(|e| matches!(e.check, Bg::Current(_)))
        });
        assert!(checked, "the plans were not checked in time");
        core
    }
}

/// The screen as text, for a snapshot.
pub fn screen(core: &mut Core<TestBackend>) -> String {
    core.draw().unwrap_or_else(|e| panic!("{e}"));
    grid(core.terminal().backend().buffer())
}

/// The current time, for tests that wait.
pub fn now() -> SystemTime {
    SystemTime::now()
}
