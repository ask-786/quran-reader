# AUR packaging

Source of truth for the [`quran-reader-bin`](https://aur.archlinux.org/packages/quran-reader-bin)
AUR package. It repackages the `.deb` the release workflow already builds, so
installing takes seconds instead of the ~10 minutes a Tauri source build costs.

`.github/workflows/aur.yml` publishes this to the AUR automatically. The steps
below are what that workflow does, and what to run if you ever need to do it by
hand.

## One-time setup

1. Generate a key for the AUR and add the public half to your account
   (https://aur.archlinux.org → My Account → SSH Public Key):

   ```sh
   ssh-keygen -t ed25519 -f ~/.ssh/aur -C 'aur'
   ```

2. Add the **private** half to the repository as the `AUR_SSH_PRIVATE_KEY`
   secret (Settings → Secrets and variables → Actions). The workflow cannot push
   without it.

3. Create the package by pushing to it once — the repo path must match `pkgname`
   exactly, and cloning it before it exists is expected to report an empty
   repository:

   ```sh
   git clone ssh://aur@aur.archlinux.org/quran-reader-bin.git ~/aur/quran-reader-bin
   ```

## How a release reaches the AUR

`release.yml` builds the artifacts into a **draft** release. Draft assets 404 for
everyone except the repo owner, so nothing can package them yet.

Publishing that draft fires `release: published`, which triggers `aur.yml`. It
sets `pkgver` from the tag, regenerates the checksums with `updpkgsums` and
`.SRCINFO` with `makepkg --printsrcinfo`, builds the package as a check, pushes
to the AUR, and commits the regenerated files back to the default branch.

So the only manual step is publishing the release.

To re-run it for a tag (or after a failure): Actions → Publish to AUR → Run
workflow, with the tag as input.

## Doing it by hand

Only needed for a packaging-only fix, where `pkgrel` bumps but `pkgver` doesn't —
the workflow always resets `pkgrel` to 1, so it can't express that.

```sh
cd packaging/aur
updpkgsums                          # rewrite sha256sums
makepkg --printsrcinfo > .SRCINFO   # the AUR rejects pushes without this

makepkg -f                          # verify it builds
namcap PKGBUILD
namcap quran-reader-bin-*.pkg.tar.zst

cp PKGBUILD .SRCINFO quran-reader.desktop ~/aur/quran-reader-bin/
cd ~/aur/quran-reader-bin
git add PKGBUILD .SRCINFO quran-reader.desktop
git commit -m "quran-reader-bin 0.1.0-2"
git push
```

`.SRCINFO` must be regenerated whenever `PKGBUILD` changes — it is what the AUR
web interface and helpers read.

namcap's "detected and implicitly satisfied" warnings on the built package are
expected: those libraries arrive transitively through `webkit2gtk-4.1` and
`gtk3`, and listing them directly would be wrong.

## Notes

- The `.deb` is built on ubuntu-22.04, so the binary needs glibc >= 2.34. Arch is
  far past that, and every other shared library it needs resolves against current
  Arch packages.
- Nothing is vendored into the binary's library path, so unlike the AppImage this
  uses the host's webkit2gtk and wayland. It does not hit the `EGL_BAD_PARAMETER`
  failure the unpatched AppImage has on current Mesa — see the repack step in
  `release.yml`.
- A from-source `quran-reader` package is possible; everything it needs
  (`database/quran.db`, the vendored fonts) is tracked in git. It would produce
  the same binary, so it would only cost users build time.
