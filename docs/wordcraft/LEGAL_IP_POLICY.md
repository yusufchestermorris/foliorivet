# Legal and IP compliance policy

Status: proposed project policy; not a legal-clearance finding. The pinned checkout and shipped artifacts must be audited before distribution. The upstream repository page identifies MIT and Apache-2.0 licenses, but that observation is not a file-level audit.

## Upstream preservation and distribution

Never remove, obscure or alter inherited copyright headers, LICENSE-MIT, LICENSE-APACHE or NOTICE contents. Preserve upstream authorship and identify FolioRivet as independent. Verify which licenses apply to each file and dependency at the pinned revision; do not assume every component has a dual-license choice. Retain MIT copyright/permission notices in copies or substantial portions. For Apache-covered distributions, supply the license, retain required notices, mark modified files and carry applicable NOTICE attribution. Check source archives, wheels, native libraries and embedded resources, not only manifests. Keep a third-party notice inventory with source, version/hash, license, modifications and inclusion location. Do not invent upstream NOTICE text or replace upstream licenses with this policy.

Sources: [MIT text](https://opensource.org/license/mit), [Apache-2.0 text, especially section 4](https://www.apache.org/licenses/LICENSE-2.0), [upstream repository](https://github.com/storytold/wordcraft). Consulted 4 November 2026. These obligations depend on the material actually used; this is operational guidance, not a legal opinion.

## Dependencies

Project policy permits new permissive dependencies only after checking actual compatibility and obligations (examples: MIT, Apache-2.0, BSD-3-Clause, ISC, Zlib). Review Rust and Python direct/transitive dependencies, build tooling and native runtime libraries. Flag GPL/AGPL/LGPL, proprietary, unknown or conflicting terms before adding; hold the addition pending a documented decision. Do not describe all copyleft licenses as universally incompatible or remove inherited components without assessing impact. Record inherited questions in a register. License scanning assists review and is not clearance by itself.

## Clean-room development

Reuse lawfully licensed Wordcraft engine code; do not rewrite it solely to satisfy a “from scratch” slogan. Genuinely new functionality must be independently authored using public documentation, open standards such as ECMA-376/ISO/IEC 29500 and lawfully obtained behavioral observations. Never reference, decompile, reverse-engineer or copy leaked, proprietary or closed-source commercial word-processor code. Keep source/provenance notes for nontrivial implementations. Do not copy standards text or schemas without checking their applicable terms. The spike adds no new desktop UI or layout engine; egui/wgpu are not requirements for the binding.

## UI, icons and trade dress

Do not reproduce Microsoft Word's distinctive appearance pixel for pixel, startup branding or exact visual motifs. Functional conventions are not blanket permission for copying presentation. Do not copy, trace or bundle Microsoft Office icons. Prefer independently drawn graphics or license-reviewed icon sets; verify the particular pack/version and preserve required notices rather than assuming a named collection is automatically usable.

## Typography and other assets

Never bundle or redistribute proprietary Microsoft font binaries (including Calibri, Aptos, Cambria, Segoe UI or Consolas) or introduce them as application defaults. Preserve font-name references already present in user templates; a name reference is different from distributing a font binary. Use host font discovery/fallback for local rendering. Bundled alternatives must have verified redistribution rights and required notices (possible candidates include Carlito, Caladea, Inter, Liberation and Noto; exact files/licenses require review). Check OFL reserved-name/modification conditions where applicable. Record installed fonts in fidelity evidence: substitutions can change layout.

Review templates, photos, charts, icons, screenshots, example data and embedded media. Public fixtures use synthetic/sanitized data with documented authorship and redistribution permission. User access to an asset does not establish redistribution rights. Never publish private report data or customer templates as examples.

## Naming and trademarks

FolioRivet and its repository/Python package names are proposals. Availability, conflicts, trademark clearance and ownership are unverified. Do not use Microsoft, Word, Office or their logos as branding, package names, titles or marketing in a way implying endorsement. Compatibility references must be accurate, descriptive and supported by evidence, such as “supports the tested Office Open XML (.docx) template features.” Attribute Wordcraft without implying its endorsement. Do not claim broad compatibility, patent clearance or legally established clean-room status based solely on upstream descriptions.

## ArtCraft brand assets — mandatory compliance

Follow the full [ArtCraft brand assets license](../brand/LICENSE-brand.txt), preserved unchanged from upstream. That license governs the ArtCraft name, wordmark and mark, including every file in `docs/brand/` in every color, size and format. Those marks belong to the ArtCraft Team, are not open source and are not covered by WordCraft's MIT or Apache-2.0 license. Neither NOTICE nor this policy grants rights to them. The requirements below implement the supplied brand license; they do not replace or expand its permission.

- The license grants limited, non-exclusive, non-transferable, revocable, royalty-free permission only as part of copies of the upstream repository and within WordCraft itself, its documentation and official builds, to identify WordCraft as made by the ArtCraft team and community and link to getartcraft.com and its community. Observe every condition in the license.
- Where use is permitted, use the marks exactly as supplied. Do not redraw, recolor, crop, stretch, rotate, animate, outline, add effects, extract parts or combine them with another name, mark or artwork.
- Use the marks only in WordCraft's context to identify its maker. Never suggest that the ArtCraft Team made, sponsors, endorses, certifies or supports FolioRivet or any other product, service, fork, modified version, plug-in, organization or person.
- Do not use the marks or the ArtCraft name in FolioRivet's name, logo, icon, domain, social media handle, app-store listing, advertising or marketing. The license reserves those uses for WordCraft.
- **Before distributing or publishing FolioRivet or any other modified or derived version, remove the ArtCraft Marks or replace them with independently owned or appropriately licensed branding. Do not call the result ArtCraft or present it as an ArtCraft product.** The permitted plain-text attribution is that the work is based on WordCraft by the ArtCraft team.
- The exception allowing unchanged marks in a fork kept only to propose changes back upstream does not cover FolioRivet's independent product purpose. Do not rely on that exception for a FolioRivet release.
- Any use outside the express permission requires prior written permission from the ArtCraft Team through [getartcraft.com](https://getartcraft.com/). Permission ends automatically upon breach; all other rights are reserved.

Before any distribution or publication, inspect the actual source archive, documentation, README, screenshots, generated previews, binaries, wheels, embedded resources, packaging and public listings for the marks and prohibited branding uses. Record each occurrence, removal or replacement and reviewer disposition in the provenance register. A local baseline retaining inherited files is not a distribution-compliance finding. Hold publication until the review is complete; retain upstream license texts, copyright notices and required attribution while removing or replacing the brand assets themselves.

## Review workflow

Before adding material, record origin, exact version/hash, license text, intended use, notices and reviewer/disposition. Before release, inspect the actual distribution contents and reconcile its dependency/asset inventory. Hold unresolved licensing or provenance issues out of distribution; seek qualified review where needed. Preserve evidence and identify unverified claims. This internal policy does not add restrictions to recipients' rights under upstream licenses.
