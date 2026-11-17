"""Turns KoSIT validator reports into the expected-results format of the
EN 16931 regression test: one line per invoice, `<key>\t<rule ids>`, rule ids
sorted and comma-separated; `<key>\t!xsd` if the XML schema check failed (the
validator then skips the rules). Invoices without a matching scenario are
reported on stderr and left out.

Usage: extract.py <report dir> <key prefix> <invoice file>...
"""
import os
import re
import sys

report_dir, prefix, files = sys.argv[1], sys.argv[2], sys.argv[3:]
for path in files:
    name = os.path.basename(path)
    report = os.path.join(report_dir, name[: -len(".xml")] + "-report.xml")
    text = open(report, encoding="utf-8").read()
    if "noScenario" in text:
        print(f"no scenario: {path}", file=sys.stderr)
        continue
    messages = re.findall(r'<rep:message id="(val-[a-z]+)[^"]*"[^>]*\scode="([^"]*)"', text)
    key = prefix + name
    if any(kind == "val-xsd" for kind, _ in messages):
        print(f"{key}\t!xsd")
    else:
        ids = sorted({code for kind, code in messages if kind == "val-sch"})
        print(f"{key}\t{','.join(ids)}")
