#!/usr/bin/env bash
# Bumps the app version everywhere a release needs it to agree.
#
# Four files carry the number and all four have to match, or the release fails:
#   package.json               the npm manifest
#   src-tauri/tauri.conf.json  what the bundler stamps into the installers
#   src-tauri/Cargo.toml       the crate version
#   src-tauri/Cargo.lock       the crate's own entry in the lock file
#
# .github/workflows/release.yml re-checks the first three against the pushed tag
# before it builds anything (a v0.2.0 tag on 0.1.0 manifests would ship bundles
# labelled 0.1.0 forever), so a mismatch here costs a whole build cycle. This
# script keeps them in step, and refuses to run if they have already drifted.
#
# packaging/aur/{PKGBUILD,.SRCINFO} are deliberately NOT touched: the AUR
# workflow rewrites pkgver and the .deb checksum after the GitHub release is
# published, and commits that back itself.
#
# Usage:
#   scripts/bump-version.sh patch|minor|major   bump the current version
#   scripts/bump-version.sh 1.2.3               set an explicit version
#   scripts/bump-version.sh patch --commit      also make the release commit + tag
#
# --commit stops short of pushing: a tag push is what triggers the release
# build, so that stays a deliberate, separate step (the script prints it).

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

commit=0
target=""
for arg in "$@"; do
  case "$arg" in
    --commit) commit=1 ;;
    -h | --help)
      sed -n '/^# Usage:/,/^$/s/^# \{0,1\}//p' "${BASH_SOURCE[0]}"
      exit 0
      ;;
    -*)
      echo "unknown option: $arg" >&2
      exit 1
      ;;
    *)
      [ -z "$target" ] || { echo "give only one version argument" >&2; exit 1; }
      target="$arg"
      ;;
  esac
done

[ -n "$target" ] || { echo "usage: scripts/bump-version.sh patch|minor|major|X.Y.Z [--commit]" >&2; exit 1; }

# Read each file the same way the release workflow does, so "they agree" here
# means the same thing it will mean in CI.
json_version() { grep -m1 '"version"' "$1" | sed -E 's/.*"version"[^"]*"([^"]+)".*/\1/'; }
toml_version() { grep -m1 '^version' "$1" | sed -E 's/.*"([^"]+)".*/\1/'; }
lock_version() { awk '/^name = "quran-reader"$/ { getline; sub(/^version = "/, ""); sub(/"$/, ""); print; exit }' "$1"; }

current="$(json_version package.json)"
for check in \
  "src-tauri/tauri.conf.json:$(json_version src-tauri/tauri.conf.json)" \
  "src-tauri/Cargo.toml:$(toml_version src-tauri/Cargo.toml)" \
  "src-tauri/Cargo.lock:$(lock_version src-tauri/Cargo.lock)"; do
  file="${check%%:*}"
  found="${check#*:}"
  if [ "$found" != "$current" ]; then
    echo "versions have drifted: package.json is $current, $file is $found" >&2
    echo "fix that by hand first — this script only bumps a consistent set" >&2
    exit 1
  fi
done

case "$target" in
  major | minor | patch)
    IFS=. read -r major minor patch <<<"$current"
    case "$target" in
      major) new="$((major + 1)).0.0" ;;
      minor) new="$major.$((minor + 1)).0" ;;
      patch) new="$major.$minor.$((patch + 1))" ;;
    esac
    ;;
  *)
    [[ "$target" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || {
      echo "not a version: $target (want X.Y.Z, or patch|minor|major)" >&2
      exit 1
    }
    new="$target"
    ;;
esac

[ "$new" != "$current" ] || { echo "already at $new — nothing to do" >&2; exit 1; }

if git rev-parse -q --verify "refs/tags/v$new" >/dev/null; then
  echo "tag v$new already exists" >&2
  exit 1
fi

echo "==> $current -> $new"

# -i with an anchored first match: "version" appears many times over in both the
# JSON manifests and Cargo.toml's dependency table, and only the topmost one is
# the package's own.
sed -i "0,/\"version\": \"$current\"/s//\"version\": \"$new\"/" package.json
sed -i "0,/\"version\": \"$current\"/s//\"version\": \"$new\"/" src-tauri/tauri.conf.json
sed -i "0,/^version = \"$current\"/s//version = \"$new\"/" src-tauri/Cargo.toml

# Prefer cargo for the lock file so it stays in cargo's own format; --offline
# because bumping our own package version needs no registry access.
if command -v cargo >/dev/null 2>&1; then
  (cd src-tauri && cargo update --offline --quiet -p quran-reader)
else
  awk -v new="$new" '
    /^name = "quran-reader"$/ { print; getline; print "version = \"" new "\""; next }
    { print }
  ' src-tauri/Cargo.lock >src-tauri/Cargo.lock.tmp
  mv src-tauri/Cargo.lock.tmp src-tauri/Cargo.lock
fi

# Re-read rather than trust the edits: a silently unmatched sed would otherwise
# surface as a failed release build twenty minutes from now.
for check in \
  "package.json:$(json_version package.json)" \
  "src-tauri/tauri.conf.json:$(json_version src-tauri/tauri.conf.json)" \
  "src-tauri/Cargo.toml:$(toml_version src-tauri/Cargo.toml)" \
  "src-tauri/Cargo.lock:$(lock_version src-tauri/Cargo.lock)"; do
  file="${check%%:*}"
  found="${check#*:}"
  [ "$found" = "$new" ] || { echo "$file is $found, expected $new" >&2; exit 1; }
  echo "    $file"
done

if [ "$commit" -eq 1 ]; then
  echo "==> committing and tagging v$new"
  git add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock
  git commit -m "chore(release): v$new"
  git tag "v$new"
  echo
  echo "push when ready — the tag is what starts the release build:"
  echo "    git push origin $(git rev-parse --abbrev-ref HEAD) && git push origin v$new"
else
  echo
  echo "next:"
  echo "    git commit -am \"chore(release): v$new\" && git tag v$new"
  echo "    git push origin $(git rev-parse --abbrev-ref HEAD) && git push origin v$new"
fi
