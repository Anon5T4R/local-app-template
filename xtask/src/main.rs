//! Workspace tooling: `cargo xtask <command>`.
//!
//! Pure Rust (std + serde_json). External tools (`cargo`, `git`) are invoked through
//! `std::process::Command`. `.cargo/config.toml` gives it the `cargo xtask` alias.

mod assets;
mod ico;
mod layers;
mod version;

use std::path::PathBuf;
use std::process::{Command, ExitCode};

const USAGE: &str = "\
usage: cargo xtask <command>

commands:
  assets          every image/icon/font file is attributed in assets/ATTRIBUTION.md; no Adobe assets
  fmt             fmt --check for the workspace's own packages (never the vendored trees in libs/)
  ico <out.ico> <in.png>...
                  pack square PNGs (<= 256 px) into a Windows .ico (see packaging/icons.sh)
  layers          enforce the crate dependency layering (see xtask/src/layers.rs)
  version [set X.Y.Z]
                  print the workspace version, or bump it (the single source) and refresh Cargo.lock

A born app adds its own commands (ci, models, parity.) next to these.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    let result = match args.first().map(String::as_str) {
        Some("ico") => ico::run(&rest),
        Some("version") => version::run(&root(), &rest),
        Some("layers") => cmd_layers(),
        Some("assets") => assets::run(&root()),
        Some("fmt") => cmd_fmt(),
        Some("-h" | "--help" | "help") | None => {
            print!("{USAGE}");
            Ok(())
        }
        Some(other) => Err(format!("unknown command `{other}`\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Workspace root (parent of the xtask crate).
pub fn root() -> PathBuf {
    // `cargo run` sets it at run time; prefer that over the compile-time value, which goes stale
    // when the checkout moves and the cached xtask binary isn't rebuilt
    let dir = std::env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from).filter(|d| d.join("Cargo.toml").is_file());
    let dir = dir.unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    dir.parent().expect("xtask has a parent dir").to_path_buf()
}

pub fn cargo() -> Command {
    let mut c = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
    c.current_dir(root());
    c
}

pub fn run(mut cmd: Command, what: &str) -> Result<(), String> {
    eprintln!("$ {what}");
    let status = cmd.status().map_err(|e| format!("{what}: failed to spawn: {e}"))?;
    if status.success() { Ok(()) } else { Err(format!("{what}: exited with {status}")) }
}

pub fn metadata() -> Result<serde_json::Value, String> {
    let out = cargo().args(["metadata", "--format-version", "1", "--no-deps"]).output().map_err(|e| format!("cargo metadata: {e}"))?;
    if !out.status.success() {
        return Err(format!("cargo metadata failed:\n{}", String::from_utf8_lossy(&out.stderr)));
    }
    serde_json::from_slice(&out.stdout).map_err(|e| format!("cargo metadata: bad JSON: {e}"))
}

fn cmd_layers() -> Result<(), String> {
    let crates = layers::from_metadata(&metadata()?)?;
    println!("Dependency layering\n");
    println!("{:<28} {:<14} workspace deps", "crate", "layer");
    for c in &crates {
        let ws: Vec<String> = c
            .deps
            .iter()
            .filter(|d| d.workspace)
            .map(|d| {
                let k = match d.kind {
                    layers::DepKind::Normal => "",
                    layers::DepKind::Dev => " (dev)",
                    layers::DepKind::Build => " (build)",
                };
                format!("{}{k}", layers::short_name(&d.name))
            })
            .collect();
        println!("{:<28} {:<14} {}", c.name, layers::describe(layers::classify(&c.name)), ws.join(", "));
    }
    let violations = layers::check(&crates);
    println!();
    if violations.is_empty() {
        println!("OK: {} crates, no layering violations.", crates.len());
        Ok(())
    } else {
        println!("{} violation(s):", violations.len());
        for v in &violations {
            println!("  - {v}");
        }
        Err(format!("{} layering violation(s)", violations.len()))
    }
}

/// The workspace's own package names, for the `fmt` gate: the vendored trees under `libs/` and
/// `assets/craft-fonts/` keep their origin formatting style, so they are never formatted here.
fn workspace_packages() -> Result<Vec<String>, String> {
    let value = metadata()?;
    let packages = value["packages"].as_array().ok_or("cargo metadata: no packages list")?;
    let mut names: Vec<String> = Vec::with_capacity(packages.len());
    for p in packages {
        let name = p["name"].as_str().ok_or("cargo metadata: a package has no name")?;
        names.push(name.to_owned());
    }
    if names.is_empty() {
        return Err("cargo metadata: the workspace lists no packages".into());
    }
    Ok(names)
}

/// `cargo xtask fmt`: formats (and only checks) the workspace's own packages. `cargo fmt --all`
/// would also reformat the vendored trees under `libs/` in this repository's style, which is not
/// ours to change; the CI runs this same command, so the gate and the local check cover the same
/// files.
fn cmd_fmt() -> Result<(), String> {
    let members = workspace_packages()?;
    let mut c = cargo();
    c.arg("fmt");
    for name in &members {
        c.args(["-p", name]);
    }
    c.args(["--", "--check"]);
    run(c, "cargo fmt -p <workspace members> -- --check")
}
