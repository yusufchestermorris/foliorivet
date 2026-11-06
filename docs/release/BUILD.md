# Build and test a candidate

Use CPython 3.12 x64, maturin 1.10.2, Rust 1.95.0 and the pinned Cargo lockfiles.
Prepare native linker/development libraries; Windows requires an x64 Visual
Studio development shell. Check out craft-i18n at the exact craft-i18n.rev pin.
Keep that checkout unchanged. Linux qualification uses Ubuntu 22.04 and its
recorded font inputs; use native storage for staging/publication tests.

From an environment with the prepared tools on PATH:

```text
python tools/release/build_candidate.py --i18n /absolute/craft-i18n --output /new/candidate-directory
```

The default is an optimized release build. `--offline` requires all locked
sources cached. `--profile dev` creates an explicitly labelled development
candidate; the first local 6 November candidates used that profile. Logs and
hashes are retained; failures stop the build and no upload occurs. No sdist is
qualified. The Linux audit requests manylinux_2_35; do not relabel a failed wheel.

In a fresh CPython 3.12 environment, install the matching wheel with
`python -m pip install --no-index /path/to/wheel.whl`, then run:

```text
python -I tools/release/smoke_installed.py --examples bindings/python/examples --cli /path/to/foliorivet-cli --output /new/native-storage/smoke-results
```

Windows CLI uses `.exe`. Record OS/runtime/font versions and Office presence.
The smoke script executes all three examples, compares Python/CLI ZIP members,
checks template preservation and output collision refusal. It does not qualify
the real production pipeline, Word compatibility or every failure case.

Final acceptance also requires the native Rust and installed regression suites,
source/dependency/asset review, actual target-machine installation and a
user-followed quick start. Production shadow integration is deferred to adoption,
not required for the experimental preview. Keep the current renderer authoritative.
See PREVIEW.md for versioned build wrappers, artifact assembly and manual uploads.
