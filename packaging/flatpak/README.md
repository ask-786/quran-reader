# Flatpak packaging

Source of truth for the `io.github.ask_786.quran-reader` Flatpak, published on
Flathub. Unlike the AUR package, this one is built **from source**: Flathub does
not accept a repackaged `.deb` for an open-source app.

| File                                          | What it is                                              |
| --------------------------------------------- | ------------------------------------------------------- |
| `io.github.ask_786.quran-reader.yml`          | The manifest Flathub builds                             |
| `io.github.ask_786.quran-reader.metainfo.xml` | Store listing: description, screenshots, releases       |
| `io.github.ask_786.quran-reader.desktop`      | Launcher entry, named after the app ID as Flatpak needs |
| `cargo-sources.json`                          | Every crate in `src-tauri/Cargo.lock`, pinned           |
| `node-sources.json`                           | Every package in `pnpm-lock.yaml`, pinned               |

The app ID differs from Tauri's `com.quranreader.app` on purpose. Flathub only
accepts an ID under a domain you can prove is yours, and `io.github.ask_786` is
proved by the GitHub account. Nothing in the app reads the Flatpak ID, so the two
can stay different.

## Why the source lists exist

A Flathub build has no network. Everything Cargo and pnpm would download has to
be listed in the manifest with a checksum, so flatpak-builder can fetch it first.
The two JSON files are those lists, generated from the lockfiles, and they go
stale whenever a lockfile changes.

`.github/workflows/flathub.yml` regenerates both from the tag's lockfiles on
every release, so they only need regenerating by hand for a local build after a
dependency change. Use the same pinned generators the workflow does:

```sh
tools=9d86d336a39b9af38b86f114b6ce5820fe260f91   # BUILDER_TOOLS in flathub.yml
uvx --from "git+https://github.com/flatpak/flatpak-builder-tools.git@$tools#subdirectory=node" \
  flatpak-node-generator pnpm pnpm-lock.yaml -o packaging/flatpak/node-sources.json
curl -fsSLO "https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/$tools/cargo/flatpak-cargo-generator.py"
uv run --no-project --with aiohttp --with tomlkit --with pyyaml \
  python flatpak-cargo-generator.py src-tauri/Cargo.lock -o packaging/flatpak/cargo-sources.json
```

The cargo generator ships as a single script with no entry point, which is why
it runs through `uv run` rather than as an installed tool.

## Building locally

```sh
flatpak install --user flathub org.flatpak.Builder org.gnome.Sdk//51 \
  org.freedesktop.Sdk.Extension.rust-stable//26.08 org.freedesktop.Sdk.Extension.node22//26.08

flatpak run org.flatpak.Builder --user --install --force-clean \
  --mirror-screenshots-url=https://dl.flathub.org/media/ --repo=repo \
  build-dir packaging/flatpak/io.github.ask_786.quran-reader.yml
flatpak run io.github.ask_786.quran-reader

# what Flathub's bot checks
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest packaging/flatpak/io.github.ask_786.quran-reader.yml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder repo repo
```

`--mirror-screenshots-url` does what Flathub's build does: it points the
catalogue's screenshots and icons at `dl.flathub.org/media` and stores copies in
the local repo. Only Flathub's build uploads them there, so a local lint always
reports `appstream-external-screenshot-url` and
`appstream-remote-icon-not-mirrored`; any other error is real.

The manifest builds the git tag it names, so a local build of uncommitted work
needs a copy of the manifest whose `type: git` source is swapped for
`type: dir` pointing at the checkout (skip `node_modules`, `src-tauri/target`,
`build` and `.svelte-kit`, or the copy takes minutes).

## How a release reaches Flathub

After the first submission, Flathub keeps the manifest in its own repo,
`flathub/io.github.ask_786.quran-reader`, and publishes whatever merges there.

1. `scripts/bump-version.sh` adds the `<release>` entry to the metainfo, from
   the commits since the last tag. It has to be in the tag, because the tag is
   what Flathub builds.
2. Publishing the GitHub release triggers `flathub.yml`. It regenerates the
   source lists from the tag's lockfiles, points the manifest at the tag, and
   opens an `update-X.Y.Z` PR in the Flathub repo. It then commits the updated
   files back to `master`, as `aur.yml` does.
3. Flathub's bot test-builds the PR. **Merging it is the one manual step**;
   Flathub publishes within a few hours.

To re-run it for a tag: Actions → Publish to Flathub → Run workflow.

### One-time setup, after Flathub accepts the app

Add a `FLATHUB_TOKEN` secret (Settings → Secrets and variables → Actions): a
token that can push to the Flathub repo. A classic personal access token with
the `public_repo` scope works. Until the secret exists, the workflow skips with
a notice instead of failing.

## First submission

Follow https://docs.flathub.org/docs/for-app-authors/submission: fork
`flathub/flathub`, branch from `new-pr`, add the manifest and the two JSON
files, and open a PR against `new-pr`.
