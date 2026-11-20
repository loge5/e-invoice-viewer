//! Spec 0027 AC10 — the bundled FeRD files are unchanged copies: their
//! SHA-256 equals the list in `rules/facturx-1.09.2/SOURCE.md`, and every
//! bundled file is listed.

use std::collections::BTreeSet;
use std::path::Path;

fn bundle() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/rules/facturx-1.09.2"))
}

/// `(sha256, path)` of the code block in `SOURCE.md`.
fn listed() -> Vec<(String, String)> {
    let source = std::fs::read_to_string(bundle().join("SOURCE.md")).expect("SOURCE.md");
    let block = source.split("```").nth(1).expect("checksum block");
    block
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let (sum, path) = line.split_once("  ").expect("`sum  path`");
            (sum.to_string(), path.to_string())
        })
        .collect()
}

#[test]
fn bundled_facturx_files_unchanged() {
    let listed = listed();
    assert_eq!(
        listed.len(),
        30,
        "5 profiles × (4 XSD, Schematron, code lists)"
    );
    for (sum, path) in &listed {
        let bytes = std::fs::read(bundle().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert_eq!(&sha256_hex(&bytes), sum, "{path} differs from SOURCE.md");
    }
    let mut bundled = BTreeSet::new();
    for profile in std::fs::read_dir(bundle()).expect("bundle") {
        let profile = profile.expect("entry").path();
        if !profile.is_dir() {
            continue;
        }
        for file in std::fs::read_dir(&profile).expect("profile dir") {
            let file = file.expect("entry").path();
            let relative = file.strip_prefix(bundle()).expect("below bundle");
            bundled.insert(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    let listed: BTreeSet<String> = listed.into_iter().map(|(_, path)| path).collect();
    assert_eq!(bundled, listed, "every bundled file is listed");
}

#[test]
fn licence_next_to_the_files_and_in_the_readme() {
    let licence = std::fs::read_to_string(bundle().join("LICENSE")).expect("LICENSE");
    assert!(licence.contains("Apache License") && licence.contains("Version 2.0"));
    let readme = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md"))
        .expect("README");
    assert!(
        readme.contains("crates/core/rules/facturx-1.09.2"),
        "README names the bundled FeRD files"
    );
}

#[test]
fn sha256_of_known_inputs() {
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(&[b'a'; 1000]),
        "41edece42d63e8d9bf515a9ba6932e1c20cbc9f5a5d134645adb5db1b9737ea3"
    );
}

/// SHA-256 (FIPS 180-4), small enough for a test, so that no dependency is
/// needed.
fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut message = data.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for block in message.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (state, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *state = state.wrapping_add(value);
        }
    }
    h.iter().map(|word| format!("{word:08x}")).collect()
}
