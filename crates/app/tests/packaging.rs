//! Spec 0020 — desktop metadata, icon, install script and AppImage build
//! scripts; spec 0022 — Windows icon, installer script and the release
//! workflow for both platforms. Needs `desktop-file-validate` and
//! `appstreamcli`, like the catalogue test needs `xgettext`. Linux only:
//! these are checks of the Linux packaging and of files read by the build.
#![cfg(target_os = "linux")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const APP_ID: &str = "de.loges.EInvoiceViewer";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn read(path: &str) -> String {
    fs::read_to_string(repo().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn run(command: &mut Command) -> Output {
    command.output().expect("command must be installed")
}

fn assert_success(output: &Output, what: &str) {
    assert!(
        output.status.success(),
        "{what}:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn desktop_entry() -> String {
    read(&format!("data/{APP_ID}.desktop"))
}

fn workspace_version() -> String {
    let manifest = read("Cargo.toml");
    let section = manifest
        .split("[workspace.package]")
        .nth(1)
        .expect("[workspace.package]");
    section
        .lines()
        .find_map(|line| line.strip_prefix("version = \""))
        .and_then(|rest| rest.strip_suffix('"'))
        .expect("version")
        .to_string()
}

/// A fresh, empty directory below the test's target directory.
fn temp_dir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

// AC5
#[test]
fn desktop_entry_is_valid_and_complete() {
    let path = repo().join(format!("data/{APP_ID}.desktop"));
    assert_success(
        &run(Command::new("desktop-file-validate").arg(&path)),
        "desktop-file-validate",
    );
    let entry = desktop_entry();
    for line in [
        "Name=E-Invoice Viewer",
        "GenericName=Invoice viewer",
        "GenericName[de]=Rechnungsbetrachter",
        "Comment=View and check electronic invoices (ZUGFeRD, Factur-X, XRechnung)",
        "Comment[de]=Elektronische Rechnungen (ZUGFeRD, Factur-X, XRechnung) ansehen und prüfen",
        "Exec=e-invoice-viewer %F",
        "Icon=de.loges.EInvoiceViewer",
        "MimeType=application/pdf;application/xml;text/xml;",
    ] {
        assert!(entry.lines().any(|l| l == line), "missing: {line}");
    }
}

// AC5
#[test]
fn metainfo_is_valid_and_matches_the_app() {
    let path = repo().join(format!("data/{APP_ID}.metainfo.xml"));
    assert_success(
        &run(Command::new("appstreamcli")
            .args(["validate", "--no-net", "--no-color"])
            .arg(&path)),
        "appstreamcli validate",
    );
    let metainfo = read(&format!("data/{APP_ID}.metainfo.xml"));
    for wanted in [
        format!("<id>{APP_ID}</id>"),
        format!("<launchable type=\"desktop-id\">{APP_ID}.desktop</launchable>"),
        format!("<release version=\"{}\"", workspace_version()),
        "<project_license>GPL-3.0-or-later</project_license>".to_string(),
        "<summary xml:lang=\"de\">".to_string(),
        "<p xml:lang=\"de\">".to_string(),
        "<binary>e-invoice-viewer</binary>".to_string(),
    ] {
        assert!(metainfo.contains(&wanted), "missing: {wanted}");
    }
}

// AC6
#[test]
fn icon_is_a_square_svg_named_after_the_app() {
    let icon = read(&format!("data/icons/{APP_ID}.svg"));
    let document = roxmltree::Document::parse(&icon).expect("well-formed SVG");
    let svg = document.root_element();
    assert_eq!(svg.tag_name().name(), "svg");
    let view_box: Vec<f64> = svg
        .attribute("viewBox")
        .expect("viewBox")
        .split_whitespace()
        .map(|n| n.parse().expect("number"))
        .collect();
    assert_eq!(view_box.len(), 4);
    assert_eq!(view_box[2], view_box[3], "square");
}

// AC6
#[test]
fn install_script_installs_entry_and_icon_and_removes_them() {
    let home = temp_dir("install-home");
    let binary_source = home.join("built-binary");
    fs::write(&binary_source, "#!/bin/sh\n").expect("binary");
    let script = repo().join("scripts/install.sh");
    let install = |args: &[&str]| {
        run(Command::new("bash")
            .arg(&script)
            .args(args)
            .env("SOURCE_BINARY", &binary_source)
            .env("BIN_DIR", home.join("bin"))
            .env("XDG_DATA_HOME", home.join("share")))
    };
    // An icon theme directory that exists already, as on a used desktop.
    let hicolor = home.join("share/icons/hicolor");
    fs::create_dir_all(hicolor.join("scalable/apps")).expect("hicolor");
    let old = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000);
    fs::File::open(&hicolor)
        .and_then(|dir| dir.set_modified(old))
        .expect("set mtime");
    let mtime = |path: &Path| {
        fs::metadata(path)
            .and_then(|m| m.modified())
            .expect("mtime")
    };
    assert_success(&install(&[]), "install");

    let binary = home.join("bin/e-invoice-viewer");
    let entry_path = home.join(format!("share/applications/{APP_ID}.desktop"));
    let icon = home.join(format!("share/icons/hicolor/scalable/apps/{APP_ID}.svg"));
    assert!(binary.is_file());
    assert_eq!(
        fs::read(&icon).expect("icon"),
        fs::read(repo().join(format!("data/icons/{APP_ID}.svg"))).expect("data icon")
    );
    // The theme directory itself must change, so that the desktop rescans it.
    assert!(mtime(&hicolor) > old, "hicolor not touched");
    let installed = fs::read_to_string(&entry_path).expect("desktop entry");
    let expected = desktop_entry().replace(
        "Exec=e-invoice-viewer %F",
        &format!("Exec=\"{}\" %F", binary.display()),
    );
    assert_eq!(installed, expected);

    assert_success(&install(&["--uninstall"]), "uninstall");
    assert!(!binary.exists());
    assert!(!entry_path.exists());
    assert!(!icon.exists());
}

// AC1
#[test]
fn download_with_wrong_checksum_stops_and_leaves_nothing() {
    let dir = temp_dir("fetch");
    let source = dir.join("tool");
    fs::write(&source, "pretend tool").expect("source");
    let dest = dir.join("downloaded");
    let fetch = |sha256: &str| {
        run(Command::new("bash")
            .arg("-c")
            .arg(r#"set -e; . "$1"; fetch some-tool "file://$2" "$3" "$4""#)
            .arg("-")
            .arg(repo().join("scripts/appimage/fetch.sh"))
            .arg(&source)
            .arg(sha256)
            .arg(&dest))
    };

    let wrong = fetch(&"0".repeat(64));
    assert!(!wrong.status.success());
    let message = String::from_utf8_lossy(&wrong.stderr);
    assert!(message.contains("wrong SHA-256 for some-tool"), "{message}");
    assert!(!dest.exists());
    assert!(!dest.with_extension("part").exists());

    let digest = run(Command::new("sha256sum").arg(&source));
    let actual = String::from_utf8_lossy(&digest.stdout)
        .split_whitespace()
        .next()
        .expect("digest")
        .to_string();
    assert_success(&fetch(&actual), "fetch with the right checksum");
    assert_eq!(fs::read(&dest).expect("downloaded"), b"pretend tool");
}

// AC1
#[test]
fn every_download_has_a_checksum() {
    let dockerfile = read("scripts/appimage/Dockerfile");
    assert!(
        dockerfile.contains("FROM ubuntu:24.04@sha256:"),
        "image pinned by digest"
    );
    let lines: Vec<&str> = dockerfile.lines().map(str::trim).collect();
    let fetches: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains("fetch ") && !line.contains("fetch.sh"))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(fetches.len(), 3, "rustup, linuxdeploy, plugin");
    for i in fetches {
        let url = lines[i + 1];
        let checksum = lines[i + 2].split_whitespace().next().expect("checksum");
        assert!(url.starts_with("https://"), "{url}");
        assert!(
            checksum.len() == 64 && checksum.chars().all(|c| c.is_ascii_hexdigit()),
            "checksum after {url}"
        );
    }
    for file in [
        "scripts/appimage/Dockerfile",
        "scripts/appimage/build.sh",
        "scripts/appimage.sh",
    ] {
        assert!(
            !read(file).lines().any(|line| {
                let line = line.trim();
                !line.starts_with('#') && (line.contains("curl -") || line.contains("wget "))
            }),
            "{file} downloads outside fetch"
        );
    }
}

// AC9
#[test]
fn tag_must_match_the_version() {
    let check = |tag: &str| {
        run(Command::new("bash")
            .arg(repo().join("scripts/appimage/check-tag.sh"))
            .arg(tag))
        .status
        .success()
    };
    assert!(check(&format!("v{}", workspace_version())));
    assert!(!check("v9.9.9"));
    assert!(!check(&workspace_version()));
    assert!(!check(""));
}

/// The text of one job of the release workflow, up to the next job.
fn job<'a>(workflow: &'a str, name: &str) -> &'a str {
    let rest = workflow
        .split(&format!("\n  {name}:\n"))
        .nth(1)
        .unwrap_or_else(|| panic!("job {name}"));
    let end = rest
        .match_indices("\n  ")
        .find(|(index, _)| {
            let line = &rest[index + 3..];
            !line.starts_with(' ') && line.split('\n').next().is_some_and(|l| l.ends_with(':'))
        })
        .map_or(rest.len(), |(index, _)| index);
    &rest[..end]
}

// 0020 AC9, 0022 AC7
#[test]
fn release_workflow_builds_both_platforms() {
    let workflow = read(".github/workflows/release.yml");
    let uses: Vec<&str> = workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- uses: "))
        .collect();
    assert_eq!(uses.len(), 6);
    for action in uses {
        let (name, rest) = action.split_once('@').expect("pinned");
        let hash = rest.split_whitespace().next().expect("hash");
        // GitHub's own actions, and MSYS2's setup for Windows (ADR 0005).
        assert!(
            name.starts_with("actions/") || name == "msys2/setup-msys2",
            "{name}"
        );
        assert!(
            hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()),
            "{action}"
        );
    }
    assert!(workflow.contains("permissions:\n  contents: read\n"));
    assert_eq!(workflow.matches("contents: write").count(), 1);

    let appimage = job(&workflow, "appimage");
    let check = appimage.find("check-tag.sh").expect("tag check");
    assert!(check < appimage.find("scripts/appimage.sh").expect("build"));

    let windows = job(&workflow, "windows");
    let check = windows.find("check-tag.sh").expect("tag check");
    let tests = windows.find("cargo test --workspace").expect("tests");
    let bundle = windows.find("scripts/windows/bundle.sh").expect("bundle");
    let installer = windows.find("installer.iss").expect("installer");
    assert!(check < tests, "the tag is checked before building");
    assert!(
        tests < bundle && bundle < installer,
        "tests before packaging"
    );

    let release = job(&workflow, "release");
    assert!(release.contains("needs: [appimage, windows]"));
    assert!(release.contains("contents: write"));
    assert!(release.contains("--draft"));
    for file in [
        "dist/*.AppImage",
        "dist/bundled-packages.txt",
        "dist/*-windows-x86_64.zip",
        "dist/*-windows-x86_64-setup.exe",
        "dist/bundled-packages-windows.txt",
    ] {
        assert!(release.contains(file), "{file}");
    }
    assert!(!appimage.contains("contents: write") && !windows.contains("contents: write"));
}

// 0022 AC2
#[test]
fn windows_icon_has_the_usual_sizes() {
    let ico = fs::read(repo().join(format!("data/icons/{APP_ID}.ico"))).expect("ico");
    assert_eq!(&ico[..4], &[0, 0, 1, 0], "ICO header");
    let count = usize::from(u16::from_le_bytes([ico[4], ico[5]]));
    let sizes: Vec<u32> = (0..count)
        .map(|i| {
            let width = ico[6 + 16 * i];
            if width == 0 { 256 } else { u32::from(width) }
        })
        .collect();
    for size in [16, 32, 48, 256] {
        assert!(sizes.contains(&size), "{size} px missing in {sizes:?}");
    }
}

// 0022 AC3
#[test]
fn installer_is_per_user_and_never_default() {
    let script = read("scripts/windows/installer.iss");
    let lines: Vec<&str> = script.lines().map(str::trim).collect();
    for wanted in [
        "PrivilegesRequired=lowest",
        r"DefaultDirName={localappdata}\Programs\E-Invoice Viewer",
        "ChangesAssociations=yes",
    ] {
        assert!(lines.contains(&wanted), "missing {wanted}");
    }
    let app_id = lines
        .iter()
        .find_map(|line| line.strip_prefix("AppId={{"))
        .expect("AppId");
    assert_eq!(app_id.len(), 37, "a fixed GUID: {app_id}");

    let registry: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| line.starts_with("Root:"))
        .collect();
    assert!(!registry.is_empty());
    for entry in &registry {
        assert!(entry.starts_with("Root: HKCU;"), "per user only: {entry}");
        assert!(
            entry.contains("uninsdeletekey") || entry.contains("uninsdeletevalue"),
            "removed on uninstall: {entry}"
        );
        // The extensions' own default value is the default application.
        for extension in [".pdf", ".xml"] {
            assert!(
                !entry.contains(&format!(r#"Subkey: "Software\Classes\{extension}";"#)),
                "never the default for {extension}: {entry}"
            );
        }
    }
    for extension in [".pdf", ".xml"] {
        assert!(
            registry
                .iter()
                .any(|entry| entry
                    .contains(&format!(r#"Software\Classes\{extension}\OpenWithProgids"#))),
            "Open with for {extension}"
        );
    }
}

// 0026 AC4
#[test]
fn ci_workflow_checks_every_push() {
    let workflow = read(".github/workflows/ci.yml");
    assert!(workflow.contains("on:\n  push:\n  pull_request:\n"));
    assert!(workflow.contains("permissions:\n  contents: read\n"));
    assert!(!workflow.contains("contents: write"));
    for action in workflow
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- uses: "))
    {
        let (name, rest) = action.split_once('@').expect("pinned");
        let hash = rest.split_whitespace().next().expect("hash");
        assert!(
            name.starts_with("actions/") || name == "msys2/setup-msys2",
            "{name}"
        );
        assert!(
            hash.len() == 40 && hash.chars().all(|c| c.is_ascii_hexdigit()),
            "{action}"
        );
    }
    let linux = job(&workflow, "linux");
    for step in [
        "cargo fmt --all --check",
        "cargo clippy --workspace --all-targets --locked -- -D warnings",
        "cargo test --workspace --locked",
        "cargo tree -p e-invoice-viewer-core -e normal",
        "desktop-file-utils appstream",
        "actions/cache@",
    ] {
        assert!(linux.contains(step), "linux job: {step}");
    }
    let windows = job(&workflow, "windows");
    for step in [
        "msystem: UCRT64",
        "cargo test --workspace --locked",
        "actions/cache@",
    ] {
        assert!(windows.contains(step), "windows job: {step}");
    }
    assert!(workflow.matches("scripts/ci-step.sh").count() >= 4);
}

// 0026 AC5
#[test]
fn readme_shows_ci_badge() {
    assert!(read("README.md").contains("actions/workflows/ci.yml/badge.svg"));
}

// 0029 AC2: the tag check needs a changelog section for the version.
#[test]
fn tag_check_requires_changelog_entry() {
    let dir = temp_dir("tag-check");
    fs::create_dir_all(dir.join("scripts/appimage")).expect("dirs");
    fs::create_dir_all(dir.join("data")).expect("dirs");
    for file in [
        "scripts/appimage/check-tag.sh",
        "Cargo.toml",
        "Cargo.lock",
        &format!("data/{APP_ID}.metainfo.xml"),
    ] {
        fs::copy(repo().join(file), dir.join(file)).expect(file);
    }
    let version = workspace_version();
    let check = || {
        run(Command::new("bash")
            .arg(dir.join("scripts/appimage/check-tag.sh"))
            .arg(format!("v{version}")))
    };
    fs::write(
        dir.join("CHANGELOG.md"),
        format!("# Changelog\n\n## [{version}] - 2026-11-19\n"),
    )
    .expect("changelog");
    assert_success(&check(), "with a section");
    fs::write(dir.join("CHANGELOG.md"), "# Changelog\n\n## [Unreleased]\n").expect("changelog");
    let output = check();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("CHANGELOG.md"));
    assert_success(
        &run(Command::new("bash")
            .arg(repo().join("scripts/appimage/check-tag.sh"))
            .arg(format!("v{version}"))),
        "the repository's own changelog",
    );
}

// 0029 AC5
#[test]
fn readme_links_changelog_and_release_guide() {
    let readme = read("README.md");
    assert!(readme.contains("(CHANGELOG.md)"));
    assert!(readme.contains("(docs/releasing.md)"));
}

// Spec 0030 AC11: desktop entry, AppStream file and installer carry the new
// languages (both files are validated by the tests above).
#[test]
fn every_language_in_desktop_entry_metainfo_and_installer() {
    use e_invoice_viewer_core::i18n::Language;
    let desktop = desktop_entry();
    let metainfo = read("data/de.loges.EInvoiceViewer.metainfo.xml");
    let installer = read("scripts/windows/installer.iss");
    // The languages Inno Setup 6 ships a translation for.
    let inno = [
        "bg", "cs", "da", "de", "es", "fi", "fr", "hu", "it", "nl", "pl", "pt", "sk", "sl", "sv",
    ];
    for language in Language::ALL {
        let code = language.code();
        if language == Language::English {
            continue;
        }
        for key in ["GenericName", "Comment", "Keywords"] {
            assert!(
                desktop.contains(&format!("\n{key}[{code}]=")),
                "{key}[{code}]"
            );
        }
        // Summary, two paragraphs and four features.
        let elements = metainfo.matches(&format!("xml:lang=\"{code}\"")).count();
        assert!(elements >= 7, "{code}: {elements} elements in the metainfo");
        assert_eq!(
            installer.contains(&format!("Name: \"{code}\"; MessagesFile:")),
            inno.contains(&code),
            "installer language {code}"
        );
    }
}

// Spec 0030 AC12
#[test]
fn readme_and_changelog_note_the_translation_quality() {
    for file in ["README.md", "CHANGELOG.md"] {
        let text = read(file);
        assert!(
            text.contains("not checked by native speakers"),
            "{file}: note on quality"
        );
        assert!(
            text.contains("github.com/loge5/e-invoice-viewer/issues"),
            "{file}: where to report corrections"
        );
    }
}
