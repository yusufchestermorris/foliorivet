# Project status example

Original synthetic text, input, layout and generator: MIT OR Apache-2.0.
The editable template retains python-docx template/theme metadata under its
applicable terms. No font binaries are bundled. The synthetic template, output and preview are public example assets.

From a clean environment containing the locally built native wheel:

```text
python -I generate.py NEW_STAGING_OUTPUT.docx
```

The output parent must exist and the output must be new. `input.json` contains
precomputed presentation values. `repeat:workstreams` wraps the heading, metric
table and editable owner comments. `value:item.name` and `value:item.progress`
resolve separately for each workstream. `if:has_review` includes/removes both
review paragraphs. No project-specific Rust behavior or calculation is used.

Actual execution, preview and compatibility status are recorded in
`compatibility.json` when available and in the public release status.
Independent Word repair-prompt review remains a separate gate.

The inherited python-docx template metadata is covered by the MIT notice in
`../LICENSE-python-docx`. All names and amounts are synthetic.
