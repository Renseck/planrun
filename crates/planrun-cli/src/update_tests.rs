//! Tests of `planrun update`'s pure parts; the network only in the ignored live test.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::*;

const RELEASE: &str = r#"{
  "url": "https://api.github.com/repos/Renseck/planrun/releases/1",
  "tag_name": "v0.2.0",
  "name": "v0.2.0",
  "draft": false,
  "prerelease": false,
  "assets": [
    {
      "name": "planrun-v0.2.0-x86_64-pc-windows-msvc.zip",
      "size": 1234,
      "browser_download_url": "https://github.com/Renseck/planrun/releases/download/v0.2.0/planrun-v0.2.0-x86_64-pc-windows-msvc.zip"
    },
    {
      "name": "planrun-v0.2.0-x86_64-unknown-linux-gnu.tar.gz",
      "size": 1234,
      "browser_download_url": "https://github.com/Renseck/planrun/releases/download/v0.2.0/planrun-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
    }
  ]
}"#;

const WINDOWS: Target = Target {
    triple: "x86_64-pc-windows-msvc",
    archive: Archive::Zip,
};

const LINUX: Target = Target {
    triple: "x86_64-unknown-linux-gnu",
    archive: Archive::TarGz,
};

fn version(text: &str) -> Version {
    Version::parse(text).unwrap_or_else(|| panic!("`{text}` is a version"))
}

/// An empty directory of the test's own under the OS temp directory.
fn scratch(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("planrun-update-test-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("{e}"));
    dir
}

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

#[rustfmt::skip]
#[test]
fn versions_parse_with_or_without_a_v() {
    let rows: &[(&str, Option<[u64; 3]>)] = &[
        ("0.1.0", Some([0, 1, 0])),
        ("v0.1.0", Some([0, 1, 0])),
        ("v10.20.30", Some([10, 20, 30])),
        ("0.1", None),
        ("0.1.0.1", None),
        ("0.1.0-rc.1", None),
        ("v", None),
        ("", None),
        ("1..0", None),
        ("+1.0.0", None),
        ("V0.1.0", None),
    ];
    for (text, want) in rows {
        let got = Version::parse(text).map(|v| [v.major, v.minor, v.patch]);
        assert_eq!(got, *want, "{text}");
    }
}

#[test]
fn versions_compare_numerically() {
    assert!(version("0.10.0") > version("0.9.9"));
    assert!(version("1.0.0") > version("0.99.99"));
    assert!(version("0.1.1") > version("0.1.0"));
    assert_eq!(version("v0.1.0"), version("0.1.0"));
    assert_eq!(version("v0.2.0").to_string(), "0.2.0");
}

#[test]
fn the_printed_version_must_name_the_release() {
    assert!(version_matches("planrun 0.2.0\n", version("0.2.0")));
    assert!(!version_matches("planrun 0.2.01\n", version("0.2.0")));
    assert!(!version_matches("planrun 0.1.0\n", version("0.2.0")));
    assert!(!version_matches("", version("0.2.0")));
}

#[test]
fn a_release_reads_its_tag_and_assets() {
    let release = Release::from_json(RELEASE).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(release.tag, "v0.2.0");
    assert_eq!(release.version, version("0.2.0"));
    assert_eq!(release.assets.len(), 2);
    assert_eq!(
        release.assets[1].url,
        "https://github.com/Renseck/planrun/releases/download/v0.2.0/planrun-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
    );
}

#[test]
fn a_tag_that_is_no_version_or_broken_json_is_an_error() {
    let nightly = RELEASE.replace("\"tag_name\": \"v0.2.0\"", "\"tag_name\": \"nightly\"");
    let e = Release::from_json(&nightly).err();
    assert_eq!(e.map(|e| e.code), Some("update-fetch"));
    let e = Release::from_json("{\"message\": \"Not Found\"}").err();
    assert_eq!(e.map(|e| e.code), Some("update-fetch"));
}

#[test]
fn each_target_has_its_asset() {
    assert_eq!(
        asset_name("v0.2.0", WINDOWS),
        "planrun-v0.2.0-x86_64-pc-windows-msvc.zip"
    );
    assert_eq!(
        asset_name("v0.2.0", LINUX),
        "planrun-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
    );
    let release = Release::from_json(RELEASE).unwrap_or_else(|e| panic!("{e}"));
    for target in [WINDOWS, LINUX] {
        let asset = asset_for(&release, target).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(asset.name, asset_name("v0.2.0", target));
    }
    let mac = Target {
        triple: "aarch64-apple-darwin",
        archive: Archive::TarGz,
    };
    assert_eq!(
        asset_for(&release, mac).err().map(|e| e.code),
        Some("update-no-asset")
    );
}

#[test]
fn this_build_knows_its_target() {
    let triple = TARGET.map(|t| t.triple);
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        assert_eq!(triple, Some("x86_64-pc-windows-msvc"));
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        assert_eq!(triple, Some("x86_64-unknown-linux-gnu"));
    } else {
        assert_eq!(triple, None);
    }
}

#[test]
fn siblings_go_before_the_extension() {
    assert_eq!(
        sibling(Path::new("bin/planrun.exe"), "old"),
        Path::new("bin/planrun.old.exe")
    );
    assert_eq!(
        sibling(Path::new("bin/planrun"), "new"),
        Path::new("bin/planrun.new")
    );
}

#[test]
fn swap_installs_the_new_binary_and_cleanup_removes_the_old_one() {
    let dir = scratch("swap");
    let exe = dir.join(format!("planrun{}", std::env::consts::EXE_SUFFIX));
    let new = dir.join("download");
    write(&exe, "old build");
    write(&new, "new build");
    swap(&exe, &new).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(read(&exe), "new build");
    assert!(!sibling(&exe, "new").exists());
    if cfg!(windows) {
        assert_eq!(read(&sibling(&exe, "old")), "old build");
    }
    cleanup(&exe);
    assert!(!sibling(&exe, "old").exists());
    assert_eq!(read(&exe), "new build");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_failed_second_rename_puts_the_old_binary_back() {
    let dir = scratch("rollback");
    let exe = dir.join("planrun.exe");
    let staged = sibling(&exe, "new");
    write(&exe, "old build");
    write(&staged, "new build");
    let mut calls = 0;
    let result = replace_aside(&exe, &staged, |from, to| {
        calls += 1;
        if calls == 2 {
            return Err(io::Error::other("in use"));
        }
        fs::rename(from, to)
    });
    assert!(result.is_err());
    assert_eq!(calls, 3);
    assert_eq!(read(&exe), "old build");
    assert!(!staged.exists());
    assert!(!sibling(&exe, "old").exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_failed_atomic_rename_leaves_the_binary_and_removes_the_staged_copy() {
    let dir = scratch("atomic");
    let exe = dir.join("planrun");
    let staged = sibling(&exe, "new");
    write(&exe, "old build");
    write(&staged, "new build");
    let result = replace_atomic(&exe, &staged, |_, _| Err(io::Error::other("read-only")));
    assert!(result.is_err());
    assert_eq!(read(&exe), "old build");
    assert!(!staged.exists());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_stale_old_binary_is_replaced_by_the_next_swap() {
    let dir = scratch("stale");
    let exe = dir.join("planrun.exe");
    let staged = sibling(&exe, "new");
    write(&exe, "second build");
    write(&sibling(&exe, "old"), "first build");
    write(&staged, "third build");
    replace_aside(&exe, &staged, |from, to| fs::rename(from, to)).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(read(&exe), "third build");
    assert_eq!(read(&sibling(&exe, "old")), "second build");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn cleanup_without_leftovers_does_nothing() {
    let dir = scratch("clean");
    let exe = dir.join("planrun.exe");
    write(&exe, "build");
    cleanup(&exe);
    assert_eq!(read(&exe), "build");
    let _ = fs::remove_dir_all(&dir);
}

/// Needs the network and a published release: `cargo test -p planrun-cli -- --ignored
/// the_latest_release`.
#[test]
#[ignore]
fn the_latest_release_is_fetched_and_has_this_builds_asset() {
    let tools = Tools::find().unwrap_or_else(|e| panic!("{e}"));
    let json = tools.fetch(LATEST).unwrap_or_else(|e| panic!("{e}"));
    let release = Release::from_json(&json).unwrap_or_else(|e| panic!("{e}"));
    if let Some(target) = TARGET {
        asset_for(&release, target).unwrap_or_else(|e| panic!("{e}"));
    }
}
