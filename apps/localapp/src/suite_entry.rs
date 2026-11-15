//! The app's own entry in the Craft App Suite, as `craft-update` knows apps.
//!
//! Every app of the suite that updates itself takes its entry from the catalog at
//! `craft_update::suite` (`craft_update::suite::<YOUR ENTRY>`), which lives in the single source
//! `Anon5T4R/local-libs` (`crates/update/src/suite.rs`) and reaches this repository through
//! `scripts/sync.ps1` as a mirror under `libs/craft-libs/crates/update`. The template cannot
//! point at a catalog entry that does not exist yet, so it declares its own here - the same
//! `App` value the catalog holds for every app - and a born app moves it into the catalog:
//!
//! 1. In `underworld\local-libs` (the suite's single source), add the entry to
//!    `crates/update/src/suite.rs` **and** to `SUITE`, with the app's real id, name, tagline and
//!    repository. Bump the `SUITE` length. Run its tests.
//! 2. `powershell -File scripts/sync.ps1 -Mode push` - copies the crate into every app's
//!    `libs/craft-libs` and verifies the hash, this repository included.
//! 3. Here: replace the value below with `pub const APP: craft_update::App =
//!    craft_update::suite::<YOUR ENTRY>;` and delete this comment block's "template" notice.
//!
//! Until then the check points at this repository's own releases, which is what makes the
//! template's About window honest without inventing a suite entry nobody installs.

use craft_update::App;

/// This app, as its releases and installers know it.
pub const APP: App = App {
    publisher: "Local App contributors",
    ..App::new(
        // Machine name: repo, binary, package and asset stem (`localapp-0.1.0-windows-x64.msi`).
        "localapp",
        // Display name: window title, installer, menu entries.
        "Local App",
        // What it is for, in a few words (English); installers write it into the menu entry.
        "Desktop template of the Local suite",
        // GitHub `owner/repo`: releases are only ever downloaded from here.
        "Anon5T4R/local-app-template",
    )
};
