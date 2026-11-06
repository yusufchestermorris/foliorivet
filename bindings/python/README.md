# Experimental template bindings

Experimental preview 0.1.0a1, based on WordCraft revision
`92ecb40105f56b674c18e3e88557e1456591868b` plus FolioRivet fidelity and template patches. Publication remains on HOLD
until the preview release checklist is complete. The Python package is `foliorivet`.

The core operation lives in `crates/engine/src/template.rs`. This isolated PyO3
crate calls it directly; no subprocess, Word installation or desktop window is
used for generation. Its separate lockfile starts from the upstream lockfile,
adding PyO3 dependencies without changing the engine's resolved build inputs.
Python build dependencies are excluded from the upstream Rust/wasm workspace.

## Installed-wheel quick start

Use a clean CPython 3.12 x64 environment. Install the locally built wheel using
`python -m pip install --no-index /path/to/foliorivet-0.1.0a1-...whl`. A matching
prebuilt wheel requires no Rust compiler at installation or generation time.
Building from source requires Rust 1.95, platform linker/build tools and
`maturin==1.10.2`; source-distribution installation is not qualified.

Run from outside the source tree:

```python
from pathlib import Path
from foliorivet import inspect_template, preflight_template, render_template

template = Path("scalar-template.docx")
inspection = inspect_template(template)
print(inspection)
context = {
    "metadata": {"model_id": "Example 42 — café"},
    "labels": {
        target["tag"].removeprefix("value:labels."): "Review — Ω"
        for target in inspection["targets"]
        if target["tag"].startswith("value:labels.")
    },
}
print(preflight_template(template, context))
Path("staging").mkdir(exist_ok=True)
output = render_template(template, context, "", Path("staging/report.docx"))
print(output)
```

The same code is runnable as `python -I quickstart.py TEMPLATE NEW_OUTPUT` with
the supplied scalar fixture. For the comprehensive installed example, use
`tests/installed_smoke.py REPO EVIDENCE CLI`. The script discovers its label tags,
constructs synthetic input, and exercises an actually installed extension. Evidence
and tested environments are recorded in `docs/wordcraft/evidence/STAGE3.md`.

## Contract

`render_template(template_path, context_json, assets_dir, output_path) -> str`
is synchronous and creates a fresh document session per call. Paths accept strings
or `os.PathLike[str]`; context accepts JSON text or a mapping. The result is the
output path as supplied. The parent staging directory must exist. Existing paths,
including the template, are never replaced. A filesystem write failure may leave
an incomplete staging file: discard it and retry with a new path. Publication is
the caller's responsibility.

`value:metadata.model_id` resolves `context["metadata"]["model_id"]`. Field
segments contain ASCII letters, digits or underscores; each segment must be
nonempty and the total path is limited to 256 bytes. Tags must be unique across
the body and all document stories, including headers and footers. Every tagged
control is required. Each supplied scalar leaf also declares a required template
target; unused input fields fail so deleted controls and misspelled fields cannot
silently discard data. Supply only values intended for this scalar template.
Context nesting is limited to 32 levels.

Values are strings, finite JSON numbers or booleans; booleans become `true`/`false`.
Python supplies precomputed/preformatted values and unavailable wording. Nulls,
objects and arrays at scalar targets fail. JSON input is limited to 1 MiB,
each replacement to 64 KiB UTF-8 and tagged targets to 1024. LF becomes a manual
line break, tab becomes a tab; CR and unsupported XML/control characters fail.
Unicode and XML-sensitive characters are supported. Single-line plain text controls
reject LF. Empty strings are supported.

Only ordinary, unnested rich/plain text controls within one paragraph are supported,
including controls inside table cells. Whole table/row/cell wrappers, fields or
pictures inside scalar content, data-bound/unknown control properties, edit-locked
controls, document protection and tracked-change editing fail. The first content
character's run formatting is used for the complete replacement. Placeholder
controls use their declared run properties when present and clear placeholder
state/style. Surrounding paragraph/run formatting, tables, sections, headers,
footers, styles and unrelated media are retained within the tested fidelity scope.
Mixed formatting within a replaced scalar is not mapped character by character.
Inherited engine roundtrips normalize some style metadata/theme links; scalar
population adds no further style changes in the executed fixture. This is a
bounded fidelity finding, not complete preservation of all OOXML style metadata.

Scalar-only templates require `assets_dir=""`. The bounded Stage 4 operations
below add `rows:` and `image:` support. Stage 5 adds the bounded `repeat:` and
`if:` conventions documented below; qualification status is recorded separately.

`inspect_template(path) -> dict` and `preflight_template(path, context, assets_dir="") -> dict`
share the renderer's resolver. Inspection lists supported status and diagnostics;
preflight validates all scalar values before editing. Neither guarantees later
save success or visual layout. Errors use `foliorivet.TemplateError` with operation,
target, scope and an actionable message. Wrong Python argument types can
also raise standard Python `TypeError`.

## Stage 4 rows and existing chart assets

`rows:class_distribution` must wrap exactly one complete non-header data row in
a top-level body table. A separate static/header row must remain. The context
field `class_distribution` is an array of objects. Controls such as
`value:item.class_name` resolve against each object. Fields are unique within
each row prototype, so separate row targets may use the same item field names.
Prototype tags and structure are validated even for an empty array. Each actual
item must supply every required value. An empty array removes the prototype only;
static rows remain.

The model operation retains table/grid properties, row/cell properties and
paragraph/run styles. Prototype/item controls are consumed after population,
leaving editable row text without duplicated control identifiers. Static content
and unrelated controls remain. At most 1000 total data rows and 16 MiB of
serialized prototype copies are accepted, in addition to the 1 MiB context and
64 KiB value limits. This is a structural limit, not a performance claim.

Unsupported repeated-row features fail: multi-row/nested wrappers, nested tables,
vertical merges anywhere in the target table, cell wrappers, header prototypes,
numbered paragraphs, hyperlinks, tracked changes, sections, bookmarks, fields,
media and other inline objects. Ordinary horizontal cell spans are retained.
Content controls that must be consumed must permit deletion and editing.
`value:item.*` fields outside a supported row scope fail.

`image:training_results` must be an unnested picture content control containing
one uncropped inline image. Its existing dimensions define the bounding box;
the new raster image fits both dimensions with its aspect ratio preserved.
Supply `training_results: {"path": "training-chart.png"}` and `assets_dir` pointing
to the containing directory. Paths use forward slashes, must remain inside that
directory after canonicalization, and must identify regular files of at most
16 MiB. Decoding uses the existing image probe with a 64 MiB budget. SVG,
metafile, OLE, floating/cropped slots and pictures inside repeated rows are
outside this scope. Slot alt text and paragraph/run formatting are retained.
The picture tag is consumed; its untagged control remains editable.

Missing files fail even when `required` is false. To explicitly omit an optional
chart, supply `{"path": null, "required": false, "unavailable": "Chart unavailable"}`.
Python supplies the wording; Rust does not invent report semantics. The picture
is replaced with that text inside the same paragraph. `required` defaults to true.
Every image target needs an explicit input. Preflight accepts `assets_dir` and
performs the same private-copy validation as rendering, including image decoding.
It does not write output or establish save/layout success. Source assets may
change between preflight and rendering; rendering validates them again.

CLI commands `template.inspect`, `template.preflightWithAssets`,
`template.populateRowsAndImages` and `template.render` call the shared Rust code.
The original `template.populateScalars` retains its strict scalar-only contract.
Population failure leaves the caller's document unchanged; rendering refuses
existing output paths. A failed filesystem write can still leave a staging file.

The original synthetic [invoice example](examples/invoice/README.md) uses only
these public APIs, with all business amounts supplied in Python JSON input.
Executed qualification and pending Word review are recorded in
[`STAGE4.md`](../../docs/wordcraft/evidence/STAGE4.md).

CLI commands use the existing registry:

- `template.inspect`: inspect the current document.
- `template.preflight`: context object as command parameters.
- `template.populateScalars`: same context, modifies the current document with undo.
- `template.render`: `{template_path, context, assets_dir, output_path}`; a fresh
  session and create-new output semantics, shared with the native binding.

For `template.render` in a CLI script, paths must be absolute or relative to the
process working directory; the existing CLI's script-relative path rewriting does
not know these new parameters. Prefer absolute paths. Existing `--save`/`file.save`
commands retain their upstream overwrite behavior; use `template.render` for the
protected staging contract.

The native extension is CPython 3.12 specific; no abi3 or wider version support is
claimed. Linux wheels currently use a `linux_x86_64` tag and require the tested
native runtime libraries; manylinux distribution is not qualified.

## Stage 5 repeated and optional blocks

`repeat:class_training_results` wraps a complete logical body block, including
its heading, paragraphs, tables, chart slot and associated spacing. Its input
is an array of objects. `value:item.class_name` and `value:item.precision` read
fields from the current object. Each object's `training_results` supplies its
`image:training_results` slot. A `rows:details` target inside the block similarly
reads the current object's `details` array; its own `value:item.*` fields resolve
against each row object. Fields and targets must be unique within their scope.
No outer-item or global-field fallback is performed. Unused supplied fields fail.

`if:has_evaluation` requires an explicit JSON boolean. False removes the entire
wrapper content, including its heading and any page-break/spacing paragraphs
inside it. True keeps that content and populates its targets from the document
context. Omit values belonging only to a false block; supplying them fails as
unused input. Prototypes are checked even for empty arrays and false conditions.
Empty repeat input removes the complete prototype. If the entire body is removed,
the document retains the single empty paragraph required for an editable body.

Both variable wrappers must be ordinary unlocked rich-text controls around
complete top-level body blocks. Paragraph and table boundaries are supported.
Nested repeat/if blocks, wrappers in headers/footers or table cells, partial
paragraph boundaries, actual section breaks inside variable blocks, numbering,
bookmarks, fields, notes, comment anchors, hyperlinks, tracked changes, unknown
objects and background pictures are rejected. Complete tables may retain their
internal geometry and merges; nested row expansion retains Stage 4 restrictions.
Ordinary uncropped inline pictures reuse the media store; output drawing IDs
are assigned by the existing writer. Consumed repeated wrappers/item controls
are removed, leaving ordinary editable content and manual-comment areas.

Limits are 1000 repeated items across the document and a conservative 16 MiB
combined prototype/expanded-body budget, in addition to the earlier JSON,
scalar, row, image and nesting limits. These are bounds, not performance claims.
Preflight runs the same private-copy preparation as rendering, including asset
validation, but does not prove filesystem save success or visual compatibility.
Counts include populated scalars, rows and picture slots in repeated items.
CLI `template.populateBlocks` accepts `{context, assets_dir}` and supports undo;
`template.render`, inspection, preflight and the native Python API select the
same shared resolver automatically. The scalar-only command stays scalar-only.

The Stage 5 matrix and project-status example are in `tests/stage5_smoke.py`
and `examples/project-status/`. See `docs/wordcraft/evidence/STAGE5.md` for actual
qualification and Word review status. No production integration is implied.

## Qualification build inputs

For reproducible qualification builds, check out the repository's `craft-i18n.rev`
and set `CRAFT_I18N_DIR` to that checkout and `CRAFT_I18N_REQUIRED=1` before
building both the CLI and native wheel. Missing language data otherwise silently
reduces proofing/hyphenation capabilities. Linux layout tests also require a CJK
font such as the system-installed Noto Sans CJK. These inputs do not widen the
supported Python/platform scope. See `docs/wordcraft/evidence/remediation/README.md`.
