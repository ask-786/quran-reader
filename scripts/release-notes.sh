#!/usr/bin/env bash
# Writes the release notes for a tag, from the commits since the previous one.
#
# Commits follow Conventional Commits (commitlint enforces it), so the type
# says who a commit is for. Readers of a release care about feat, fix and perf;
# dependency bumps go in a collapsed list; release and AUR bookkeeping, CI,
# style and the rest are left out. So is anything scoped to the website (site),
# which deploys on its own and isn't part of the app. Merge commits are skipped
# — the commits they bring in are listed on their own.
#
# .github/workflows/release.yml runs this and hands the output to tauri-action
# as the release body. Run it locally to see what a tag will say before
# pushing it.
#
# With --appstream it prints the same feat/fix/perf changes as a <release>
# entry for the Flatpak metainfo instead, which Flathub shows as the
# changelog. scripts/bump-version.sh inserts it before the release commit, so
# the tag Flathub builds already carries it.
#
# Usage:
#   scripts/release-notes.sh v1.2.3                  notes for an existing tag
#   scripts/release-notes.sh                         notes for what HEAD would release
#   scripts/release-notes.sh --appstream 1.2.3       metainfo <release> for what HEAD would release

set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

case "${1:-}" in
  -h | --help)
    sed -n '/^# Usage:/,/^$/s/^# \{0,1\}//p' "${BASH_SOURCE[0]}"
    exit 0
    ;;
esac

appstream=""
if [ "${1:-}" = "--appstream" ]; then
  appstream="${2:?usage: scripts/release-notes.sh --appstream X.Y.Z}"
  shift 2
fi

tag="${1:-}"
end="${tag:-HEAD}"
git rev-parse --verify --quiet "$end^{commit}" > /dev/null || { echo "no such tag: $end" >&2; exit 1; }

# The previous app release. `v[0-9]*` keeps the packs-v* tafsir tags out.
prev="$(git describe --tags --abbrev=0 --match 'v[0-9]*' "$end^" 2> /dev/null || true)"
range="${prev:+$prev..}$end"

# owner/repo, from Actions when there, else from the origin remote.
repo="${GITHUB_REPOSITORY:-$(git remote get-url origin | sed -E 's#^.*github\.com[:/]##; s#\.git$##')}"

feat="" fix="" perf="" deps=""
# Plain-text twins for the metainfo: no links or markdown, which AppStream
# doesn't render.
feat_xml="" fix_xml="" perf_xml=""
xml() {
  local s="$1"
  s="${s//'&'/'&amp;'}"
  s="${s//'<'/'&lt;'}"
  s="${s//'>'/'&gt;'}"
  printf '%s' "${s^}"
}
while IFS=$'\t' read -r sha subject; do
  [[ "$subject" =~ ^([a-z]+)(\(([^\)]*)\))?(!)?:\ (.*)$ ]] || continue
  type="${BASH_REMATCH[1]}"
  scope="${BASH_REMATCH[3]}"
  breaking="${BASH_REMATCH[4]}"
  text="${BASH_REMATCH[5]}"

  line="- ${breaking:+**Breaking:** }${scope:+**$scope:** }$text ([\`${sha:0:7}\`](https://github.com/$repo/commit/$sha))"
  item="          <li>$(xml "$text")</li>"$'\n'
  case "$type/$scope" in
    */site) ;;
    feat/*) feat+="$line"$'\n' feat_xml+="$item" ;;
    fix/*) fix+="$line"$'\n' fix_xml+="$item" ;;
    perf/*) perf+="$line"$'\n' perf_xml+="$item" ;;
    chore/deps) deps+="- $text"$'\n' ;;
  esac
done < <(git log --no-merges --reverse --format='%H%x09%s' "$range")

if [ -n "$appstream" ]; then
  xml_section() { [ -z "$2" ] || printf '        <p>%s</p>\n        <ul>\n%s        </ul>\n' "$1" "$2"; }
  printf '    <release version="%s" date="%s">\n      <description>\n' "$appstream" "$(date +%F)"
  xml_section "New:" "$feat_xml"
  xml_section "Fixes:" "$fix_xml"
  xml_section "Performance:" "$perf_xml"
  [ -n "$feat_xml$fix_xml$perf_xml" ] || printf '        <p>Maintenance release: no user-facing changes.</p>\n'
  printf '      </description>\n    </release>\n'
  exit 0
fi

section() { [ -z "$2" ] || printf '### %s\n\n%s\n' "$1" "$2"; }

section "New" "$feat"
section "Fixes" "$fix"
section "Performance" "$perf"
if [ -z "$feat$fix$perf" ]; then
  printf 'Maintenance release: no user-facing changes.\n\n'
fi
if [ -n "$deps" ]; then
  printf '<details>\n<summary>Dependency updates</summary>\n\n%s\n</details>\n\n' "$deps"
fi

cat << EOF
### Installing

Download the installer for your system from the assets below. They are unsigned: Windows SmartScreen warns before running the installer, and on macOS, after copying the app to Applications, clear the download quarantine or macOS will call it damaged:

\`\`\`
xattr -dr com.apple.quarantine "/Applications/Quran Reader.app"
\`\`\`
EOF

if [ -n "$prev" ]; then
  printf '\n**Full changelog:** https://github.com/%s/compare/%s...%s\n' "$repo" "$prev" "${tag:-master}"
fi
