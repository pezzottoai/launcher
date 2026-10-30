//! Catalog data model (mirrors `schema/app.schema.json` in the catalog repo) and loading.

use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::paths;

pub const DEFAULT_CATALOG: &str =
    "https://raw.githubusercontent.com/pezzottoai/launcher-catalog/main/index.json";
/// minisign public key the catalog index is signed with
/// (`keys/pezzotto-launcher-catalog.pub` in the launcher-catalog repository).
pub const CATALOG_PUBLIC_KEY: &str = "RWSno9tBYnLnG/KQRfAa2iK69RTj9Zfl1ThL59bCuCs2NC71wZflkRPG";
const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub schema_version: u32,
    #[serde(default)]
    pub generated: Option<String>,
    pub topics: Vec<Topic>,
    pub apps: Vec<App>,
    /// How a remote index was authenticated. Not part of the file; `None` for local files and
    /// directories.
    #[serde(skip)]
    pub signature: Option<SignatureStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureStatus {
    /// `index.json.minisig` verified against the catalog public key.
    Verified,
    /// PEZZOTTO_LAUNCHER_UNSIGNED_CATALOG=1 was set: the signature was not checked.
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub name: String,
    pub summary: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub developer: String,
    pub license: String,
    pub homepage: String,
    pub source_code: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub screenshots: Vec<String>,
    pub topics: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Present when every download installer is pinned to one release file with its hash.
    #[serde(default)]
    pub certified: Option<Certified>,
    pub installers: Vec<Installer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certified {
    pub level: String,
    pub date: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Linux,
    Windows,
    Macos,
}

impl Platform {
    pub fn current() -> Option<Self> {
        match std::env::consts::OS {
            "linux" => Some(Self::Linux),
            "windows" => Some(Self::Windows),
            "macos" => Some(Self::Macos),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Windows => "Windows",
            Self::Macos => "macOS",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arch {
    X86_64,
    Aarch64,
}

impl Arch {
    pub fn current() -> Option<Self> {
        match std::env::consts::ARCH {
            "x86_64" => Some(Self::X86_64),
            "aarch64" => Some(Self::Aarch64),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installer {
    pub platform: Platform,
    #[serde(default)]
    pub arch: Vec<Arch>,
    #[serde(flatten)]
    pub kind: InstallerKind,
    #[serde(default)]
    pub run: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum InstallerKind {
    Flatpak {
        #[serde(rename = "ref")]
        flatpak_ref: String,
        #[serde(default = "default_remote")]
        remote: String,
    },
    Winget {
        package: String,
    },
    Homebrew {
        package: String,
        #[serde(default = "yes")]
        cask: bool,
    },
    Download(Download),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub format: DownloadFormat,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub github: Option<GithubRelease>,
    #[serde(default)]
    pub exec: Option<String>,
    #[serde(default = "yes")]
    pub strip_root: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadFormat {
    Appimage,
    Archive,
    Binary,
    Installer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubRelease {
    pub repo: String,
    pub asset: String,
}

fn default_remote() -> String {
    "flathub".into()
}

fn yes() -> bool {
    true
}

impl InstallerKind {
    pub fn label(&self) -> String {
        match self {
            Self::Flatpak { flatpak_ref, .. } => format!("Flatpak ({flatpak_ref})"),
            Self::Winget { package } => format!("winget ({package})"),
            Self::Homebrew { package, cask } => {
                format!(
                    "Homebrew {} ({package})",
                    if *cask { "cask" } else { "formula" }
                )
            }
            Self::Download(d) => match (&d.url, &d.github, &d.version) {
                (Some(_), _, Some(v)) => format!("Download (pinned {v})"),
                (Some(_), _, None) => "Download (pinned)".into(),
                (None, Some(gh), _) => format!("Download from GitHub releases ({})", gh.repo),
                (None, None, _) => "Direct download".into(),
            },
        }
    }
}

impl Installer {
    pub fn matches_host(&self) -> bool {
        Some(self.platform) == Platform::current()
            && (self.arch.is_empty() || Arch::current().is_some_and(|a| self.arch.contains(&a)))
    }
}

impl App {
    pub fn platforms(&self) -> Vec<Platform> {
        let mut out: Vec<Platform> = Vec::new();
        for i in &self.installers {
            if !out.contains(&i.platform) {
                out.push(i.platform);
            }
        }
        out
    }

    pub fn matches_query(&self, q: &str) -> bool {
        if q.is_empty() {
            return true;
        }
        let q = q.to_lowercase();
        [&self.name, &self.summary, &self.developer, &self.id]
            .iter()
            .any(|s| s.to_lowercase().contains(&q))
            || self.tags.iter().any(|t| t.to_lowercase().contains(&q))
    }
}

/// Load a catalog from an http(s) URL, an `index.json` file, or a checkout of the catalog repo
/// (directory with `topics.json` and `apps/*.json`). Remote catalogs are cached so the launcher
/// keeps working offline; returns the catalog and, when the cache was used, the fetch error.
pub fn load(source: &str) -> Result<(Catalog, Option<String>)> {
    if source.starts_with("http://") || source.starts_with("https://") {
        let cache = paths::cache_dir().join("index.json");
        let cache_sig = paths::cache_dir().join("index.json.minisig");
        match fetch_signed(source) {
            Ok((text, sig)) => {
                let status = verify(&text, sig.as_deref())?;
                let mut cat = parse_index(&text)?;
                cat.signature = Some(status);
                let _ = std::fs::create_dir_all(paths::cache_dir());
                let _ = std::fs::write(&cache, &text);
                let _ = match &sig {
                    Some(s) => std::fs::write(&cache_sig, s),
                    None => std::fs::remove_file(&cache_sig),
                };
                Ok((cat, None))
            }
            Err(e) => {
                let text = std::fs::read_to_string(&cache)
                    .with_context(|| format!("cannot fetch catalog ({e:#}) and no cached copy"))?;
                let sig = std::fs::read_to_string(&cache_sig).ok();
                let status = verify(&text, sig.as_deref()).context("cached catalog")?;
                let mut cat = parse_index(&text)?;
                cat.signature = Some(status);
                Ok((cat, Some(format!("{e:#}"))))
            }
        }
    } else {
        let path = Path::new(source);
        if path.is_dir() {
            Ok((load_dir(path)?, None))
        } else {
            let text =
                std::fs::read_to_string(path).with_context(|| format!("reading {source}"))?;
            Ok((parse_index(&text)?, None))
        }
    }
}

fn fetch(url: &str) -> Result<String> {
    let resp = crate::install::http()?
        .get(url)
        .send()?
        .error_for_status()?;
    Ok(resp.text()?)
}

/// Fetch the index and its detached minisign signature (`<url>.minisig`). A missing signature
/// is only tolerated when signature checking is disabled.
fn fetch_signed(url: &str) -> Result<(String, Option<String>)> {
    let text = fetch(url)?;
    let sig = match fetch(&format!("{url}.minisig")) {
        Ok(s) => Some(s),
        Err(_) if unsigned_allowed() => None,
        Err(e) => return Err(e.context("fetching the catalog signature (index.json.minisig)")),
    };
    Ok((text, sig))
}

fn unsigned_allowed() -> bool {
    std::env::var_os("PEZZOTTO_LAUNCHER_UNSIGNED_CATALOG").is_some_and(|v| v == "1")
}

/// Check the index against its minisign signature. Fails closed: a missing signature, a
/// different key or a modified index are all errors, unless PEZZOTTO_LAUNCHER_UNSIGNED_CATALOG=1.
/// PEZZOTTO_LAUNCHER_CATALOG_KEY replaces the built-in public key, for a catalog you sign yourself.
fn verify(text: &str, sig: Option<&str>) -> Result<SignatureStatus> {
    if unsigned_allowed() {
        return Ok(SignatureStatus::Skipped);
    }
    let sig = sig.ok_or_else(|| anyhow!("the catalog has no signature (index.json.minisig)"))?;
    let key_b64 = std::env::var("PEZZOTTO_LAUNCHER_CATALOG_KEY")
        .unwrap_or_else(|_| CATALOG_PUBLIC_KEY.to_string());
    let key = minisign_verify::PublicKey::from_base64(key_b64.trim())
        .map_err(|e| anyhow!("bad catalog public key: {e}"))?;
    let sig = minisign_verify::Signature::decode(sig)
        .map_err(|e| anyhow!("bad catalog signature: {e}"))?;
    key.verify(text.as_bytes(), &sig, false).map_err(|e| {
        anyhow!("the catalog signature does not verify ({e}); refusing to use this catalog")
    })?;
    Ok(SignatureStatus::Verified)
}

fn parse_index(text: &str) -> Result<Catalog> {
    let cat: Catalog = serde_json::from_str(text).context("parsing catalog index")?;
    if cat.schema_version != SCHEMA_VERSION {
        bail!(
            "catalog schema_version {} is not supported (expected {SCHEMA_VERSION}); update pezzotto.ai launcher",
            cat.schema_version
        );
    }
    Ok(cat)
}

fn load_dir(dir: &Path) -> Result<Catalog> {
    #[derive(Deserialize)]
    struct Topics {
        topics: Vec<Topic>,
    }
    let topics: Topics = serde_json::from_str(
        &std::fs::read_to_string(dir.join("topics.json")).context("reading topics.json")?,
    )?;
    let mut apps = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(dir.join("apps"))
        .context("reading apps/")?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    entries.sort();
    for p in entries {
        let app: App = serde_json::from_str(&std::fs::read_to_string(&p)?)
            .with_context(|| format!("parsing {}", p.display()))?;
        apps.push(app);
    }
    apps.sort_by_key(|a| a.name.to_lowercase());
    Ok(Catalog {
        schema_version: SCHEMA_VERSION,
        generated: None,
        topics: topics.topics,
        apps,
        signature: None,
    })
}
