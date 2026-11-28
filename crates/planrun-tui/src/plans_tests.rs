//! The Plans view's tests: the coverage strip, and dates and times in UTC.

use super::*;

fn cov(actions: usize, manifest: usize, notes: usize, manual: usize) -> TaskCoverage {
    TaskCoverage {
        task: planrun_model::TaskId {
            raw: "P1-T1".into(),
            ordinal: 1,
        },
        actions,
        absorbed: Vec::new(),
        manifest,
        notes,
        manual,
        dropped: 0,
    }
}

#[test]
fn the_strip_has_sixteen_cells_and_a_manual_cell_when_there_is_one() {
    for c in [
        cov(10, 3, 2, 0),
        cov(30, 1, 1, 1),
        cov(1, 0, 0, 0),
        cov(5, 5, 5, 5),
    ] {
        let cells: usize = strip(&c).iter().map(|s| s.0).sum();
        assert_eq!(cells, 16);
    }
    let s = strip(&cov(30, 1, 1, 1));
    assert_eq!(s.last().map(|p| (p.0 >= 1, p.2)), Some((true, Token::Warn)));
    assert_eq!(
        strip(&cov(1, 0, 0, 0)),
        vec![(16, Glyph::CovAction, Token::Fg2)]
    );
}

#[test]
fn dates_and_times_are_utc() {
    assert_eq!(civil(0), (1970, 1, 1));
    // 2026-11-27 12:34:56 UTC
    let ms = 1_795_782_896_000;
    assert_eq!(civil(ms), (2026, 11, 27));
    assert_eq!(date_utc(ms), "27 Nov");
    assert_eq!(clock_utc(ms), "12:34");
}

#[test]
fn journalled_plans_are_the_recorded_files_that_exist() {
    let dir = std::env::temp_dir().join(format!("planrun-journalled-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("notes")).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(dir.join("notes/p1.md"), "# P1\n").unwrap_or_else(|e| panic!("{e}"));
    let absolute = dir.join("notes/p1.md").display().to_string();
    let found = journalled(&dir, ["notes/p1.md", "notes/gone.md", absolute.as_str()]);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(found, [dir.join("notes/p1.md")]);
}
