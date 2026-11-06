# Training and evaluation example

Original synthetic content, layout and minimal OOXML authored for FolioRivet.
MIT OR Apache-2.0. No imported document template, proprietary assets or font
binaries. Regenerate the template with tools/release/make_training_example.py
only at a new destination; the builder refuses to overwrite it.

With the installed native wheel:

```text
python -I generate.py NEW_STAGING_OUTPUT.docx
```

The output parent must exist. Input metrics are precomputed in Python JSON.
`repeat:classes` includes class headings, metrics and editable reviewer notes;
`if:has_evaluation` includes or removes the complete evaluation section.
See compatibility.json for actual checks and pending Word review.
