---
name: cut-release
description: Cut a release of this app — bump the version, tag, publish the draft, land AUR. Use when asked to cut, ship, or tag a release or bump the version, and when a release build fails or a published release never reached AUR.
---

# Cut a release

The chain has one manual link in the middle: the build leaves a **draft**, and nothing downstream moves until a human publishes it. Pushing the tag is not the finish line.

## Steps

1. **Start from a clean, current `master`.** `git status` clean, then `git pull --ff-only`.

2. **Bump.** `scripts/bump-version.sh patch|minor|major|X.Y.Z` writes the four files that have to agree — `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` — and refuses to run if they have already drifted. `--commit` also makes the `chore(release): vX.Y.Z` commit and the tag. Leave `packaging/aur/*` alone; step 6 rewrites it.

3. **Push the branch, then the tag.** `git push origin master && git push origin vX.Y.Z`. The tag push is the trigger — pushing the branch alone releases nothing.

4. **Watch the build.** The Release workflow's `verify-version` job fails fast when the tag and the manifests disagree; after that the three bundle jobs run and they dominate the wall time.

5. **Publish the draft.** Confirm the assets are all attached first — Linux `.deb`/`.rpm`/AppImage, macOS `.dmg`, Windows `.msi`/`.exe` — then `gh release edit vX.Y.Z --draft=false`. Draft assets 404 for everyone but the repo owner, so nothing can consume the release until this runs.

6. **Confirm AUR landed.** Publishing starts the Publish to AUR run, which rewrites `pkgver` and the `.deb` checksum and commits back to `master`. `git pull` when it is green.

Done when the release is published, the AUR run is green, and local `master` carries its commit.

## Notes

- macOS and Windows artifacts are unsigned, so Gatekeeper and SmartScreen warn on install. Expected, not a broken release.
- A 403 on release creation means `RELEASE_TOKEN` expired. The job uses that repo-scoped PAT rather than the Actions token, which gets a hard 403 on `POST /releases` here.
- `packs-v*` tags are a separate track — downloadable tafsir editions, published by hand through the Publish Tafsir Packs workflow. An app release neither builds nor needs them.
