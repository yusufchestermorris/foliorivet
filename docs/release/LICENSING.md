# Licenses and attribution

FolioRivet is based on WordCraft by the ArtCraft team. Preserve LICENSE-MIT,
LICENSE-APACHE, NOTICE, upstream copyright headers and ATTRIBUTION.md. Retain
prominent change notices on modified Apache-covered files. These upstream terms
are not replaced by this guide. See also docs/brand/LICENSE-brand.txt: ArtCraft
marks are separately licensed and are not FolioRivet branding.

Dependency and embedded-language-data notices are tracked under
bindings/python/python/foliorivet/licenses and bundled with Python wheels.
packaging/cli-notices contains the CLI-only supplement; CLI bundles include both
sets. Inventories describe the Windows/Linux normal/build declaration supersets,
not a claim that every listed dependency is linked into every binary. Reconcile
new dependency/platform additions with their exact source notices. Existing
macOS/UEFI exclusions do not qualify those distributions.

Duplicate dependency notice files were removed only when they were byte-identical
to an inventoried file in the same package directory. Every inventoried filename,
notice text and hash is retained. Upstream root license and NOTICE texts are intact.

All three examples contain synthetic content. Original text, code and layouts
are MIT OR Apache-2.0. Invoice/project-status templates retain python-docx default
template metadata; its MIT notice is included at
bindings/python/examples/LICENSE-python-docx (source:
https://github.com/python-openxml/python-docx/blob/v1.2.0/LICENSE).
No font binaries, private reports or Word-resaved review copies are included.
Named fonts are references, not bundled software. See ATTRIBUTION.md for asset
sources and applicable terms. Do not remove attribution as part of cleanup.

This records engineering provenance and notice handling, not a legal opinion or
an additional license restriction. No separate human legal-sign-off workflow is
an alpha release gate. Required redistribution obligations still apply.
