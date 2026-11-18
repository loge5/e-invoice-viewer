//! Compiles the translations in `po/` into binary catalogues that are
//! embedded into the program (ADR 0003).

use std::path::Path;
use std::process::Command;

/// Languages with a catalogue; English is the source language.
const LANGUAGES: [&str; 1] = ["de"];

fn main() {
    let po_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../po");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    for language in LANGUAGES {
        let po = po_dir.join(format!("{language}.po"));
        let mo = Path::new(&out_dir).join(format!("{language}.mo"));
        println!("cargo::rerun-if-changed={}", po.display());
        let status = Command::new("msgfmt")
            .arg("--check")
            .arg("-o")
            .arg(&mo)
            .arg(&po)
            .status()
            .unwrap_or_else(|error| {
                panic!("msgfmt (GNU gettext) is needed to build the translations: {error}")
            });
        assert!(status.success(), "msgfmt failed for {}", po.display());
    }
}
