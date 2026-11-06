"""Assemble explicitly selected candidates; never upload or grant release clearance."""
import argparse
import hashlib
import json
import shutil
import tomllib
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--windows", type=Path, required=True)
    parser.add_argument("--linux", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[2]
    version = tomllib.loads((repo / "bindings/python/pyproject.toml").read_text())["project"]["version"]
    notice_root = repo / "bindings/python/python/foliorivet/licenses"
    cli_notices = repo / "dist/cli-notices"
    for directory in (notice_root / "dependencies", cli_notices):
        inventory = json.loads((directory / "inventory.json").read_text())
        if inventory["missing"]:
            raise SystemExit(f"Missing notices: {directory}")
    candidates = []
    for platform, directory, binary, tag in (
        ("windows-x64", args.windows, "foliorivet-cli.exe", "win_amd64"),
        ("linux-x64", args.linux, "foliorivet-cli", "manylinux_2_35_x86_64"),
    ):
        manifest = json.loads((directory / "manifest.json").read_text())
        if manifest["profile"] != "release":
            raise SystemExit("Only optimized release candidates may be assembled")
        wheels = list(directory.glob(f"foliorivet-{version}-cp312-cp312-*{tag}.whl"))
        if len(wheels) != 1:
            raise SystemExit(f"Expected exactly one matching wheel in {directory}")
        wheel = wheels[0]
        for file in (wheel, directory / binary):
            if hashlib.sha256(file.read_bytes()).hexdigest() != manifest["files"].get(file.name):
                raise SystemExit(f"Candidate hash mismatch: {file}")
        with ZipFile(wheel) as archive:
            for name in ("LICENSE-MIT", "LICENSE-APACHE", "NOTICE"):
                if archive.read("foliorivet/licenses/" + name) != (repo / name).read_bytes():
                    raise SystemExit(f"Wheel notice differs: {name}")
            for file in notice_root.rglob("*"):
                if file.is_file() and archive.read("foliorivet/licenses/" + file.relative_to(notice_root).as_posix()) != file.read_bytes():
                    raise SystemExit(f"Wheel notice differs: {file.name}")
        candidates.append((platform, directory / binary, wheel))
    args.output.mkdir(parents=True, exist_ok=False)
    uploads = args.output / "pypi"
    uploads.mkdir()
    for platform, binary, wheel in candidates:
        shutil.copy2(wheel, uploads / wheel.name)
        with ZipFile(args.output / f"foliorivet-{version}-{platform}-cli.zip", "x", ZIP_DEFLATED) as archive:
            archive.write(binary, binary.name)
            for name in ("LICENSE-MIT", "LICENSE-APACHE", "NOTICE"):
                archive.write(repo / name, name)
            for directory, prefix in ((cli_notices, "licenses/dependencies"), (notice_root / "language-data", "licenses/language-data")):
                for file in directory.rglob("*"):
                    if file.is_file():
                        archive.write(file, prefix + "/" + file.relative_to(directory).as_posix())
            archive.writestr("README.txt", f"FolioRivet {version} experimental preview candidate.\nRun {binary.name} --help. Extract all files together.\nNot production-qualified. Redistribution and final review remain required.\nBased on WordCraft by the ArtCraft team.\n")
    lines = [hashlib.sha256(f.read_bytes()).hexdigest() + "  " + f.relative_to(args.output).as_posix() for f in sorted(args.output.rglob("*")) if f.is_file()]
    (args.output / "SHA256SUMS.txt").write_text("\n".join(lines) + "\n")
    print("Assembled review artifacts; publication remains HOLD:", args.output)


if __name__ == "__main__":
    main()
