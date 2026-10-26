#!/usr/bin/env python3
"""Generate Unicode 17 titlecase exceptions from official UCD input files.

Input URLs:
https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt
https://www.unicode.org/Public/17.0.0/ucd/SpecialCasing.txt
License: ../licenses/UNICODE.txt
"""
import argparse
import subprocess
from pathlib import Path


def generate(unicode_data: Path, special_casing: Path) -> str:
    upper, title, title_chars = {}, {}, []
    for line in unicode_data.read_text().splitlines():
        fields = line.split(";")
        if len(fields) != 15:
            raise ValueError("expected UnicodeData records with 15 fields")
        code = int(fields[0], 16)
        upper[code] = [int(fields[12], 16)] if fields[12] else [code]
        title[code] = [int(fields[14], 16)] if fields[14] else upper[code]
        if fields[2] == "Lt":
            title_chars.append(code)
    special = special_casing.read_text()
    if not special.startswith("# SpecialCasing-17.0.0.txt"):
        raise ValueError("expected Unicode 17.0.0 SpecialCasing data")
    for line in special.splitlines():
        fields = line.split("#", 1)[0].strip().split(";")
        if len(fields) < 5 or fields[4].strip():
            continue
        code = int(fields[0], 16)
        title[code] = [int(value, 16) for value in fields[2].split()]
        upper[code] = [int(value, 16) for value in fields[3].split()]
    lines = [
        "//! Generated Unicode 17.0.0 titlecase exceptions; do not edit by hand.",
        "//! Source: UCD UnicodeData.txt and SpecialCasing.txt; see licenses/UNICODE.txt.",
        "//! Regenerate with scripts/generate_unicode_case.py.",
        "", "pub(super) fn title(ch: char) -> String {", "    match ch {",
    ]
    for code, values in sorted(title.items()):
        if values == upper[code]:
            continue
        mapped = "".join(f"\\u{{{value:X}}}" for value in values)
        lines.append(f"        '\\u{{{code:X}}}' => \"{mapped}\".into(),")
    lines += ["        _ => ch.to_uppercase().collect(),", "    }", "}", "", "pub(super) fn is_titlecase(ch: char) -> bool {", "    matches!(ch,"]
    lines.append("        " + " | ".join(f"'\\u{{{code:X}}}'" for code in title_chars))
    lines += ["    )", "}", ""]
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("unicode_data", type=Path)
    parser.add_argument("special_casing", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.write_text(generate(args.unicode_data, args.special_casing))
    subprocess.run(["rustfmt", "--edition", "2021", str(args.output)], check=True)


if __name__ == "__main__":
    main()
