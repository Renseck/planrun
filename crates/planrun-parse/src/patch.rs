//! Reading a git patch as text: its file header (paths, `index` line) and the shape of its
//! hunks. Nothing is applied here; `git apply` does that in planrun-exec (decision D20).

use planrun_model::AbbrevBlobId;

/// The file header of a one-file git patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchHeader {
    /// Every path the header names (`diff --git a/… b/…`, `--- a/…`, `+++ b/…`), `/dev/null`
    /// left out.
    pub paths: Vec<String>,
    /// The preimage blob id of the `index` line.
    pub pre: Option<AbbrevBlobId>,
    /// The postimage blob id of the `index` line.
    pub post: Option<AbbrevBlobId>,
}

/// Reads the header of `patch`. Refused: no `diff --git` or `---`/`+++` header, more than one
/// file, an `index` line that does not parse.
pub fn header(patch: &str) -> Result<PatchHeader, String> {
    let mut header = PatchHeader {
        paths: Vec::new(),
        pre: None,
        post: None,
    };
    let mut files = 0;
    let mut markers = 0;
    for line in patch.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            files += 1;
            if files > 1 {
                return Err("the patch changes more than one file".to_owned());
            }
            header.paths.extend(git_paths(rest)?);
            continue;
        }
        if line.starts_with("@@") {
            continue;
        }
        if let Some(ids) = line.strip_prefix("index ") {
            if markers == 0 {
                (header.pre, header.post) = index_ids(ids)?;
            }
            continue;
        }
        let side = line
            .strip_prefix("--- ")
            .map(|p| (p, "a/"))
            .or_else(|| line.strip_prefix("+++ ").map(|p| (p, "b/")));
        let Some((path, prefix)) = side.filter(|_| markers < 2) else {
            continue;
        };
        markers += 1;
        if path == "/dev/null" {
            continue;
        }
        let Some(path) = path.strip_prefix(prefix) else {
            return Err(format!("`{line}` does not name an `{prefix}` path"));
        };
        header.paths.push(path.to_owned());
    }
    if files == 0 && markers < 2 {
        return Err("not a git patch: no `diff --git` line and no `---`/`+++` pair".to_owned());
    }
    Ok(header)
}

fn git_paths(rest: &str) -> Result<[String; 2], String> {
    let parsed = rest
        .strip_prefix("a/")
        .and_then(|r| r.rsplit_once(" b/"))
        .map(|(a, b)| [a.to_owned(), b.to_owned()]);
    parsed.ok_or_else(|| format!("`diff --git {rest}` does not name `a/…` and `b/…` paths"))
}

fn index_ids(ids: &str) -> Result<(Option<AbbrevBlobId>, Option<AbbrevBlobId>), String> {
    let range = ids.split_whitespace().next().unwrap_or("");
    let Some((pre, post)) = range.split_once("..") else {
        return Err(format!("`index {ids}` is not `<pre>..<post>`"));
    };
    let parse = |id: &str| AbbrevBlobId::parse(id).map_err(|e| e.to_string());
    Ok((Some(parse(pre)?), Some(parse(post)?)))
}

/// Checks the hunks of `patch`: at least one, and each hunk's body has exactly the line counts
/// its `@@ -a,b +c,d @@` header gives. An empty line counts as context, as `git apply` reads it.
/// Returns the number of hunks.
pub fn check_hunks(patch: &str) -> Result<usize, String> {
    let mut lines = patch.lines().enumerate().peekable();
    while lines.next_if(|(_, l)| !l.starts_with("@@")).is_some() {}
    let mut hunks = 0;
    while let Some((n, line)) = lines.next() {
        if line.starts_with('\\') {
            continue;
        }
        let Some((mut old, mut new)) = hunk_counts(line) else {
            return Err(format!(
                "patch line {}: `{line}` is not a hunk header",
                n + 1
            ));
        };
        hunks += 1;
        while old > 0 || new > 0 {
            let Some((m, body)) = lines.next() else {
                return Err(format!(
                    "the hunk at patch line {} ends early: {old} old and {new} new lines missing",
                    n + 1
                ));
            };
            let (takes_old, takes_new) = match body.chars().next() {
                None | Some(' ') => (1, 1),
                Some('-') => (1, 0),
                Some('+') => (0, 1),
                Some('\\') => (0, 0),
                Some(_) => {
                    return Err(format!("patch line {}: `{body}` is not a hunk line", m + 1));
                }
            };
            if takes_old > old || takes_new > new {
                return Err(format!(
                    "the hunk at patch line {} has more lines than its header says",
                    n + 1
                ));
            }
            old -= takes_old;
            new -= takes_new;
        }
    }
    if hunks == 0 {
        return Err("the patch has no hunk".to_owned());
    }
    Ok(hunks)
}

/// `@@ -a[,b] +c[,d] @@…` → (b, d), a missing count being 1.
fn hunk_counts(line: &str) -> Option<(u32, u32)> {
    let rest = line.strip_prefix("@@ -")?;
    let (ranges, _) = rest.split_once(" @@")?;
    let (old, new) = ranges.split_once(" +")?;
    Some((count(old)?, count(new)?))
}

fn count(range: &str) -> Option<u32> {
    match range.split_once(',') {
        Some((start, len)) => {
            start.parse::<u32>().ok()?;
            len.parse().ok()
        }
        None => range.parse::<u32>().ok().map(|_| 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PATCH: &str = "diff --git a/src/lib.rs b/src/lib.rs
index aa80cf1..97bd92a 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,4 @@ fn x
 a

+b
-c
+d
";

    #[test]
    fn the_header_names_paths_and_blob_ids() {
        let h = header(PATCH).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(h.paths, ["src/lib.rs"; 4]);
        assert_eq!(h.pre.map(|i| i.to_string()).as_deref(), Some("aa80cf1"));
        assert_eq!(h.post.map(|i| i.to_string()).as_deref(), Some("97bd92a"));
        assert_eq!(check_hunks(PATCH), Ok(1));
    }

    #[test]
    fn a_new_file_names_dev_null() {
        let p = "diff --git a/n.rs b/n.rs\nnew file mode 100644\nindex 0000000..1111111\n\
                 --- /dev/null\n+++ b/n.rs\n@@ -0,0 +1 @@\n+x\n";
        let h = header(p).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(h.paths, ["n.rs", "n.rs", "n.rs"]);
        assert!(h.pre.is_some_and(|i| i.is_absent()));
        assert_eq!(check_hunks(p), Ok(1));
    }

    #[test]
    fn bad_headers_are_refused() {
        assert!(header("just text\n").is_err());
        assert!(header(&format!("{PATCH}diff --git a/b b/b\n")).is_err());
        assert!(header("diff --git a/x b/x\nindex zz..yy\n").is_err());
    }

    #[test]
    fn hunk_counts_must_match() {
        let short = "--- a/a\n+++ b/a\n@@ -1,2 +1 @@\n-a\n+b\n";
        assert!(check_hunks(short).is_err());
        let long = "--- a/a\n+++ b/a\n@@ -1 +1 @@\n-a\n+b\n+c\n";
        assert!(check_hunks(long).is_err());
        assert!(check_hunks("--- a/a\n+++ b/a\n").is_err());
        let two = "--- a/a\n+++ b/a\n@@ -1 +1 @@\n-a\n+b\n\\ No newline at end of file\n@@ -5 +5 @@\n-x\n+y\n";
        assert_eq!(check_hunks(two), Ok(2));
    }
}
