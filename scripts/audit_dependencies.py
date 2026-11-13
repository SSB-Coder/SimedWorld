"""Query OSV advisories for pinned PyPI or Cargo dependencies before execution.

Known-advisory matching is not a guarantee against malicious code or unknown bugs.
This tool downloads JSON only; it does not install/import audited packages.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import sys
import tomllib
import urllib.request


def read_packages(path: Path, kind: str) -> list[dict]:
    if kind == "cargo":
        data = tomllib.loads(path.read_text(encoding="utf-8"))
        packages = [{"name": item["name"], "version": item["version"], "ecosystem": "crates.io"}
                    for item in data.get("package", []) if item.get("source", "").startswith("registry+")]
    else:
        packages = []
        for line in path.read_text(encoding="utf-8").splitlines():
            match = re.match(r"^([A-Za-z0-9_.-]+)==([^\s;\\]+)", line)
            if match:
                # CUDA/local tags share the upstream public version's advisory range.
                packages.append({"name": match[1], "version": match[2].split("+")[0],
                                 "installed_version": match[2], "ecosystem": "PyPI"})
    if not packages:
        raise ValueError(f"No pinned dependencies found in {path}")
    return sorted(packages, key=lambda item: (item["name"], item["version"]))


def audit(packages: list[dict]) -> list[dict]:
    findings = []
    for offset in range(0, len(packages), 100):
        batch = packages[offset:offset + 100]
        queries = [{"package": {"name": p["name"], "ecosystem": p["ecosystem"]},
                    "version": p["version"]} for p in batch]
        request = urllib.request.Request(
            "https://api.osv.dev/v1/querybatch", data=json.dumps({"queries": queries}).encode(),
            headers={"Content-Type": "application/json", "User-Agent": "World-dependency-review/0.1"})
        with urllib.request.urlopen(request, timeout=30) as response:
            results = json.load(response)["results"]
        if len(results) != len(batch):
            raise ValueError("Advisory service returned an incomplete result")
        for package, result in zip(batch, results, strict=True):
            if result.get("next_page_token"):
                raise ValueError("Paginated advisory results require manual review; refusing a clean verdict")
            if result.get("vulns"):
                findings.append({**package, "advisories": result["vulns"]})
    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kind", choices=("python", "cargo"))
    parser.add_argument("lock", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    report = {"checked_at_utc": datetime.now(timezone.utc).isoformat(), "service": "https://api.osv.dev",
              "input": str(args.lock), "scope": args.kind,
              "limitation": "Known advisory matches only; not proof of safety or full source-code review."}
    try:
        report["input_sha256"] = hashlib.sha256(args.lock.read_bytes()).hexdigest()
        packages = read_packages(args.lock, args.kind)
        findings = audit(packages)
        report.update(status="findings" if findings else "no_known_advisory_matches",
                      package_count=len(packages), packages=packages, findings=findings)
        exit_code = 1 if findings else 0
    except Exception as error:
        report.update(status="incomplete", error=str(error))
        exit_code = 2
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: report[key] for key in ("status", "package_count", "findings", "error") if key in report}))
    return exit_code


if __name__ == "__main__":
    sys.exit(main())
