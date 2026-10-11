"""Generate an offline dependency inventory and candidate notice bundle.

Python 3.11+ and Cargo are required. This collects evidence, not legal approval.
It deliberately inventories the entire workspace lockfile graph (including the
demo, build, and target-specific dependencies), not one shipped executable.
"""
import argparse
import hashlib
import json
import pathlib
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    args = parser.parse_args()
    root = pathlib.Path(__file__).resolve().parents[1]
    metadata = json.loads(subprocess.check_output(
        ["cargo", "metadata", "--manifest-path", str(root / "Cargo.toml"),
         "--locked", "--offline", "--format-version", "1"], text=True, encoding="utf-8"))
    packages = sorted(metadata["packages"], key=lambda p: (p["name"], p["version"]))
    args.output.mkdir(parents=True, exist_ok=True)
    rows = []
    notices = ["Candidate third-party notices for ratatui-typst\n"
               "Entire workspace graph, including demo, build, and target-only dependencies.\n"
               "Review missing files and nested assets before shipping a binary.\n"]
    missing = []
    for package in packages:
        if not package["source"]:  # Project code is licensed separately.
            continue
        directory = pathlib.Path(package["manifest_path"]).parent
        candidates = set()
        for child in directory.iterdir():
            if child.is_file() and child.name.upper().startswith(("LICENSE", "LICENCE", "NOTICE", "COPYING", "COPYRIGHT", "UNLICENSE")):
                candidates.add(child)
            elif child.is_dir() and child.name.lower() in ("licenses", "license"):
                candidates.update(path for path in child.rglob("*") if path.is_file())
        if package["license_file"]:
            candidates.add(directory / package["license_file"])
        files = []
        for path in sorted(candidates):
            if not path.is_file():
                continue
            data = path.read_bytes()
            try:
                content = data.decode("utf-8")
            except UnicodeDecodeError:
                continue
            relative = path.relative_to(directory).as_posix()
            files.append({"path": relative, "sha256": hashlib.sha256(data).hexdigest()})
            notices.append(f"\n{'=' * 78}\n{package['name']} {package['version']} / {relative}\n"
                           f"Declared license: {package['license'] or 'UNDECLARED'}\n"
                           f"{'=' * 78}\n{content}\n")
        if not files:
            missing.append(f"{package['name']} {package['version']}")
        rows.append({"name": package["name"], "version": package["version"],
                     "license": package["license"], "source": package["source"], "files": files})
    assets = next(p for p in packages if p["name"] == "typst-assets")
    asset_root = pathlib.Path(assets["manifest_path"]).parent
    comparisons = {}
    for filename in ("LICENSE", "NOTICE"):
        stored = root / "licenses" / f"typst-assets-{filename}.txt"
        upstream = asset_root / filename
        comparisons[filename] = {"matches": stored.read_bytes() == upstream.read_bytes(),
                                 "sha256": hashlib.sha256(upstream.read_bytes()).hexdigest()}
    supplemental = []
    for path in sorted((root / "licenses").glob("LICENSE-*.txt")):
        data = path.read_bytes()
        supplemental.append({"path": path.name, "sha256": hashlib.sha256(data).hexdigest()})
        notices.append(f"\n{'=' * 78}\nSupplemental asset license / {path.name}\n{'=' * 78}\n{data.decode('utf-8')}\n")
    report = {"scope": "full resolved workspace Cargo graph, including demo; not a per-binary compliance clearance",
              "packages": rows, "missing_license_files": missing,
              "typst_assets_notices": comparisons,
              "supplemental_asset_licenses": supplemental,
              "cargo_lock_sha256": hashlib.sha256((root / "Cargo.lock").read_bytes()).hexdigest()}
    (args.output / "inventory.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    (args.output / "THIRD_PARTY_NOTICES.txt").write_text("".join(notices), encoding="utf-8")
    markdown = ["# Dependency License Inventory\n",
                "Generated from workspace Cargo.lock; includes demo, build, and target-specific dependencies.\n"
                "Declared license expressions are not a legal approval. Candidate notices\n"
                "must be reviewed for nested assets and missing text before distribution.\n",
                "| Dependency | Version | Declared license | Notice files |\n| --- | --- | --- | --- |"]
    for row in rows:
        markdown.append(f"| {row['name']} | {row['version']} | {row['license'] or 'UNDECLARED'} | {len(row['files'])} |")
    markdown.append("\n## Missing notice files\n\n" + ("\n".join(f"- {name}" for name in missing) or "None detected by filename scan."))
    (args.output / "INVENTORY.md").write_text("\n".join(markdown) + "\n", encoding="utf-8")
    print(f"Inventoried {len(rows)} dependencies; {len(missing)} without detected license files.")
    print(f"Typst asset copies match: {all(item['matches'] for item in comparisons.values())}")
    if not all(item["matches"] for item in comparisons.values()):
        raise SystemExit("Bundled Typst notices do not match the locked dependency")


if __name__ == "__main__":
    main()
