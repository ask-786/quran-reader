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

Regenerate both after any dependency change, before a release that Flathub will
build:

```sh
# once
uv tool install "git+https://github.com/flatpak/flatpak-builder-tools.git#subdirectory=node"
curl -LO https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py

# every time
flatpak-node-generator pnpm pnpm-lock.yaml -o packaging/flatpak/node-sources.json
uv run --no-project --with aiohttp --with tomlkit --with pyyaml \
  python flatpak-cargo-generator.py src-tauri/Cargo.lock -o packaging/flatpak/cargo-sources.json
```

The cargo generator ships as a single script with no entry point, which is why
it runs through `uv run` rather than `uv tool install`.

## Building locally

```sh
flatpak install --user flathub org.flatpak.Builder org.gnome.Sdk//51 \
  org.freedesktop.Sdk.Extension.rust-stable//26.08 org.freedesktop.Sdk.Extension.node22//26.08

flatpak run org.flatpak.Builder --user --install --force-clean \
  build-dir packaging/flatpak/io.github.ask_786.quran-reader.yml
flatpak run io.github.ask_786.quran-reader
```

The manifest builds the git tag it names, so a local build of uncommitted work
needs a copy of the manifest whose `type: git` source is swapped for
`type: dir` pointing at the checkout (skip `node_modules`, `src-tauri/target`,
`build` and `.svelte-kit`, or the copy takes minutes).

## Releasing to Flathub

After the first submission, Flathub keeps the manifest in its own repo,
`flathub/io.github.ask_786.quran-reader`. For each release:

1. Add a `<release>` entry to the metainfo and regenerate the source lists, on
   `master`, before tagging. The tag is what Flathub builds.
2. In the Flathub repo, bump `tag` and `commit` in the manifest, copy in the
   two JSON files, and open a PR. Flathub's bot builds it, and merging publishes.

## First submission

Follow https://docs.flathub.org/docs/for-app-authors/submission: fork
`flathub/flathub`, branch from `new-pr`, add the manifest and the two JSON
files, and open a PR against `new-pr`.
