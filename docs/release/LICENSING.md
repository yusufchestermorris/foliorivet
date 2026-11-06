# Fork licensing and distribution review

6 November 2026. Review in progress; no cleared release candidate yet.

FolioRivet is based on WordCraft by the ArtCraft team. Preserve upstream
LICENSE-MIT, LICENSE-APACHE, NOTICE, copyright headers and applicable attribution.
Modified files distributed under Apache terms need prominent change notices;
record fork changes and retain the upstream revision and source provenance.
See https://www.apache.org/licenses/LICENSE-2.0 section 4.

The bundled docs/brand/LICENSE-brand.txt requires removing/replacing ArtCraft
marks before distribution of a modified version. The upstream-contribution-only
exception does not apply to this independent product. Required copyright names
in notices must not be removed by blanket text substitution. Source, README,
images, binary resources, installers and package metadata all require inspection.

The earlier dependency scanner found eight packages without top-level notices:
alloc-stdlib, objc2, objc2-core-foundation, objc2-core-text, objc2-encode,
objc2-foundation, r-efi and wuff. A missing file in a crate archive is not proof
of absent licensing. Determine inclusion separately for Windows/Linux, inspect
pinned upstream sources and retain exact applicable texts with source hashes.
Do not infer clearance from Cargo's license field or from excluding a platform.

Fixtures must be original and redistributable. Keep private supplied reports,
Word-produced review files, personal paths, build caches and historical output
archives out of public artifacts. Preserve their hashes and findings privately.

Public artifacts remain held until actual inventory and required notice inclusion
are reconciled. This record tracks engineering compliance work, not legal advice.

## Current source-text findings

The Windows/Linux binding graph contains 232 normal/build package declarations;
the CLI graph contains 226. Neither contains objc2 or r-efi. Their absence is a
platform-graph disposition, not a clearance for Apple/UEFI distributions.
All included graph entries now have collected notice texts. The r-efi source
also contains its license text in AUTHORS; objc2's pinned LICENSE.md describes
additional SDK-derived licensing questions, retained outside these artifacts.

Pinned supplemental sources:
- https://raw.githubusercontent.com/nicoburns/wuff/78e67f52713c0a2c36bd083fea18ba029565c9ec/LICENSE
- https://raw.githubusercontent.com/dropbox/rust-alloc-no-stdlib/ae42d22078b98549e987d2f03d12df7b984fde47/LICENSE

Root LICENSE-MIT, LICENSE-APACHE and NOTICE contents are unchanged relative to
upstream after CRLF normalization. Modified tracked source carries change notices.
Eight mark files were archived and removed from the checkout; brand license text
is retained. Wheel inventories contain no named ArtCraft mark/logo files.
Binary-resource inspection, source-export review and exact obligation/inclusion
reconciliation remain release gates; these findings do not authorize upload.
