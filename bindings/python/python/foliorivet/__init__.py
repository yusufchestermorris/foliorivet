"""Experimental synchronous templates. Each render uses a fresh Rust session."""
from __future__ import annotations

import json
from os import PathLike, fspath
from typing import Any, Mapping

from . import _native
from ._native import TemplateError

__all__ = ["TemplateError", "render_template", "inspect_template", "preflight_template", "analyze_context"]

Path = str | PathLike[str]


def _context_json(context: str | Mapping[str, Any]) -> str:
    try:
        return context if isinstance(context, str) else json.dumps(context, ensure_ascii=False, allow_nan=False)
    except (TypeError, ValueError) as exc:
        raise TemplateError(f"serialize: target='context', scope=document: {exc}") from exc


def render_template(template_path: Path, context_json: str | Mapping[str, Any],
                    assets_dir: Path, output_path: Path, *, scalar_fanout: bool = False) -> str:
    """Populate supported scalar, row, picture, repeat and conditional tags into a new staging DOCX.

    Accept JSON text or a mapping of prevalidated values. Pass "" for assets_dir
    when no chart files are required. Existing output paths are refused. TemplateError
    includes operation, target and scope. No report calculations run here.
    """
    return _native.render_template(fspath(template_path), _context_json(context_json),
                                   fspath(assets_dir), fspath(output_path), scalar_fanout=scalar_fanout)


def inspect_template(template_path: Path, *, scalar_fanout: bool = False) -> dict[str, Any]:
    """List tags using the renderer's resolver, including unsupported diagnostics."""
    return json.loads(_native.inspect_template(fspath(template_path), scalar_fanout=scalar_fanout))


def preflight_template(template_path: Path, context_json: str | Mapping[str, Any],
                       assets_dir: Path = "", *, scalar_fanout: bool = False) -> dict[str, Any]:
    """Validate targets, values and required assets; does not guarantee saving or layout."""
    return json.loads(_native.preflight_template(fspath(template_path), _context_json(context_json),
                                                fspath(assets_dir), scalar_fanout=scalar_fanout))


def analyze_context(template_path: Path, context_json: str | Mapping[str, Any],
                    *, scalar_fanout: bool = False) -> dict[str, Any]:
    """Return active/inactive, missing and unused paths without decoding assets or writing output."""
    return json.loads(_native.analyze_context(fspath(template_path), _context_json(context_json),
                                            scalar_fanout=scalar_fanout))
