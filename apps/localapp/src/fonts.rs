//! The craft-fonts build input (vendored in `assets/craft-fonts`, recipe from its
//! `docs/integration.md`), registered with egui at start-up.
//!
//! egui's bundled fonts cover Latin, Greek and Cyrillic; the craft-fonts faces add the CJK,
//! Arabic, Hebrew, Devanagari (.) scripts the rest of the suite shows. They are appended as
//! fallbacks, so Latin text keeps the bundled face and a glyph the bundled face lacks comes from
//! here. Without the checkout the app still builds and runs: those scripts show boxes
//! (`CRAFT_FONTS_REQUIRED=1` fails a release build instead, see `build.rs`).

/// One row of the craft-fonts `fonts/manifest.txt`: family, style, the scripts it covers, and the
/// font file itself, embedded at build time.
pub struct CraftFont {
    pub family: &'static str,
    pub style: &'static str,
    pub scripts: &'static [&'static str],
    pub bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/craft_fonts.rs"));

/// Installs the embedded fonts into `ctx`. Call once, before the first frame.
pub fn install(ctx: &egui::Context) {
    if CRAFT_FONTS.is_empty() {
        log::info!("no craft-fonts: CJK, Arabic, Hebrew and Devanagari text shows boxes (set CRAFT_FONTS_DIR)");
        return;
    }
    let mut fonts = egui::FontDefinitions::default();
    for (i, font) in CRAFT_FONTS.iter().enumerate() {
        // The name egui knows the face by: unique per manifest row (one family has several styles).
        let name = format!("craft:{}-{}-{i}", font.family, font.style);
        fonts.font_data.insert(name.clone(), std::sync::Arc::new(egui::FontData::from_static(font.bytes)));
        for family in families(font.scripts) {
            // Appended: these are fallbacks, never the first face of the family.
            fonts.families.entry(family).or_default().push(name.clone());
        }
    }
    log::info!("{} craft-fonts face(s) registered", CRAFT_FONTS.len());
    ctx.set_fonts(fonts);
}

/// The egui families a face belongs in: everything that draws text needs it as a fallback, and a
/// face covering an East Asian script is wanted for monospace too (code, tables of numbers).
fn families(scripts: &[&str]) -> Vec<egui::FontFamily> {
    let mut out = vec![egui::FontFamily::Proportional];
    let cjk = scripts.iter().any(|s| matches!(*s, "Jpan" | "Hans" | "Hant" | "Kore"));
    if cjk {
        out.push(egui::FontFamily::Monospace);
    }
    out
}
