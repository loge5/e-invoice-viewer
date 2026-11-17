# KoSIT test instances

- Source: https://github.com/itplr-kosit/xrechnung-schematron, tag `v2.6.0`
  (commit `217c6cbf8ac9a99d780cefb4f044b2c2778d4744`), directory
  `test/instances/{cii,ubl-inv,ubl-cn}`, copied unchanged.
- Only instances that are valid against the XML schema are kept (the KoSIT
  validator skips the rules for the others); one UBL CreditNote was left out.
- Licence: Apache License 2.0, see `LICENSE`.
- `expected-en16931.tsv`: fired EN 16931 rule IDs per invoice, produced with
  the KoSIT validator 1.6.3 by `scripts/kosit-reference.sh` (spec 0009).
