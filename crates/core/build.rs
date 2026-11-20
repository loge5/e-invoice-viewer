//! Compiles the translations in `po/` into binary catalogues that are
//! embedded into the program (ADR 0003): every `po/<code>.po` becomes
//! `<code>.mo`, listed in the generated `catalogues.rs`. English is the
//! source language and has no catalogue.

use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

fn main() {
    let po_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../po");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    println!("cargo::rerun-if-changed={}", po_dir.display());
    let mut codes: Vec<String> = std::fs::read_dir(&po_dir)
        .expect("po directory")
        .map(|entry| entry.expect("po entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "po"))
        .map(|path| {
            path.file_stem()
                .expect("file name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    codes.sort();

    let mut list = String::from("pub(crate) const CATALOGUES: &[(&str, &[u8])] = &[\n");
    for code in &codes {
        let po = po_dir.join(format!("{code}.po"));
        let mo = Path::new(&out_dir).join(format!("{code}.mo"));
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
        writeln!(
            list,
            "    ({code:?}, include_bytes!({:?})),",
            mo.display().to_string()
        )
        .expect("write to string");
    }
    list.push_str("];\n");
    std::fs::write(Path::new(&out_dir).join("catalogues.rs"), list).expect("write catalogues.rs");
}
