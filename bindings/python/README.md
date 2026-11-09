# Experimental native DOCX templates

FolioRivet 0.1.0a1 provides thin synchronous Python bindings to the WordCraft-based
Rust engine. CPython 3.12 x64, Windows and Linux glibc 2.35+ only. Word is not
required for generation. Install the matching binary wheel from PyPI:

```text
python -m pip install --only-binary=:all: foliorivet==0.1.0a1
python -I bindings/python/examples/training-evaluation/generate.py NEW_OUTPUT.docx
```

This README describes the current development checkout. Its scalar fan-out,
`analyze_context`, conditional bullets and bounded nesting additions are unreleased;
the PyPI command above installs the earlier 0.1.0a1 behavior. Build and install a
wheel from this checkout to use the additions. Published release identifiers have
not changed.

From a source checkout, preserve the templates and create the output parent first.
See examples/ for three synthetic templates, inputs, generators, reference outputs
and previews. See ../../docs/release/BUILD.md and ../../CONTRIBUTING.md for builds
and tests. No private fixture or report is required.

## Contract

`render_template(template_path, context_json, assets_dir, output_path, *, scalar_fanout=False) -> str`
is synchronous and creates a fresh document session per call. Paths accept strings
or `os.PathLike[str]`; context accepts JSON text or a mapping. The result is the
output path as supplied. The parent staging directory must exist. Existing paths,
including the template, are never replaced. A filesystem write failure may leave
an incomplete staging file: discard it and retry with a new path. Publication is
the caller's responsibility.

`value:metadata.model_id` resolves `context["metadata"]["model_id"]`. Field
segments contain ASCII letters, digits or underscores; each segment must be
nonempty and the total path is limited to 256 bytes. By default, tags must be
unique in their effective scope, including across body/header/footer stories for
document-level scalars. `scalar_fanout=True` permits repeated `value:` destinations
in that scope; each occurrence is counted and validated, and its formatting is
preserved. Images and variable wrappers retain unique-target requirements.
Every active target needs data; fields exclusive to a false conditional may be
omitted. Unused supplied leaves fail, catching deleted controls and misspelled
fields. Supply only data consumed by active template operations.
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

Scalar-only templates require `assets_dir=""`. The bounded row/picture operations
below add `rows:` and `image:` support. Block support adds the bounded `repeat:` and
`if:` conventions documented below; qualification status is recorded separately.

`inspect_template(path, *, scalar_fanout=False) -> dict` and
`preflight_template(path, context, assets_dir="", *, scalar_fanout=False) -> dict`
share the renderer's resolver. Inspection lists supported status and diagnostics;
preflight validates all scalar values before editing. Neither guarantees later
save success or visual layout. Errors use `foliorivet.TemplateError` with operation,
target, scope and an actionable message. Wrong Python argument types can
also raise standard Python `TypeError`.

## Rows and existing chart assets

`rows:class_distribution` must wrap exactly one complete non-header data row in
a top-level body table. A separate static/header row must remain. The context
field `class_distribution` is an array of objects. Controls such as
`value:item.class_name` resolve against each object. Scalar target uniqueness is
strict within each row prototype by default; opt-in fan-out stays inside that
row instance. Separate row targets may use the same item field names.
Prototype tags and structure are validated even for an empty array. Each actual
item must supply every required value. An empty array removes the prototype only;
static rows remain.

The model operation retains table/grid properties, row/cell properties and
paragraph/run styles. Prototype/item controls are consumed after population,
leaving editable row text without duplicated control identifiers. Static content
and unrelated controls remain. The document-wide ceiling is 1000 data rows,
separate from the 1000-repeat-item ceiling. Prototype-copy and populated-output
reservations share 16 MiB across all row/repeat operations, including nested
preparation; counters do not reset per item or conditional. These supplement the
1 MiB context and 64 KiB scalar limits and are not performance claims.

Unsupported repeated-row features fail: multi-row/nested wrappers, nested tables,
vertical merges anywhere in the target table, cell wrappers, header prototypes,
numbered paragraphs, hyperlinks, tracked changes, sections, bookmarks, fields,
media and other inline objects. Ordinary horizontal cell spans are retained.
Content controls that must be consumed must permit deletion and editing.
`value:item.*` fields outside a supported row or repeat-item scope fail.

`image:training_results` must be an unnested picture content control containing
one uncropped inline image. Its existing dimensions define the bounding box;
the new raster image fits both dimensions with its aspect ratio preserved.
Supply `training_results: {"path": "training-chart.png"}` and `assets_dir` pointing
to the containing directory. Paths use forward slashes, must remain inside that
directory after canonicalization, and must identify regular files of at most
16 MiB. The shared probe retains a 64 MiB conservative allocation reservation,
with diagnostics for native pixel representation/output, decoder-held storage,
conversion reservation, estimated peak and limit. The maximum of native output
plus decoder storage, native output plus RGBA8 conversion, and two RGBA8 buffers
is reserved. This estimates modeled buffers, not measured process memory; it is
not a hard process-memory ceiling. The 3571 × 2366 eight-byte floor is 67,591,888
bytes, above the 67,108,864-byte limit. No resizing or recompression is performed.
Budget refusal, decoder-enforced limits, corrupt data and file-size refusal have
distinct diagnostics. SVG,
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
Qualification scope is recorded in ../../docs/release/STATUS.md.

CLI commands use the existing registry:

- `template.inspect`: inspect the current document; optional `{scalar_fanout: true}` enables repeated scalar occurrences.
- `template.preflight`: context object as command parameters.
- `template.populateScalars`: same context, modifies the current document with undo.
- `template.analyzeContext`: `{context, scalar_fanout?: false}`; scoped context findings without asset decoding or output writing.
- `template.render`: `{template_path, context, assets_dir, output_path, scalar_fanout?: false}`; a fresh
  session and create-new output semantics, shared with the native binding.

For `template.render` in a CLI script, paths must be absolute or relative to the
process working directory; the existing CLI's script-relative path rewriting does
not know these new parameters. Prefer absolute paths. Existing `--save`/`file.save`
commands retain their upstream overwrite behavior; use `template.render` for the
protected staging contract.

The native extension is CPython 3.12 specific; no abi3 or wider version support is
claimed. Qualified Linux wheels use `manylinux_2_35_x86_64` and require glibc 2.35+
and the documented native runtime libraries. Initial Linux qualification uses
Ubuntu 22.04 under WSL2; it is not independent clean-host qualification.

## Repeated and optional blocks

`repeat:class_training_results` wraps a complete logical body block, including
its heading, paragraphs, tables, chart slot and associated spacing. Its input
is an array of objects. `value:item.class_name` and `value:item.precision` read
fields from the current object. Each object's `training_results` supplies its
`image:training_results` slot. A `rows:details` target inside the block similarly
reads the current object's `details` array; its own `value:item.*` fields resolve
against each row object. Scalar targets must be unique within their effective
scope by default; opt-in fan-out binds all matching destinations in that instance.
Variable wrappers and image targets remain unique. No outer-item or global-field
fallback is performed. Unused supplied fields fail.

`if:has_evaluation` requires an explicit JSON boolean. False removes the entire
wrapper content, including its heading and any page-break/spacing paragraphs
inside it. True keeps that content and populates its targets from the document
context. Omit values belonging only to a false block; supplying them fails as
unused input. Prototypes are checked even for empty arrays and false conditions.
Empty repeat input removes the complete prototype. If the entire body is removed,
the document retains the single empty paragraph required for an editable body.

Both variable wrappers must be ordinary unlocked rich-text controls around
complete body blocks. Paragraph and table boundaries are supported. An outer
top-level `if:` may contain multiple immediate `repeat:` children. Conditional-in-
conditional, conditional-in-repeat and repeat-in-repeat compositions remain
unsupported; the published 0.1.0a1 rejects all nested variable blocks. Inactive
prototypes are validated before context evaluation.

Conditionals permit direct/style-inherited native bullets with valid resolved
numbering levels, preserving numbering references and paragraph formatting.
Numbered lists, unresolved numbering and all numbering in repeated blocks/rows
remain unsupported. The published 0.1.0a1 also rejects conditional bullets.
Wrappers in headers/footers or table cells, partial paragraph boundaries, actual
section breaks, bookmarks, fields, notes, comment anchors, hyperlinks, tracked
changes, unknown objects and background pictures are rejected. Complete tables may retain their
internal geometry and merges; nested row expansion retains row restrictions.
Ordinary uncropped inline pictures reuse the media store; output drawing IDs
are assigned by the existing writer. Consumed repeated wrappers/item controls
are removed, leaving ordinary editable content and manual-comment areas. Scalar
controls in retained document/conditional content may remain in the output.

Limits are 1000 repeated items across the document and a conservative 16 MiB
combined prototype/expanded-output reservation shared with row operations, in
addition to the separate 1000-data-row ceiling and JSON, scalar, image and nesting
limits. For either count, 500 + 500 fits and 600 + 600 fails; exactly 16 MiB fits
and one extra reserved byte fails. These are bounds, not performance claims.
Preflight runs the same private-copy preparation as rendering, including asset
validation, but does not prove filesystem save success or visual compatibility.
Counts include populated scalars, rows and picture slots in repeated items.
CLI `template.populateBlocks` accepts `{context, assets_dir, scalar_fanout?: false}` and supports undo;
`template.render`, inspection, preflight and the native Python API select the
same shared resolver automatically. The scalar-only command stays scalar-only.

The public native/CLI regression runs all three examples; see tests/README.md.

## Build inputs

Use pinned craft-i18n data with CRAFT_I18N_REQUIRED=1 for consistent proofing.
See ../../docs/release/BUILD.md. Production qualification is outside alpha scope.

## Development checkout: alpha feedback options

`inspect_template`, `preflight_template` and `render_template` accept keyword-only
`scalar_fanout=False`. Enable it to bind one scalar leaf to every `value:`
occurrence in its effective scope, preserving each destination's formatting.
Item fields remain local to their row/repeat instance. Images and variable
wrappers still require unique tags. Every destination is validated before edits;
existing output files are always refused.

```python
from foliorivet import analyze_context, inspect_template

inspection = inspect_template("template.docx", scalar_fanout=True)
findings = analyze_context("template.docx", {"show_details": False}, scalar_fanout=True)
```

`analyze_context(template_path, context_json, *, scalar_fanout=False)` returns
`active_required_paths`, `inactive_paths`, `missing_paths`, `unused_supplied_paths`
and `diagnostics`. Supplied repeat/row items use indexed paths such as
`classes[0].name`. Uninstantiated inactive item fields use `classes[].name`.
Missing/invalid conditions leave descendant activity undetermined; they do not
mark those paths inactive. Ordinary context findings are returned; malformed JSON
and unsupported template structure raise `TemplateError`. Analysis opens no
assets and writes no output. Strict preflight still decodes required images.

CLI commands accept `scalar_fanout` alongside their existing structured parameters;
`template.inspect` takes `{scalar_fanout: true}` and `template.analyzeContext`
takes `{context, scalar_fanout?: false}`. The legacy raw-context commands
`template.preflight` and `template.populateScalars` retain strict behavior. Use
`template.preflightWithAssets` or `template.populateRowsAndImages` with
`{context: {...}, assets_dir: "", scalar_fanout: true}` for scalar-only jobs with
fan-out, preserving compatibility with raw context fields named `scalar_fanout`.
Rust callers use `TemplateOptions` and the `*_with_options` entry points; existing
entry points retain strict defaults.

These additions are development source changes, not changes to the published wheel.


Context analysis also checks active consumers across included conditional sections.
Duplicate scalar destinations in that effective scope produce scoped findings in
strict mode and are permitted with `scalar_fanout=True`. A condition or other
consuming wrapper cannot share its context path with a scalar, even with fan-out.
Invalid context keys are reported without hiding unused fields in later siblings.
