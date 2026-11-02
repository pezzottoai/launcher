# pezzotto.ai launcher

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://pezzotto.ai/assets/logo-dark.svg">
  <img src="https://pezzotto.ai/assets/logo.svg" alt="pezzotto.ai" width="360">
</picture>

> **'o guaglione** — the shop boy at the stall who hands you the goods.

The launcher is the third piece of [pezzotto.ai](https://pezzotto.ai), the street stall of
open-source knock-offs of closed-source apps. The stall catalogs the clones; the launcher gets
them onto your machine. It is a small cross-platform (Linux, Windows, macOS) installer and
launcher for open-source applications, written in Rust with [egui](https://github.com/emilk/egui):
browse a curated catalog by topic, install with one click, launch, remove. No account, no
telemetry, works offline once the catalog is cached.

![pezzotto.ai launcher](docs/screenshot.png)

Only apps that can actually be installed on your machine are listed; if some need a package
manager you don't have (Flatpak, winget, Homebrew), the sidebar says how many more you'd get.
Ctrl+F (or `/`) searches, Esc clears the search and closes the details pane.

## Downloads

Builds for every platform are on the [releases page](https://github.com/pezzottoai/launcher/releases/latest),
with a rolling [nightly](https://github.com/pezzottoai/launcher/releases/tag/nightly) from `main`:

| platform | files |
|---|---|
| Linux x86_64, aarch64 | `.deb`, `.AppImage`, portable `.tar.gz` |
| Windows x86_64 | setup `.exe` (NSIS, per-user), `.msi`, portable `.zip` |
| Windows arm64 | setup `.exe`, portable `.zip` |
| macOS Apple Silicon, Intel | `.dmg`, portable `.tar.gz` |

Each release has a `SHA256SUMS.txt`. Builds are not signed with a paid certificate yet, which is
very much in the spirit of the stall:

- macOS: the app is ad-hoc signed. The first time, open it, then go to System Settings → Privacy &
  Security → "Open Anyway" (or run `xattr -dr com.apple.quarantine "/Applications/pezzotto.ai launcher.app"`).
- Windows: SmartScreen asks for confirmation ("More info" → "Run anyway").

## What's in the catalog

The launcher reads its catalog from a secondary repository,
[launcher-catalog](https://github.com/pezzottoai/launcher-catalog): one JSON manifest per app
and a JSON Schema for the format. It carries the knock-offs graded on the pezzotto.ai blanket
(the ArtCraft apps, RustGIS, Chord and friends) alongside the open-source classics they sit
next to at the market (GIMP, Inkscape, LibreOffice, QGIS, Blender…). To add an app, open a pull
request there.

The launcher knows which is which. A knock-off carries the stall's verdict in its manifest (the
`pezzotto` block, copied from the [pezzotto.ai catalog](https://pezzotto.ai/#catalog)): what
closed-source original it imitates, its pezzotto grade, whether anyone at the stall has actually
run it, and what the original has that the clone lacks. In the launcher that shows up as a red
`PEZZOTTO · B` stamp on the tile, a **Knock-offs** view in the sidebar, and a grade pane in the
details with the missing features listed and a link to the entry on pezzotto.ai. Searching for
the original ("photoshop") finds its knock-off. The grade is the stall's, not the launcher's: to
dispute it, open an issue on the site repo.

The published `index.json` is signed (minisign, Ed25519) by the catalog's CI, and the launcher
ships the public key: an index whose signature is missing or does not verify is refused, also
when it comes from the offline cache. So a mirror, a proxy or a hijacked DNS entry cannot hand
the launcher a different catalog. Anyone can check the file too:

```sh
minisign -Vm index.json -p keys/pezzotto-launcher-catalog.pub   # key in the launcher-catalog repo
```

## How installs work

Each catalog entry lists installers in order of preference. The launcher uses the first one
that matches the OS/architecture and whose tool is present, and falls back to the next one if
it fails:

| kind | how |
|---|---|
| `flatpak` | `flatpak install --user` from Flathub (Flathub remote is added per-user if missing) |
| `winget` | `winget install --id … --exact --silent` |
| `homebrew` | `brew install [--cask]` |
| `download` | fetched by the launcher itself: the exact file and SHA-256 pinned in the signed catalog, or else the latest GitHub release (checked only against GitHub's own digest); then AppImage/binary made executable, `.zip`/`.tar.gz` extracted, or native installer opened |

Apps whose downloads are all pinned carry `certified: pinned` in the catalog and show
"pinned release, checked <date>" in their details. In **strict mode** (`--strict` or
`PEZZOTTO_LAUNCHER_STRICT=1`) the launcher installs only pinned downloads and never resolves
"latest" at install time. Flatpak, winget and Homebrew installs are unaffected: those package
managers verify their own packages.

Downloaded apps go to the per-user data directory (`~/.local/share/pezzotto-launcher/apps/<id>` on
Linux, `%APPDATA%\pezzotto\pezzotto-launcher\data\apps` on Windows,
`~/Library/Application Support/ai.pezzotto.pezzotto-launcher/apps` on macOS). No admin rights are
needed except where winget/Homebrew/a native installer ask for them. What was installed and how
is recorded in `installed.json` in the same directory.

## Usage

```sh
pezzotto-launcher                          # GUI
pezzotto-launcher list                     # apps and how they'd install here (* = installed)
pezzotto-launcher install org.qgis.qgis
pezzotto-launcher run org.qgis.qgis
pezzotto-launcher remove org.qgis.qgis
pezzotto-launcher --strict install com.getartcraft.GridCraft   # only if pinned in the catalog
pezzotto-launcher --catalog ../launcher-catalog    # use a local catalog checkout (or index.json, or a URL)
```

Environment:

- `PEZZOTTO_LAUNCHER_CATALOG`: catalog URL/path (default: the published `index.json`). Local
  files and directories are not signature-checked; URLs are.
- `PEZZOTTO_LAUNCHER_STRICT=1`: strict mode, see above.
- `PEZZOTTO_LAUNCHER_CATALOG_KEY`: minisign public key (base64) for a catalog you sign yourself.
- `PEZZOTTO_LAUNCHER_UNSIGNED_CATALOG=1`: skip the signature check. For local testing only; the
  window header then says "unsigned catalog".
- `PEZZOTTO_LAUNCHER_RENDERER=wgpu`: use wgpu instead of the default OpenGL renderer.
- `GITHUB_TOKEN`: optional, raises the GitHub API rate limit for release lookups.

## Building

Rust 1.95 or later (required by egui 0.37).

```sh
cargo build --release
```

On Linux you need the X11/Wayland development headers egui uses
(`libxkbcommon-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev` on Debian/Ubuntu).

Installers are made with [cargo-packager](https://github.com/crabnebula-dev/cargo-packager)
(config in `[package.metadata.packager]` in `Cargo.toml`):

```sh
cargo install cargo-packager --locked
cargo build --release --target x86_64-unknown-linux-gnu
cargo packager --release --target x86_64-unknown-linux-gnu --formats deb,appimage
```

## Releases

`.github/workflows/release.yml` builds every platform on each push to `main` (the nightly
pre-release) and on each `vX.Y.Z` tag (a versioned release; the version is taken from the tag).
To cut a release: `git tag v0.2.0 && git push origin v0.2.0`.

## Layout

- `src/catalog.rs`: catalog data model (mirrors the catalog's JSON Schema) and loading/caching
- `src/install.rs`: installers, uninstall, launch, `installed.json`
- `src/ui.rs`: egui front end
- `src/main.rs`: CLI entry point

## History

The launcher started life as FOSS Launcher and moved to pezzotto.ai in October 2026. Bundle
identifier, binary name, environment variables and the per-user data directory changed with the
rebrand; an older FOSS Launcher install keeps its own `installed.json` and is not migrated.

## License

[MIT](LICENSE). No affiliation with any of the originals the clones imitate. That's the point.
