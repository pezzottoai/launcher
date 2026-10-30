# pezzotto.ai launcher

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
(the ArtCraft apps, RustGIS and friends) alongside the open-source classics they sit next to at
the market (GIMP, Inkscape, LibreOffice, QGIS, Blender…). To add an app, open a pull request
there.

## How installs work

Each catalog entry lists installers in order of preference. The launcher uses the first one
that matches the OS/architecture and whose tool is present, and falls back to the next one if
it fails:

| kind | how |
|---|---|
| `flatpak` | `flatpak install --user` from Flathub (Flathub remote is added per-user if missing) |
| `winget` | `winget install --id … --exact --silent` |
| `homebrew` | `brew install [--cask]` |
| `download` | fetched by the launcher itself, pinned URL + SHA-256 or the latest GitHub release (checked against GitHub's published digest), then: AppImage/binary made executable, `.zip`/`.tar.gz` extracted, or native installer opened |

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
pezzotto-launcher --catalog ../launcher-catalog    # use a local catalog checkout (or index.json, or a URL)
```

Environment:

- `PEZZOTTO_LAUNCHER_CATALOG`: catalog URL/path (default: the published `index.json`).
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
