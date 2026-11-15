# Interface catalogs

Translations of the app's English source text, one directory per locale, read by `craft-i18n`
(vendored in `libs/craft-libs/crates/i18n`) and embedded by `apps/localapp/build.rs`.

```
locales/<code>/localapp.tsv   # this app's strings
locales/<code>/common.tsv     # optional: strings shared with the rest of the suite
```

Format, one row per line, columns separated by a TAB: `context ⇥ English source ⇥ translation`.
The header comment of each catalog documents the details (`@id`, `@plural`, `@msg`, escapes,
placeholders, and the rule that a trailing "." must appear in both columns).

- A missing translation shows the English text; coverage grows one row at a time and never fails a
  build, a test or a release. `apps/localapp/src/i18n.rs` has a test that every embedded catalog
  parses (`I18n::check_all`).
- A new language: add `locales/<code>/localapp.tsv` (with its `@id language.name` row) and one row
  to `LANGUAGES` in `apps/localapp/src/i18n.rs`. The picker and the system-language match pick it up.
- Clean room: write every row from the plain meaning of the English text. Do not copy another
  product's localisation or string table; `assets/ATTRIBUTION.md` records who translated what.

The catalogs are our own work (the three seeded locales are part of the template); they are not a
copy of the upstream craft-i18n data repository.
