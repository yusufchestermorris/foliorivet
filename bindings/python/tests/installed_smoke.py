"""Installed native/CLI regression using only committed synthetic examples.

Usage: python -I bindings/python/tests/installed_smoke.py REPO NEW_RESULTS CLI
"""
import runpy
import sys
from pathlib import Path

repo, output, cli = sys.argv[1:]
sys.argv = ["smoke_installed.py", "--examples", str(Path(repo) / "bindings/python/examples"), "--cli", cli, "--output", output]
runpy.run_path(str(Path(repo) / "tools/release/smoke_installed.py"), run_name="__main__")
