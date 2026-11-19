//! On Windows, embeds the app icon and version information into the
//! executable (spec 0022, ADR 0005), with `windres` from MinGW. Nothing
//! happens for other targets.

use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let icon = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/icons/de.loges.EInvoiceViewer.ico")
        .canonicalize()
        .expect("data/icons/de.loges.EInvoiceViewer.ico");
    println!("cargo::rerun-if-changed={}", icon.display());
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let version = env!("CARGO_PKG_VERSION");
    let numbers: Vec<&str> = version.split(['.', '-']).take(3).collect();
    let comma_version = format!("{},0", numbers.join(","));

    // windres reads forward slashes on Windows as well.
    let icon_path = icon.display().to_string().replace('\\', "/");
    let rc = Path::new(&out_dir).join("app.rc");
    std::fs::write(
        &rc,
        format!(
            r#"1 ICON "{icon_path}"
1 VERSIONINFO
FILEVERSION {comma_version}
PRODUCTVERSION {comma_version}
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "FileDescription", "E-Invoice Viewer"
      VALUE "ProductName", "E-Invoice Viewer"
      VALUE "FileVersion", "{version}"
      VALUE "ProductVersion", "{version}"
      VALUE "OriginalFilename", "e-invoice-viewer.exe"
      VALUE "LegalCopyright", "GPL-3.0-or-later"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
        ),
    )
    .expect("write app.rc");

    let object = Path::new(&out_dir).join("app-resource.o");
    let windres = std::env::var("WINDRES").unwrap_or_else(|_| "windres".to_string());
    let status = Command::new(&windres)
        .arg(&rc)
        .args(["-O", "coff", "-o"])
        .arg(&object)
        .status()
        .unwrap_or_else(|error| panic!("{windres} (MinGW binutils) is needed: {error}"));
    assert!(status.success(), "{windres} failed");
    println!("cargo::rustc-link-arg-bins={}", object.display());
}
