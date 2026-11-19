# Fuzzing (spec 0025)

The targets feed arbitrary bytes into the parts of `crates/core` that read
untrusted files: opening a document (PDF and XML, attachments, field
model), the validation (XML schema and business rules) and the layout of the
XML view. None of them may panic, abort or take longer than 5 s.

Requirements (developer only, not needed for the app): a nightly Rust
compiler and `cargo-fuzz`:

```bash
rustup toolchain install nightly
cargo install cargo-fuzz --locked
```

Run a target, e.g. for one hour, starting from the kept corpus:

```bash
cd fuzz
cargo +nightly fuzz run open corpus/open -- -max_total_time=3600 -timeout=5 -rss_limit_mb=4096
cargo +nightly fuzz run validate corpus/validate -- -max_total_time=3600 -timeout=5 -rss_limit_mb=4096
cargo +nightly fuzz run xml_layout corpus/xml_layout -- -max_total_time=3600 -timeout=5 -rss_limit_mb=4096
```

A crash or timeout leaves the input in `artifacts/<target>/`. Minimise it
(`cargo +nightly fuzz tmin <target> <file>`), add it as a regression test in
`crates/core/tests/robustness.rs`, fix the cause, and keep the corpus
minimised (`cargo +nightly fuzz cmin <target> corpus/<target>`).

The corpus starts from the sample invoices in `crates/core/tests/fixtures`.
