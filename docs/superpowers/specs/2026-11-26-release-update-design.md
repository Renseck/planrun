# P6a — Release, install and `planrun update` (design)

**Status:** approved in brainstorming on 2026-11-26; the plan follows in
`docs/plans/2026-11-26-p6a-release-update.md` (assay format, so planrun can execute it).

## Goal

planrun is easy to install and keep current from GitHub:

- a CI gate on every push and pull request, on Windows and Linux;
- a release workflow that gates, builds and publishes binaries on a version tag;
- install scripts that download the latest release, or build from a clone;
- `planrun update`, which installs a newer release over the running binary and removes what the
  previous update left behind.

## Decisions taken in brainstorming

| Topic | Choice | Reason |
|---|---|---|
| Repository visibility | Rens makes `Renseck/planrun` public before the first release | Anonymous API and asset downloads; no token handling in planrun |
| Download and unpack | Shell out to `curl` and `tar` | No new crate (D9); both ship with Windows 10+ and Linux; planrun already shells out to git |
| Targets | `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu` | The platforms the docs claim; macOS is untested (D64 has no macOS path) |
| Install modes | Download (default) and from source, one script per OS | Both put the binary in the same place, so `update` works after either |
| Extras | CI gate on push/PR | Closes P5 open item 4 (Linux paths run on a Linux machine) |
| Not included | SHA256SUMS, dependabot, CHANGELOG, release profile tuning | Declined; TLS covers transport, the `--version` check covers a wrong or broken binary |

Checked before going public: by file name, git history holds nothing from `tests/corpus/local/`
except its README (D16). There is no `LICENSE` file although `Cargo.toml` says MIT; T1 adds one.

## New decision D83 — `planrun update` writes its own install directory

Hard rule 6 (D7) limits what **plan execution** writes: the working tree, `.git/planrun/` and the
OS temp directory. `planrun update` is not plan execution. It writes exactly two places: a
directory under the OS temp directory, and the directory of `std::env::current_exe()`, where it
creates and renames only `planrun[.exe]`, `planrun.new[.exe]` and `planrun.old[.exe]`. Every
planrun start removes a leftover `planrun.old[.exe]` and `planrun.new[.exe]` there, best effort.

D83 also records: curl and tar are external tools like git; the release and asset naming below;
releases come from `v<major>.<minor>.<patch>` tags whose version equals `[workspace.package]`.

## Tasks

The gate (fmt, clippy `-D warnings`, test, each also with the `e2e` feature) passes after each.

1. **T1 — docs.** The P5→P6 handover `docs/handovers/2026-11-26-p5-to-p6.md` with the six open items
   from the P5 review (stated decision 22 of the P5 plan, items 1–4; the two message strings that
   lost their `\` continuation; `resume-dirt` covering only file-action paths). D83 in
   `docs/decisions.md`. `LICENSE` (MIT). README: current status, decision range, an Install/Update
   section.
2. **T2 — `.github/workflows/ci.yml`.**
3. **T3 — `.github/workflows/release.yml`.**
4. **T4 — `scripts/install.ps1`, `scripts/install.sh`.**
5. **T5 — `update` logic**: version compare, release JSON, asset choice, the swap; unit tests.
6. **T6 — `update` wiring**: the curl and tar calls, the subcommand, the startup cleanup;
   architecture.md, roadmap.md (P6a) and CLAUDE.md layout line for `update`.

## CI — `.github/workflows/ci.yml`

- Triggers: `push` to `main`, `pull_request`, `workflow_call` (the release reuses it).
- Matrix: `windows-latest`, `ubuntu-latest`; `fail-fast: false`.
- Env: `PLANRUN_REQUIRE_SHELLS=1`, `CARGO_TERM_COLOR=always`. Both runner images have pwsh and
  bash; on Windows, planrun finds Git's bash through `git --exec-path`
  (`crates/planrun-exec/src/shell/bash.rs`).
- Steps: `actions/checkout`, `dtolnay/rust-toolchain@stable` with `clippy, rustfmt`,
  `Swatinem/rust-cache`, then:
  - `cargo fmt --check` (ubuntu only)
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo clippy -p planrun-cli --all-targets --features e2e -- -D warnings`
  - `cargo test`
  - `cargo test -p planrun-cli --features e2e --test run`
- Actions are referenced by major tag (`@v4`), not by SHA.
- Not run in CI: the `--ignored` local-corpus tests (the corpus is private).

## Release — `.github/workflows/release.yml`

- Trigger: `push` of tags matching `v*.*.*`.
- `gate`: `uses: ./.github/workflows/ci.yml`.
- `version`: the tag without its `v` must equal `version` in `[workspace.package]` of
  `Cargo.toml`; otherwise the job fails with both values in the message.
- `build` (needs `gate`, `version`; matrix over the two targets):
  `cargo build --release --locked -p planrun-cli --target <target>`, then package:
  - `planrun-v<ver>-x86_64-pc-windows-msvc.zip`, `planrun.exe` at the archive root;
  - `planrun-v<ver>-x86_64-unknown-linux-gnu.tar.gz`, `planrun` at the archive root.

  Each archive is uploaded with `actions/upload-artifact`.
- `publish` (needs `build`; `permissions: contents: write`): download the artifacts,
  `gh release create <tag> --generate-notes <archives>`.
- The first release is `v0.1.0`; the plan bumps no version. Rens tags after the plan is done.

## Install scripts

### `scripts/install.ps1`

Parameters: `-FromSource` (switch), `-InstallDir` (default
`$env:LOCALAPPDATA\Programs\planrun\bin`).

- Download (default): query `releases/latest`, take the `x86_64-pc-windows-msvc` asset, download
  and expand it to a temp directory, copy `planrun.exe` to `-InstallDir`. Add `-InstallDir` to the
  user `PATH` when it is missing, and say so.
- `-FromSource`: requires the script to run from a clone (the repository root is the parent of
  `$PSScriptRoot` and must hold `crates/planrun-cli/Cargo.toml`; under `irm | iex`, where
  `$PSScriptRoot` is empty, it stops with a message saying to clone first) and `cargo` on `PATH`. Runs
  `cargo install --path crates/planrun-cli --locked --root <parent of -InstallDir>`, so the binary
  lands in `-InstallDir`; the PATH step is the same.
- Ends with `planrun --version` from `-InstallDir`.
- One-liner: `irm https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.ps1 | iex`.
- `$ErrorActionPreference = 'Stop'`; PowerShell 7 and Windows PowerShell 5.1 both run it (the
  one-liner may run in either).

### `scripts/install.sh`

Options: `--from-source`, `--install-dir <dir>` (default `$HOME/.local/bin`). The same two modes,
with the `x86_64-unknown-linux-gnu` asset, `tar -xzf`, and `--root <parent of dir>`. It warns when
the directory is not on `PATH`; it does not edit shell rc files. `set -eu`. Refuses a platform
other than Linux x86_64 in download mode.

### Verification

In the plan: `-FromSource` into a temporary `-InstallDir`, then `--version` of the result; the same
for `install.sh --from-source` where bash and cargo exist. Download mode needs a release; it is a
post-release check in the handover.

## `planrun update`

### Surface

```
planrun update           install the latest release if it is newer
planrun update --check   report only; change nothing
```

Exit 0 when up to date, when `--check` reports, and after a successful update; exit 1 on any
failure, with the installed binary unchanged.

Messages:

- `planrun 0.1.0 is up to date`
- `planrun 0.2.0 is available (installed: 0.1.0); run planrun update`
- `updated planrun 0.1.0 → 0.2.0`

### Code

`crates/planrun-cli/src/update.rs` (and `update_tests.rs` once the file passes about 400 lines).
planrun-parse and planrun-exec are unchanged. planrun-cli gains `serde_json.workspace = true` (a
workspace dependency already justified in D9; no new crate).

Pure parts (T5), each unit-tested:

- `Version { major, minor, patch }` with `parse("v0.2.0" | "0.2.0")` and `Ord`. A tag that does not
  parse is an error, not "not newer".
- `Release::from_json(&str)` → `tag_name` and `assets: Vec<Asset { name, url }>` from
  `browser_download_url`. Unknown fields ignored.
- `asset_for(release, target)`: the asset named `planrun-<tag>-<target>.<zip|tar.gz>`; none →
  `update-no-asset`.
- `TARGET`: a `cfg` match on `target_os`/`target_arch` giving one of the two triples; any other
  build has no target and `update` reports `update-unsupported`.
- `swap(dir, exe_name, new_binary)` takes paths, never `current_exe()`:
  1. copy `new_binary` to `dir/planrun.new[.exe]`;
  2. Unix: set mode 755, then `rename(new, exe)` (atomic on one filesystem);
  3. Windows: `rename(exe, planrun.old.exe)`, then `rename(new, exe)`; if the second rename fails,
     `rename(old, exe)` back and remove `new`.
  Tested on a temp directory with plain files, including the rollback (the second rename made to
  fail by a directory in the way).
- `cleanup(dir)`: best-effort removal of `planrun.old[.exe]` and `planrun.new[.exe]`; errors are
  ignored.

Effectful parts (T6):

1. Fetch `https://api.github.com/repos/Renseck/planrun/releases/latest` with
   `curl -fsSL -H "Accept: application/vnd.github+json" -A planrun`. On Windows the programs are
   `%SystemRoot%\System32\curl.exe` and `tar.exe` by absolute path, so an MSYS curl or GNU tar on
   `PATH` never runs (GNU tar does not read zip). Not found → `update-tools`.
2. Compare with `env!("CARGO_PKG_VERSION")`. Not newer → up to date. `--check` → report.
3. Download the asset to `temp_dir()/planrun-update-<pid>/` (`curl -fsSL -o`), extract there
   (`tar -xf` on Windows reads zip; `tar -xzf` on Linux). A failure → `update-download`.
4. Run `<extracted> --version`; its output must contain the release version → else
   `update-verify`.
5. `swap` into the directory of `current_exe()` → failure is `update-swap`.
6. Remove the temp directory; a drop guard also removes it on every error path.

Startup cleanup: `main` calls `cleanup` on the directory of `current_exe()` before dispatching any
command. One or two `remove_file` calls, no process start.

Diagnostic codes: `update-tools`, `update-fetch`, `update-unsupported`, `update-no-asset`,
`update-download`, `update-verify`, `update-swap`. Tests pin the codes.

### Testing

- Unit tests for every pure part (T5), with a release JSON fixture as a string constant.
- One `#[ignore]` live test that fetches and parses the real `releases/latest`; it returns a usable
  result after `v0.1.0`.
- After `v0.1.0` and a later `v0.1.1`: `planrun update --check`, then `planrun update` on Windows
  and Linux, then a second start that removes `planrun.old.exe`. Listed in the handover as the
  post-release acceptance.

## Out of scope

Checksums, a token or private-repo path, an update hint on other commands, pre-release channels,
`update --version <x>`, macOS and other targets, dependabot, a CHANGELOG, release-profile tuning,
the cosmetic P5 message fix (it stays an open item in the handover).
