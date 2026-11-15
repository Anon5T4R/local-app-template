//! Dependency layering rules.
//!
//! The suite grows every app as layers: `localapp-<name>` crates only ever depend downwards, so
//! the engine can be tested without the UI and no lower layer drags egui in. The rule engine works
//! on a small, metadata-independent model so it can be unit-tested; `from_metadata` builds that
//! model from `cargo metadata`.
//!
//! A born app fills [`TABLE`] and [`INTRA_LAYER_ORDER`] with its own crates as it adds them. With
//! only the binary, the table holds just the app and the tooling, and the gate passes trivially.

use serde_json::Value;

/// Where a workspace crate sits.
// A born app registers its crates in [`TABLE`] and every class becomes reachable; with only the
// binary, the table holds one `Exempt` entry each and the rest is dead code.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Regular layered crate.
    Layer(u8),
    /// Layer 0, and additionally may depend on no workspace crate at all.
    Standalone,
    /// Test tooling: may depend on anything up to L5 (it sits at L6 for rule purposes); other
    /// crates may use it only as a dev-dependency.
    Testkit,
    /// Binaries and build tooling: exempt from the rules.
    Exempt,
}

impl Class {
    fn layer(self) -> Option<u8> {
        match self {
            Class::Layer(l) => Some(l),
            Class::Standalone => Some(0),
            Class::Testkit => Some(5),
            Class::Exempt => None,
        }
    }
}

/// The layering table. Names are package names without the `localapp-` prefix.
pub const TABLE: &[(&str, Class)] = &[
    // L6 apps and tooling
    ("localapp", Class::Exempt),
    ("xtask", Class::Exempt),
    // A born app adds its engine crates here as it writes them, e.g.:
    // ("core", Class::Layer(0)),
    // ("engine", Class::Layer(3)),
];

/// Explicit orderings *within* a layer (earlier may be used by later).
pub const INTRA_LAYER_ORDER: &[&[&str]] = &[
    // e.g. &["geom", "color", "raster"],
];

/// The package-name prefix of the app's own crates. A born app renames it with the rest of the
/// template (`scripts/nascer-app.ps1`).
pub const PREFIX: &str = "localapp-";

fn intra_layer_allowed(from: &str, to: &str) -> bool {
    let (from, to) = (short_name(from), short_name(to));
    INTRA_LAYER_ORDER.iter().any(|chain| match (chain.iter().position(|n| *n == from), chain.iter().position(|n| *n == to)) {
        (Some(f), Some(t)) => t < f,
        _ => false,
    })
}

/// External crates that constitute a UI toolkit / windowing dependency. Entries ending in `*` are
/// prefixes.
pub const UI_CRATES: &[&str] = &["craft-ui", "egui", "eframe", "winit", "egui_kittest", "rfd", "bevy*"];

/// First layer allowed to use UI crates.
pub const UI_MIN_LAYER: u8 = 5;

pub fn short_name(pkg: &str) -> &str {
    pkg.strip_prefix(PREFIX).unwrap_or(pkg)
}

pub fn classify(pkg: &str) -> Option<Class> {
    classify_in(TABLE, pkg)
}

/// [`classify`] against an explicit table, so a test can extend it the way a born app does.
pub fn classify_in(table: &[(&'static str, Class)], pkg: &str) -> Option<Class> {
    let s = short_name(pkg);
    table.iter().find(|(n, _)| *n == s).map(|(_, c)| *c)
}

fn is_ui_crate(name: &str) -> bool {
    UI_CRATES.iter().any(|p| match p.strip_suffix('*') {
        Some(prefix) => name.starts_with(prefix),
        None => name == *p,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepKind {
    Normal,
    Dev,
    Build,
}

#[derive(Debug, Clone)]
pub struct Dep {
    pub name: String,
    pub kind: DepKind,
    /// `true` if the dependency is a workspace member.
    pub workspace: bool,
}

#[derive(Debug, Clone)]
pub struct Crate {
    pub name: String,
    pub deps: Vec<Dep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    Unregistered { krate: String },
    Upward { krate: String, dep: String, from: u8, to: u8, kind: DepKind },
    StandaloneHasWorkspaceDep { krate: String, dep: String },
    TestkitAsNormalDep { krate: String },
    UiBelowL5 { krate: String, dep: String, layer: u8 },
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Violation::Unregistered { krate } => {
                write!(f, "{krate}: unknown workspace crate; register it in xtask/src/layers.rs TABLE")
            }
            Violation::Upward { krate, dep, from, to, kind } => {
                write!(f, "{krate} (L{from}) -> {dep} (L{to}) [{kind:?}]: may only depend on strictly lower layers")
            }
            Violation::StandaloneHasWorkspaceDep { krate, dep } => {
                write!(f, "{krate}: standalone crate must not depend on workspace crate {dep}")
            }
            Violation::TestkitAsNormalDep { krate } => {
                write!(f, "{krate}: test tooling may only be a dev-dependency")
            }
            Violation::UiBelowL5 { krate, dep, layer } => {
                write!(f, "{krate} (L{layer}) depends on UI crate `{dep}`; UI toolkits are only allowed in L5+")
            }
        }
    }
}

/// Check all rules. Returns violations sorted for stable output.
pub fn check(crates: &[Crate]) -> Vec<Violation> {
    check_with(crates, TABLE)
}

/// [`check`] against an explicit table (a born app's table, in a test).
pub fn check_with(crates: &[Crate], table: &[(&'static str, Class)]) -> Vec<Violation> {
    let mut out = Vec::new();
    for c in crates {
        let Some(class) = classify_in(table, &c.name) else {
            out.push(Violation::Unregistered { krate: c.name.clone() });
            continue;
        };
        if class == Class::Exempt {
            continue;
        }
        let layer = class.layer().unwrap_or(0);
        for d in &c.deps {
            // Self dev-dependencies (e.g. to enable features in tests) are fine.
            if d.name == c.name {
                continue;
            }
            if d.workspace {
                if class == Class::Standalone {
                    out.push(Violation::StandaloneHasWorkspaceDep { krate: c.name.clone(), dep: d.name.clone() });
                    continue;
                }
                match classify_in(table, &d.name) {
                    // Unregistered deps are reported on their own entry.
                    None => {}
                    Some(Class::Testkit) if d.kind != DepKind::Dev => out.push(Violation::TestkitAsNormalDep { krate: c.name.clone() }),
                    Some(Class::Testkit) => {}
                    Some(dc) => {
                        let to = dc.layer().unwrap_or(u8::MAX);
                        if to >= layer && !(to == layer && intra_layer_allowed(&c.name, &d.name)) {
                            out.push(Violation::Upward {
                                krate: c.name.clone(),
                                dep: d.name.clone(),
                                from: layer,
                                to: if to == u8::MAX { 7 } else { to },
                                kind: d.kind,
                            });
                        }
                    }
                }
            } else if layer < UI_MIN_LAYER && is_ui_crate(&d.name) {
                out.push(Violation::UiBelowL5 { krate: c.name.clone(), dep: d.name.clone(), layer });
            }
        }
    }
    out.sort_by_key(|v| v.to_string());
    out.dedup();
    out
}

/// Build the model from `cargo metadata --format-version 1 --no-deps`.
pub fn from_metadata(meta: &Value) -> Result<Vec<Crate>, String> {
    let pkgs = meta["packages"].as_array().ok_or("metadata: no packages array")?;
    let members: Vec<&str> = pkgs.iter().filter_map(|p| p["name"].as_str()).collect();
    let mut out = Vec::new();
    for p in pkgs {
        let name = p["name"].as_str().ok_or("package without name")?.to_owned();
        let mut deps = Vec::new();
        for d in p["dependencies"].as_array().into_iter().flatten() {
            let dname = d["name"].as_str().unwrap_or_default().to_owned();
            let kind = match d["kind"].as_str() {
                Some("dev") => DepKind::Dev,
                Some("build") => DepKind::Build,
                _ => DepKind::Normal,
            };
            let workspace = members.contains(&dname.as_str()) || d["path"].is_string();
            deps.push(Dep { name: dname, kind, workspace });
        }
        out.push(Crate { name, deps });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn describe(class: Option<Class>) -> String {
    match class {
        Some(Class::Layer(l)) => format!("L{l}"),
        Some(Class::Standalone) => "L0 standalone".into(),
        Some(Class::Testkit) => "testkit".into(),
        Some(Class::Exempt) => "exempt".into(),
        None => "UNREGISTERED".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use DepKind::*;

    fn c(name: &str, deps: &[(&str, DepKind, bool)]) -> Crate {
        Crate { name: name.into(), deps: deps.iter().map(|(n, k, w)| Dep { name: (*n).into(), kind: *k, workspace: *w }).collect() }
    }

    /// The table this template ships with: only the app and the tooling, so the gate passes.
    #[test]
    fn the_shipped_table_is_clean() {
        let g = [c("localapp", &[("egui", Normal, false), ("craft-update", Normal, false)]), c("xtask", &[("serde_json", Normal, false)])];
        assert!(check(&g).is_empty(), "{:?}", check(&g));
    }

    #[test]
    fn an_unregistered_workspace_crate_is_flagged() {
        let v = check(&[c("localapp-mystery", &[])]);
        assert!(matches!(&v[..], [Violation::Unregistered { krate }] if krate == "localapp-mystery"));
    }

    /// The rules a born app relies on, with its own crates registered.
    #[test]
    fn upward_ui_below_l5_and_testkit_rules() {
        let table: &[(&'static str, Class)] = &[
            ("geom", Class::Layer(0)),
            ("engine", Class::Layer(3)),
            ("ui", Class::Layer(5)),
            ("testkit", Class::Testkit),
            ("localapp", Class::Exempt),
            ("xtask", Class::Exempt),
        ];
        let g = [
            c("localapp-geom", &[]),
            c("localapp-engine", &[("localapp-geom", Normal, true)]),
            c("localapp-ui", &[("localapp-engine", Normal, true), ("egui", Normal, false)]),
            c("localapp", &[("localapp-ui", Normal, true), ("localapp-testkit", Dev, true)]),
        ];
        assert!(check_with(&g, table).is_empty(), "{:?}", check_with(&g, table));

        // engine -> ui goes up
        let up = check_with(&[c("localapp-engine", &[("localapp-ui", Normal, true)])], table);
        assert!(matches!(up[..], [Violation::Upward { from: 3, to: 5, .. }]), "{up:?}");

        // a UI toolkit below L5
        let ui = check_with(&[c("localapp-engine", &[("egui", Normal, false)])], table);
        assert!(matches!(ui[..], [Violation::UiBelowL5 { layer: 3, .. }]), "{ui:?}");

        // test tooling as a normal dependency
        let tk = check_with(&[c("localapp-engine", &[("localapp-testkit", Normal, true)])], table);
        assert!(matches!(tk[..], [Violation::TestkitAsNormalDep { .. }]), "{tk:?}");
    }
}
