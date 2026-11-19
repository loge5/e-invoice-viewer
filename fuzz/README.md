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
cargo +nightly fuzz run -O open corpus/open -- -max_total_time=3600 -timeout=5 -rss_limit_mb=2048
cargo +nightly fuzz run -O -s none validate corpus/validate -- -max_total_time=3600 -timeout=5 -rss_limit_mb=2048
cargo +nightly fuzz run -O xml_layout corpus/xml_layout -- -max_total_time=3600 -timeout=5 -rss_limit_mb=2048
```

Run one target at a time: each takes up to 2 GB, and libFuzzer's time limit
is meaningless on an overloaded machine. On a desktop, a memory limit for the
whole run helps: `systemd-run --user --scope -p MemoryMax=4G -p
MemorySwapMax=0 cargo +nightly fuzz run …`.

`validate` runs without the address sanitizer (`-s none`): the sanitizer
makes a validation about 20 times slower (a realistic invoice takes 4 s
instead of 0.2 s), so the 5 s limit would measure the instrumentation, not
the validation. The sanitizer only instruments Rust code, which is
memory-safe apart from the libxml2 calls in `validate/xsd.rs`; libxml2
itself is not instrumented either way. `open` and `xml_layout` keep it.

A crash or timeout leaves the input in `artifacts/<target>/`. Minimise it
(`cargo +nightly fuzz tmin <target> <file>`), add it as a regression test in
`crates/core/tests/robustness.rs`, fix the cause, and keep the corpus
minimised (`cargo +nightly fuzz cmin <target> corpus/<target>`).

The corpus starts from the sample invoices in `crates/core/tests/fixtures`.
