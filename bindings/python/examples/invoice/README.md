# Editable synthetic quotation

Install the locally built CPython 3.12 wheel, then run from any directory:

```console
python -I /path/to/bindings/python/examples/invoice/generate.py /path/to/staging/quotation.docx
```

The parent output directory must exist and the output file must be new.
`template.docx` is editable; preserve the `value:` and `rows:` tags when editing.
`input.json` contains synthetic line items and precomputed/preformatted totals.
The script uses only public inspection/preflight/render APIs; Rust contains no
invoice-specific code or business calculations. Manual comments remain editable.

Original generator, template layout, synthetic data and text are authored for
FolioRivet under MIT OR Apache-2.0. The template uses font-name references only;
no font binaries are bundled. The synthetic reference output and document-only preview are included for comparison.

Compatibility observations are recorded in `compatibility.json`.
An OOXML/ZIP check alone does not establish Word compatibility. Word repair-dialog
review and supported-platform scope must be read with the executed evidence.

The inherited python-docx template metadata is covered by the MIT notice in
`../LICENSE-python-docx`. All names and amounts are synthetic.
