//! Build input of the app binary. Three optional/conditional outputs, all baked into the binary:
//!
//! 1. **Windows resources** (icon + VERSIONINFO), so Explorer, the taskbar, the Start menu and
//!    Alt-Tab show the app icon and the installer can read the version. No-op off Windows. A
//!    missing resource compiler is a warning, so a cross-compile still links, unless
//!    `LOCALAPP_REQUIRE_WINRES=1` turns it into an error (release builds).
//! 2. **`craft_i18n.rs`**: `CRAFT_I18N`, the (locale, shared catalog, app catalog) list
//!    `craft-i18n` reads. From `CRAFT_I18N_DIR` (the in-repo `i18n/` by default).
//!    `CRAFT_I18N_REQUIRED=1` (releases) makes a missing catalog set an error.
//! 3. **`craft_fonts.rs`**: `CRAFT_FONTS`, one entry per font of a craft-fonts checkout
//!    (`CRAFT_FONTS_DIR`, the in-repo `assets/craft-fonts/` by default; recipe from craft-fonts'
//!    `docs/integration.md`). Unset/absent: empty, and CJK text shows boxes.
//!    `CRAFT_FONTS_REQUIRED=1` fails the build when a CJK interface language has no face.
//!
//! Nothing is fetched: every input is a file of this repository or of the given checkout.

use std::fmt::Write as _;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    windows_resources();
    write_out("craft_i18n.rs", &craft_i18n());
    write_out("craft_fonts.rs", &craft_fonts());
}

fn out_dir() -> PathBuf {
    PathBuf::from(std::env::var_os("OUT_DIR").unwrap_or_default())
}

fn write_out(name: &str, contents: &str) {
    let out = out_dir().join(name);
    if let Err(e) = std::fs::write(&out, contents) {
        println!("cargo::error=writing {}: {e}", out.display());
    }
}

fn env_flag(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| v.trim() == "1")
}

/// The workspace root (the app crate lives in `<root>/apps/<id>`): where a relative build input is
/// resolved from, so `CRAFT_FONTS_DIR=assets/craft-fonts` means the same everywhere.
fn workspace_root() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `dir`, or `dir` resolved from the workspace root when it is relative.
fn absolute(dir: PathBuf) -> PathBuf {
    if dir.is_absolute() { dir } else { workspace_root().join(dir) }
}

// ---- Windows resources ---------------------------------------------------------------------------

fn windows_resources() {
    println!("cargo:rerun-if-changed=../../assets/app-icon/localapp.ico");
    println!("cargo:rerun-if-env-changed=LOCALAPP_REQUIRE_WINRES");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("../../assets/app-icon/localapp.ico")
        .set("ProductName", "Local App")
        .set("FileDescription", "Local App - template of the Local suite")
        .set("LegalCopyright", "Copyright (c) the Local App authors. MIT OR Apache-2.0.")
        .set("OriginalFilename", "localapp.exe")
        .set("InternalName", "localapp");
    if let Err(e) = res.compile() {
        if std::env::var_os("LOCALAPP_REQUIRE_WINRES").is_some() {
            panic!("embedding Windows resources failed: {e}");
        }
        println!("cargo:warning=localapp.exe built without icon/version resources: {e}");
    }
}

// ---- i18n catalogs -------------------------------------------------------------------------------

/// This app's catalog in each locale: `locales/<locale>/localapp.tsv`.
const APP_CATALOG: &str = "localapp.tsv";

/// `CRAFT_I18N` from `CRAFT_I18N_DIR` (the in-repo `i18n/` by default), the `Embedded` list the
/// craft-i18n crate reads.
fn craft_i18n() -> String {
    println!("cargo::rerun-if-env-changed=CRAFT_I18N_DIR");
    println!("cargo::rerun-if-env-changed=CRAFT_I18N_REQUIRED");
    let required = env_flag("CRAFT_I18N_REQUIRED");
    let default_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../i18n");
    let entries = match std::env::var_os("CRAFT_I18N_DIR").filter(|d| !d.is_empty()) {
        // Canonicalizing is avoided: on Windows it gives `\\?\C:\...`, which `include_str!` can't read.
        Some(dir) => match locales(&absolute(PathBuf::from(dir)).join("locales")) {
            Ok(entries) => entries,
            Err(e) => {
                let level = if required { "error" } else { "warning" };
                println!("cargo::{level}=CRAFT_I18N_DIR: {e}; the interface is English only");
                Vec::new()
            }
        },
        None => match locales(&default_dir.join("locales")) {
            Ok(entries) => entries,
            Err(e) => {
                let level = if required { "error" } else { "warning" };
                println!("cargo::{level}=i18n catalogs: {e}; the interface is English only");
                Vec::new()
            }
        },
    };
    let mut o = String::from("/// (locale, shared catalog, this app's catalog), one entry per locale; `\"\"` for a missing file.\n");
    o.push_str("pub static CRAFT_I18N: &craft_i18n::Embedded = &[\n");
    for (code, shared, app) in entries {
        // `{:?}` escapes the path (Windows backslashes) as a Rust string literal.
        let shared = shared.map_or_else(|| "\"\"".to_string(), |p| format!("include_str!({p:?})"));
        let _ = writeln!(o, "    ({code:?}, {shared}, include_str!({app:?})),");
    }
    o.push_str("];\n");
    o
}

/// Every locale under `dir` with this app's catalog, by code: (code, its `common.tsv` if any, its
/// app catalog). An error when `dir` isn't a craft-i18n `locales/` directory.
#[allow(clippy::type_complexity)]
fn locales(dir: &std::path::Path) -> Result<Vec<(String, Option<PathBuf>, PathBuf)>, String> {
    // Build scripts run in the crate's directory and `include_str!` resolves against `OUT_DIR`, so
    // a relative path would mean two different places.
    if !dir.is_absolute() {
        return Err(format!("{} is not an absolute path", dir.display()));
    }
    let list = std::fs::read_dir(dir).map_err(|e| format!("{} is not a craft-i18n checkout ({e})", dir.display()))?;
    println!("cargo::rerun-if-changed={}", dir.display());
    let mut out = Vec::new();
    for entry in list.flatten() {
        let path = entry.path();
        let Some(code) = path.file_name().and_then(std::ffi::OsStr::to_str).map(str::to_string) else { continue };
        let app = path.join(APP_CATALOG);
        if !app.is_file() {
            continue;
        }
        println!("cargo::rerun-if-changed={}", app.display());
        let shared = Some(path.join("common.tsv")).filter(|p| p.is_file());
        if let Some(p) = &shared {
            println!("cargo::rerun-if-changed={}", p.display());
        }
        out.push((code, shared, app));
    }
    if out.is_empty() {
        return Err(format!("{} has no locale with a {APP_CATALOG}", dir.display()));
    }
    out.sort();
    Ok(out)
}

// ---- craft-fonts ---------------------------------------------------------------------------------

/// Scripts a release build (`CRAFT_FONTS_REQUIRED`) must embed a font for: the CJK interface
/// languages this app offers (Japanese, Simplified Chinese).
const REQUIRED_SCRIPTS: &[&str] = &["Jpan", "Hans"];

/// `CRAFT_FONTS` from a craft-fonts checkout: the in-repo `assets/craft-fonts/` by default,
/// `CRAFT_FONTS_DIR` overriding it (what the packaging scripts export).
fn craft_fonts() -> String {
    println!("cargo::rerun-if-env-changed=CRAFT_FONTS_DIR");
    println!("cargo::rerun-if-env-changed=CRAFT_FONTS_REQUIRED");
    let dir = match std::env::var_os("CRAFT_FONTS_DIR").filter(|d| !d.is_empty()) {
        Some(dir) => absolute(PathBuf::from(dir)),
        None => workspace_root().join("assets/craft-fonts"),
    };
    let mut src = String::from("pub static CRAFT_FONTS: &[CraftFont] = &[\n");
    match fonts(&dir) {
        Ok((entries, covered)) => {
            src.push_str(&entries);
            // Each CJK interface language needs its own face: without the Chinese one, Chinese
            // menus fall back to the Japanese face and show boxes for simplified hanzi.
            if env_flag("CRAFT_FONTS_REQUIRED") {
                for s in REQUIRED_SCRIPTS.iter().filter(|s| !covered.iter().any(|c| c == *s)) {
                    println!("cargo::error=CRAFT_FONTS_DIR={}: no {s} font in fonts/manifest.txt", dir.display());
                }
            }
        }
        Err(e) if env_flag("CRAFT_FONTS_REQUIRED") => println!("cargo::error=CRAFT_FONTS_DIR={}: {e}", dir.display()),
        Err(e) => println!("cargo::warning=building without craft-fonts: {}: {e}", dir.display()),
    }
    src.push_str("];\n");
    src
}

/// One `CraftFont { .. }` initialiser per manifest line, and the scripts the embedded fonts cover.
fn fonts(dir: &std::path::Path) -> Result<(String, Vec<String>), String> {
    let manifest = dir.join("fonts/manifest.txt");
    println!("cargo::rerun-if-changed={}", manifest.display());
    let text = std::fs::read_to_string(&manifest).map_err(|e| format!("{}: {e}", manifest.display()))?;
    let (mut out, mut covered) = (String::new(), Vec::new());
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let f: Vec<&str> = line.split(" | ").map(str::trim).collect();
        let [family, style, file, scripts, ..] = f.as_slice() else {
            return Err(format!("malformed manifest line: {line}"));
        };
        // No `canonicalize()`: on Windows it yields `\\?\C:\...`, which `include_bytes!` does not
        // always read; `dir` is already absolute here.
        let path = absolute(dir.join(file));
        if !path.is_file() {
            return Err(format!("{file}: no such file in {}", dir.display()));
        }
        println!("cargo::rerun-if-changed={}", path.display());
        covered.extend(scripts.split(',').map(|s| s.trim().to_string()));
        let scripts: Vec<String> = scripts.split(',').map(|s| format!("{:?}", s.trim())).collect();
        let _ = writeln!(
            out,
            "    CraftFont {{ family: {family:?}, style: {style:?}, scripts: &[{}], bytes: include_bytes!({:?}) }},",
            scripts.join(", "),
            path.display().to_string(),
        );
    }
    Ok((out, covered))
}
