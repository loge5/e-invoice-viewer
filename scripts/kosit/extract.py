"""Turns KoSIT validator reports into the expected-results format of the
validation regression tests.

EN 16931 (spec 0009, default): one line per invoice, `<key>\t<rule ids>`, rule
ids sorted and comma-separated; `<key>\t!xsd` if the XML schema check failed
(the validator then skips the rules).

XRechnung (spec 0010, `--xrechnung`): one line per invoice that the validator
assigned to an XRechnung scenario and that passed the XML schema check,
`<key>\t<scenario>\t<id>:<level>,...` with the sorted, distinct pairs of rule
id and severity of all findings. The severity is the adjusted one ("Adj.
Grad" in the report): the scenario's `customLevel` for the rule if there is
one, else the level of the message, as computed by `rep:custom-level` in the
configuration's `default-report.xsl`.

Invoices without a matching scenario are reported on stderr and left out.

Usage: extract.py [--xrechnung] <report dir> <key prefix> <invoice file>...
"""
import os
import re
import sys

args = sys.argv[1:]
xrechnung = args[0] == "--xrechnung"
if xrechnung:
    args = args[1:]
report_dir, prefix, files = args[0], args[1], args[2:]
for path in files:
    name = os.path.basename(path)
    report = os.path.join(report_dir, name[: -len(".xml")] + "-report.xml")
    text = open(report, encoding="utf-8").read()
    if "noScenario" in text:
        print(f"no scenario: {path}", file=sys.stderr)
        continue
    messages = re.findall(
        r'<rep:message id="(val-[a-z]+)[^"]*" level="([a-z]+)"[^>]*\scode="([^"]*)"', text
    )
    key = prefix + name
    xsd_failed = any(kind == "val-xsd" for kind, _, _ in messages)
    if xrechnung:
        scenario = re.search(r"<rep:scenarioMatched>\s*<s:scenario>\s*<s:name>([^<]*)", text)
        scenario = scenario.group(1).strip() if scenario else ""
        if xsd_failed or "XRechnung" not in scenario:
            continue
        custom = {}
        for level, codes in re.findall(r'<s:customLevel level="([a-z]+)">([^<]*)</s:customLevel>', text):
            for code in codes.split():
                custom.setdefault(code, level)
        pairs = sorted(
            {f"{code}:{custom.get(code, level)}" for kind, level, code in messages if kind == "val-sch"}
        )
        print(f"{key}\t{scenario}\t{','.join(pairs)}")
    elif xsd_failed:
        print(f"{key}\t!xsd")
    else:
        ids = sorted({code for kind, _, code in messages if kind == "val-sch"})
        print(f"{key}\t{','.join(ids)}")
