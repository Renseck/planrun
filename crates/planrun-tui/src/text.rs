//! Text measured in cells (`unicode-width`), cut and padded to a width.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// The width of `s` in cells.
pub fn width(s: &str) -> usize {
    s.width()
}

/// `s` cut to at most `cells` cells; when cut, its last cell is `ellipsis`.
pub fn fit(s: &str, cells: usize, ellipsis: &str) -> String {
    if s.width() <= cells {
        return s.to_owned();
    }
    if cells == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0;
    for c in s.chars() {
        let w = c.width().unwrap_or(0);
        if used + w > cells - 1 {
            break;
        }
        out.push(c);
        used += w;
    }
    out.push_str(ellipsis);
    out
}

/// `s` cut to `cells` cells and padded with spaces to exactly that width.
pub fn pad(s: &str, cells: usize, ellipsis: &str) -> String {
    let mut out = fit(s, cells, ellipsis);
    let w = out.width();
    out.extend(std::iter::repeat_n(' ', cells.saturating_sub(w)));
    out
}

/// `s` padded on the left to `cells` cells (never cut).
pub fn right(s: &str, cells: usize) -> String {
    let w = s.width();
    let mut out: String = std::iter::repeat_n(' ', cells.saturating_sub(w)).collect();
    out.push_str(s);
    out
}

/// A duration for the tree and the console: `0.1s`, `12.4s`, `3m05s`, `1h02m`.
pub fn duration(ms: u64) -> String {
    let secs = ms / 1000;
    if secs < 60 {
        return format!("{}.{}s", secs, (ms % 1000) / 100);
    }
    if secs < 3600 {
        return format!("{}m{:02}s", secs / 60, secs % 60);
    }
    format!("{}h{:02}m", secs / 3600, (secs % 3600) / 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitting_cuts_with_an_ellipsis_and_pads() {
        assert_eq!(fit("src/lib.rs", 20, "…"), "src/lib.rs");
        assert_eq!(fit("New-Item docs/modules.txt", 12, "…"), "New-Item do…");
        assert_eq!(width(&fit("New-Item docs/modules.txt", 12, "…")), 12);
        assert_eq!(pad("ab", 4, "…"), "ab  ");
        assert_eq!(right("7", 3), "  7");
        assert_eq!(fit("abc", 0, "…"), "");
    }

    #[test]
    fn durations_read_at_a_glance() {
        assert_eq!(duration(140), "0.1s");
        assert_eq!(duration(12_400), "12.4s");
        assert_eq!(duration(185_000), "3m05s");
        assert_eq!(duration(3_720_000), "1h02m");
    }
}
