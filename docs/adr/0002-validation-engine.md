# 0002 — Validation engine: official Schematron asserts evaluated with xee (XPath 3.1)

- Status: accepted
- Date: 2026-11-16
- Spike: [specs/0001](../../specs/0001-spike-validation-engine.md)

## Context

The viewer has to show EN 16931 and XRechnung findings (requirements,
decisions 5 and 6) inside a self-contained binary: no Java, no network. The
official rules are published as Schematron (`queryBinding="xslt2"`), normally
compiled to XSLT and executed with Saxon. The spike checked whether they can
be evaluated in Rust with the same results as the official KoSIT validator.

## Decision

**Option (a): evaluate the official, unmodified Schematron artefacts at run
time with the pure-Rust XPath 3.1 engine [`xee-xpath`](https://github.com/Paligo/xee)**
(Paligo, MIT). A small Schematron runner in `crates/core`:

1. Parses the `.sch` files (resolving `include`): namespaces, `let`
   (schema/pattern level = global, rule level = local), `xsl:function`,
   patterns, rules, asserts with `id`, `flag` and message.
2. Turns each rule context (an XSLT match pattern) into an XPath expression
   (`//` prefix for relative union branches), evaluated against the document.
3. Applies Schematron first-match semantics per pattern: a node is handled only
   by the first rule of a pattern whose context matches it.
4. Compiles each assert `test` into one XPath 3.1 expression. Variables become
   XPath `let` bindings (globals evaluated against the document node via
   `root(.) ! (…)`). Each `xsl:function` becomes an inline function item bound
   to a variable, and its calls are rewritten.
5. Evaluates the asserts per context node. An assert fires if the effective
   boolean value is false. If evaluating an assert raises a dynamic error, that
   counts as fired (see "Short-circuit" below).

## Results

All runs on the reference machine (Intel Core Ultra 7 155H, 30 GB RAM),
release build, single thread. The prototype is throwaway code and is not
merged.

Rule versions used:

| Rule set | Version |
|---|---|
| CEN EN 16931 (CII and UBL) | git `b17b269`, schematron identical to release `validation-1.3.17` |
| KoSIT XRechnung | `xrechnung-schematron` git `7e3ddbe`, schematron identical to release `v2.6.0` (XRechnung 3.0.2) |

**Reference:** KoSIT validator 1.6.3, which uses Saxon HE.

- For EN 16931, the spike used its own scenario with the CEN XSLT of the same
  git version: XSD plus CEN rules, any CII or UBL invoice.
- For XRechnung, it used the official configuration
  `xrechnung-3.0.2-validator-configuration-2026-08-31`. The comparison covers
  only rule IDs from the XRechnung artefact, and only invoices that KoSIT
  assigns to an XRechnung scenario.

**Corpus:** All public CII and UBL instances from:

- CEN `cii/examples`, `ubl/examples` and `test/cii`
- KoSIT `xrechnung-schematron/test/instances`
- KoSIT `xrechnung-testsuite`
- our fixtures A and B
- 6 mutants of A with deliberate violations

**Excluded:** Invoices that fail the XSD are excluded, because KoSIT skips
Schematron when the schema fails (1 per syntax). The 2 CEN `testSet`
containers are also excluded: they are not invoices, and KoSIT reports
`noScenario` for them.

### Rule IDs compared with KoSIT

| Rule set | Invoices compared | With findings | Identical | Differences |
|---|---|---|---|---|
| EN 16931 CII (spike criterion) | 191 | 48 | **191** | 0 |
| EN 16931 UBL (extra, beyond the criterion) | 287 | 110 | **287** | 0 |
| XRechnung CIUS CII (extra) | 170 | 118 | **170** | 0 |

**EN 16931 CII:** The 48 invoices with findings include errors, such as
`BR-CO-10`, `BR-CO-15`, `BR-CO-16`, `BR-S-08`, `BR-05`, `BR-CL-01`, `BR-42`
and `BR-44`, as well as warnings (`CII-SR-*`). The invoice with 3000 lines
also matches (`BR-CO-10` and `BR-S-08` on both sides).

**The policy "dynamic error = fired" is needed for full agreement:**

- Without it, UBL has 285/287 identical; the 2 differences are `BR-CO-15`.
- Without it, XRechnung has 167/170 identical; the 3 differences are
  `BR-DE-19` and `BR-DE-20` from `xr:checkIBAN`.
- The XRechnung run above already contains a top-level `and` rewrite inside
  `xsl:function`. With the error policy alone, that rewrite is not needed.

### Coverage (asserts in the official artefact)

| Rule set | Asserts | Evaluable | Not evaluable |
|---|---|---|---|
| EN 16931 CII (`EN16931-CII-validation-preprocessed.sch`) | 810 | 810 (100 %), measured | none |
| EN 16931 UBL (`EN16931-UBL-validation-preprocessed.sch`) | 982 | 982 (100 %), compiled and also measured | none |
| XRechnung CIUS CII (`XRechnung-CII-validation.sch` + `common.sch`) | 52 | 52 (100 %), compiled and also measured | none |
| XRechnung CIUS UBL (for reference) | 56 | 56 (100 %), compiled | none |

**What the CEN rules use:** only XPath 2.0 core functions (`not`, `count`,
`sum`, `round`, `xs:decimal`, `normalize-space`, `upper-case`, `matches`,
`every … satisfies`, …). They use no variables and no functions of their own.

**What XRechnung additionally uses:**

- 26 `let` variables (globals and pattern level, also inside rule contexts,
  e.g. `[$isExtension]`)
- 2 `xsl:function` (`u:decimalOrZero`, `xr:checkIBAN`), with regex,
  `for … return`, `string-to-codepoints` and `mod`

All of these map to plain XPath 3.1 as described in steps 4 and 5.

### Time per invoice

The time is validation only: parsing into xee plus evaluating all asserts.
Compiling the rule set happens once at start-up: 150 ms for CII, 480 ms for
UBL.

| Invoice | Prototype | KoSIT CLI (incl. JVM start) |
|---|---|---|
| Sample A (2 lines, CII) | **38 ms** | 1.4 s |
| CII, 1–5 lines (185 invoices) | median 48 ms, max 226 ms | – |
| UBL, 1–5 lines (277 invoices) | median 150 ms, max 760 ms | – |
| XRechnung CII rules, additionally | median 4 ms, max 17 ms | – |
| CII 500 lines | 2.7 s (7.5 s before hoisting) | 2.2 s |
| CII 3000 lines (2 MB) | 37 s (228 s before hoisting) | 5.0 s |

**A typical invoice stays well below 1 s**, for CII and UBL alike, including
the XRechnung rules.

**Large invoices grow superlinearly.** xee evaluates the descendant axis at
about 1–1.5 µs per node, and does not lift context-independent
subexpressions out of loops. Saxon does both.

- Some asserts test an absolute path once per line item, e.g. `BR-CO-18`:
  `//rsm:SupplyChainTradeTransaction/…` per `IncludedSupplyChainTradeLineItem`.
- Evaluating purely absolute tests only once per document ("hoisting") gave a
  factor of 3–6, with identical results.
- The remainder comes from mixed expressions, e.g. `CII-DT-033`
  `not(@unitCode) or (/rsm:…)`, and from 104 UBL rule contexts that are all
  evaluated as `//…` scans.

An attempt to bundle all asserts of a rule into one query using `except` was
slower. It also showed that xee **panics** on very large expressions
(`jump too far` in `xee-ir`).

### Engine findings to handle in 0009

| Finding | Handling |
|---|---|
| xee does not short-circuit `and`/`or`, including nested ones (`every … satisfies (A) and (B)`). Saxon does. Example: `count(…) eq 1 and (… + …)` raises `XPTY0004` when there are two tax amounts | A dynamic error in a test counts as "assert fired". This matched Saxon in every case of the corpus. Long term, rewrite `and` as `if (A) then B else false()` on the AST, or contribute a fix upstream |
| Compiling can panic on very large expressions | Compile asserts individually and guard compilation with `catch_unwind`. All official asserts compile without a panic |
| Descendant axis is slow, and there is no loop hoisting | Hoist purely absolute tests (implemented in the prototype). Further levers: lift absolute subexpressions into `let` variables evaluated once per document; match contexts in a single tree walk instead of one `//` scan per rule; validate on a background thread |
| Match patterns are not XPath | Prefix relative union branches with `//`. All 115 CII, 104 UBL and 27 XRechnung contexts produced the same results this way |

## Alternatives

| Option | Assessment |
|---|---|
| **(a) Official asserts with an XPath engine in Rust (chosen)** | 100 % of the asserts are evaluable, and the results are identical to KoSIT on the whole public corpus. Rule updates are swapping files, not code changes. Risks: xee is young (0.x); performance on very large invoices; semantic gaps (short-circuit) |
| (a′) Same with `sxd-xpath` | XPath 1.0 only: no `xs:decimal`, `upper-case`, `matches`, `every`/`some`, `let`, inline functions. Most asserts would not compile |
| (b) Re-implement the rules natively in Rust | About 1850 asserts (CII + UBL) plus 108 XRechnung asserts, each a rule-update risk. Every CEN and KoSIT release means manual follow-up and a new test effort. Fast and fully controllable, but contradicts the aim of showing the official rules. Possibly worthwhile only selectively, for single hot spots |
| (c) External KoSIT validator (Java) as an optional fallback | Contradicts decision 6 (no Java at run time) and the 1.4 s start-up per call. It remains a development tool only: it produces the reference results for the regression tests. It is not an end-user option |

## Licences

| Component | Licence | Use | GPL-3.0-or-later |
|---|---|---|---|
| CEN `eInvoicing-EN16931` (Schematron, code lists) | **EUPL-1.2** (not Apache 2.0, as the spec assumed) | bundled in the app | compatible: EUPL-1.2 Art. 5 plus appendix name GPL-3.0 as a compatible licence. Keep the EUPL notice for the bundled rule files |
| KoSIT `xrechnung-schematron` | Apache-2.0 | bundled in the app | compatible (Apache-2.0 → GPL-3.0). Keep the licence file / NOTICE |
| KoSIT `xrechnung-testsuite` | Apache-2.0 | test data (fixtures) only | compatible, permitted under AGENTS.md (fixtures synthetic or Apache-licensed) |
| `xee-xpath` and the `xee-*` family, `xot` | MIT | library | compatible |
| Transitive dependencies of xee (221 crates) | MIT, Apache-2.0, Unicode-3.0 (ICU4X), Zlib, BSD, Unlicense, MPL-2.0/LGPL only as alternatives | library | all permissive, compatible. Pin it down with `cargo deny check licenses` (ADR 0001) |
| KoSIT validator 1.6.3 (incl. Saxon HE) | Apache-2.0 (Saxon HE: MPL-2.0) | development only, not distributed | not relevant |

**Size:** The prototype binary is 9.3 MB (release, mostly ICU4X via xee).

## Consequences for 0009 and 0010

**New dependency:** `xee-xpath` (exactly pinned, `=0.1.x`) goes into
`crates/core`. It is GUI-free, so the `cargo tree` gate still holds. This
needs approval during the 0009 implementation, as AGENTS.md requires. Also
`roxmltree`, already present, is used to parse the `.sch` files.

**Rule versions to bundle:**

- 0009: CEN `validation-1.3.17` (CII and UBL preprocessed `.sch` plus the
  CEN code lists as included in it)
- 0010: KoSIT `xrechnung-schematron` `v2.6.0` (XRechnung 3.0.2)

This resolves the NEEDS CLARIFICATION in 0009 and 0010. The `.sch` files are
stored unchanged under `crates/core/rules/<source>-<version>/`, together with
the licence text and the source URL. They are embedded into the binary with
`include_str!`, so no network access is needed and nothing is read at run
time.

**Rule updates:**

- An update means swapping the folder, bumping the version string, and
  re-running the reference regression test.
- The rule version is shown in the result ("applied rule sets", 0009
  contract).
- There is no automatic download, in line with decision 6.

**Regression test against KoSIT:**

- The KoSIT reference results are generated once, at development time, by a
  script.
- They are committed as expected values (rule IDs per invoice) for a
  selection of the Apache-licensed KoSIT instances plus our fixtures.
- The CEN instances (EUPL) and their expected values are not committed.
- A `cargo test` compares them, without Java.

**0009 scope:**

- Provide the Schematron runner (steps 1–5) plus hoisting of absolute tests.
- Error policy "dynamic error = fired", with a test case for `BR-CO-15` (UBL,
  two `TaxTotal` amounts).
- `catch_unwind` for compiling and evaluating.
- Location: the context node of the fired assert as a 1-based XPath, as the
  field model already produces. This is what links findings to fields in
  0011.
- Messages: the text of the `assert` element, normalised for whitespace.
  `value-of` / `name` inside messages is not evaluated (KoSIT messages are
  also mostly static); the rule ID and the location carry the information.

**Performance target for 0009:**

- Typical invoice (1–5 lines): < 1 s; measured 40–150 ms.
- Validation runs off the GTK main thread, with no UI blocking even for large
  invoices.
- Large invoices (500+ lines): the target, e.g. "< 3 s for sample L", is set
  in 0009. The levers are listed above, starting with hoisting absolute
  subexpressions into `let` variables. Upstream issues for xee (`and`
  short-circuit, descendant performance) are worth reporting.

**0010 scope:**

- Same runner, plus support for `let` and `xsl:function` (as an inline
  function), as in step 4.
- XRechnung rules apply only when BT-24 names XRechnung 3.0, CIUS, Extension
  or CVD. This is the same scenario choice KoSIT makes; otherwise there are no
  `BR-DE` findings.
- UBL support for XRechnung depends on 0006.

**Compile time:**

- 150 ms (CII), 480 ms (UBL) and 50 ms (XRechnung) once per rule set.
- Compile lazily at the first validation of each syntax, on the background
  thread, and keep it for the lifetime of the app.
