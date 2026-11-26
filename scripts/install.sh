#!/bin/sh
# Installs planrun on Linux: the latest release, or a build of this clone (D83).
#
#   install.sh [--from-source] [--install-dir DIR]
#
# Without --from-source: downloads the latest release's x86_64-unknown-linux-gnu archive from
# GitHub and puts planrun in DIR (default ~/.local/bin). With --from-source: builds this clone
# with `cargo install --locked` into the same directory (its name must be bin). `planrun update`
# keeps the install current. The script warns when DIR is not on PATH; it edits no rc file.
#
# One line, without a clone:
#   curl -fsSL https://raw.githubusercontent.com/Renseck/planrun/main/scripts/install.sh | sh
set -eu

repo=Renseck/planrun
target=x86_64-unknown-linux-gnu
from_source=no
dir="${HOME}/.local/bin"

fail() {
    echo "planrun: $*" >&2
    exit 1
}

while [ $# -gt 0 ]; do
    case "$1" in
        --from-source) from_source=yes ;;
        --install-dir)
            [ $# -ge 2 ] || fail "--install-dir needs a directory"
            dir="$2"
            shift
            ;;
        -h | --help)
            sed -n '2,12s/^# \{0,1\}//p' "$0"
            exit 0
            ;;
        *) fail "unknown argument: $1" ;;
    esac
    shift
done

mkdir -p "$dir"
dir=$(cd "$dir" && pwd)

install_from_source() {
    # $0 is the script when it runs from a file; under `curl … | sh` it is the shell.
    root=$(cd "$(dirname "$0")/.." 2>/dev/null && pwd) || root=
    [ -n "$root" ] && [ -f "$root/crates/planrun-cli/Cargo.toml" ] \
        || fail "--from-source builds a clone: git clone https://github.com/$repo, then run scripts/install.sh --from-source in it"
    command -v cargo >/dev/null 2>&1 || fail "--from-source needs cargo on PATH (https://rustup.rs)"
    [ "$(basename "$dir")" = bin ] \
        || fail "--from-source needs an --install-dir named bin (cargo installs into <root>/bin): $dir"
    cargo install --path "$root/crates/planrun-cli" --locked --force --root "$(dirname "$dir")"
}

install_release() {
    [ "$(uname -s)" = Linux ] && [ "$(uname -m)" = x86_64 ] \
        || fail "releases are built for Linux x86_64 only; use --from-source"
    command -v curl >/dev/null 2>&1 || fail "the download needs curl"
    command -v tar >/dev/null 2>&1 || fail "the download needs tar"
    json=$(curl -fsSL -H "Accept: application/vnd.github+json" -A planrun-install \
        "https://api.github.com/repos/$repo/releases/latest")
    tag=$(printf '%s\n' "$json" | sed -n 's/^ *"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)
    [ -n "$tag" ] || fail "the latest release has no tag"
    name="planrun-$tag-$target.tar.gz"
    temp=$(mktemp -d)
    trap 'rm -rf "$temp"' EXIT
    curl -fsSL -A planrun-install -o "$temp/$name" \
        "https://github.com/$repo/releases/download/$tag/$name" \
        || fail "release $tag has no $name"
    tar -xzf "$temp/$name" -C "$temp"
    install -m 755 "$temp/planrun" "$dir/planrun"
}

if [ "$from_source" = yes ]; then install_from_source; else install_release; fi

case ":$PATH:" in
    *":$dir:"*) ;;
    *) echo "planrun: $dir is not on PATH; add it in your shell's profile" >&2 ;;
esac
"$dir/planrun" --version
