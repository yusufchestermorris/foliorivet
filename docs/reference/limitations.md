# Known Limitations

FolioRivet supports a bounded set of native DOCX template operations. Development
changes below are unreleased; published 0.1.0a1 behavior is called out where it
differs. Support matrix and allocation audit reviewed 9 November 2026.

---

## 1. Template Structure & Nesting

- **Bounded Nesting**:
  The development checkout permits an outer `if:` containing immediate `repeat:` children with complete block boundaries. A false condition removes the subtree without requiring its exclusive data; supplied unused fields still fail strict validation. Repeats cannot contain variable blocks, and nested conditionals remain unsupported. A repeat can contain a `rows:` table or an `image:` slot. The published 0.1.0a1 rejects all nested variable blocks.
- **No Mid-Paragraph Block Controls**:
  Block controls (`repeat:` and `if:`) must wrap entire paragraphs and entire tables. Wrapping a few words or a sentence halfway through a paragraph is rejected.
- **Top-Level Body Only for Blocks**:
  `repeat:` and `if:` controls are supported only in the primary document body (`StoryRef::Body`). They cannot be placed inside headers, footers, footnotes, or inside table cells.

---

## 2. Table Limitations

- **No Vertical Cell Merges in Repeated Tables**:
  Vertical cell merges (`w:vMerge`) anywhere in a table containing a `rows:` prototype row are unsupported.
- **No Cell-Level Control Wrappers**:
  Content controls wrapping entire individual cells are rejected. Wrap either the entire row (`rows:`) or text runs inside cells (`value:item.*`).
- **No Nested Tables in Repeated Rows**:
  Prototype rows cannot contain inner nested tables.
- **Header Row Requirement**:
  A table cannot consist solely of a `rows:` prototype row. It must contain at least one static non-prototype row (such as a header row) so that if the data list is empty, a valid table remains.

---

## 3. Image Limitations

- **Inline Wrapping Only**:
  Picture replacement slots must use **In Line with Text** wrapping. Floating images (*Square*, *Tight*, *Behind Text*, *In Front of Text*) cannot be targeted by `image:`.
- **Uncropped Slots Only**:
  The placeholder picture in Word must be uncropped (`crop == [0,0,0,0]`).
- **Raster Images Only**:
  Only PNG and JPEG formats are supported for image replacement. Vector formats (SVG, EMF, WMF) and embedded OLE objects are not supported in 0.1.0.

---

## 4. Document-Level Features

- **Tracked Changes**:
  Documents with tracked changes (`w:trackRevisions`) enabled are rejected by preflight. All revisions must be accepted or rejected before using a document as a FolioRivet template.
- **Document Protection & Passwords**:
  Password-protected documents or read-only protected forms are unsupported.
- **Lists in Variable Blocks**:
  The development checkout permits native bullets with valid numbering definitions in conditional (`if:`) blocks, including style-inherited bullets and nested levels. Numbered lists remain unsupported there. All numbering, including bullets, remains unsupported in repeated blocks and repeated rows. The published 0.1.0a1 rejects bullets in conditionals as well.

---

## 5. Expression & Calculation Limitations

- **No Embedded Expressions**:
  FolioRivet does not evaluate arbitrary programming expressions, conditional math, filters, or formatters inside template tags. All mathematical aggregations, currency formatting, date transformations, and boolean evaluations must be performed in your Python application prior to calling FolioRivet.
- **Strict Bidirectional Key Mapping**:
  All declared template targets must receive values, and all leaf keys in your context dictionary must map to declared targets. Supplying extra, unused keys in your context raises an error.

## Allocation and aggregate budgets

Image assets have a separate 16 MiB encoded-file limit and a 64 MiB conservative
decode reservation. The reservation is the largest of native output plus
decoder-held storage, native output plus an RGBA8 conversion, and two RGBA8
buffers. It is not raw pixel size or a measurement of process memory. A
3571 × 2366 RGBA8 image reserves 67,591,888 bytes, exceeding the 67,108,864-byte
limit. Images are not implicitly resized or recompressed. Development diagnostics
report dimensions, representation, allocation components and the limit.

The 1000-item repeat and separate 1000-row budgets apply across the entire
render, including active conditional children, repeated-item row operations and
global row operations. Repeat items do not consume the row counter. For each
counter, 500 + 500 fits; 600 + 600 fails. Expansion shares a 16 MiB reservation
across these operations: prototype bytes times copies plus serialized populated
output bytes, including reservations in child operations. Nested reservations
are deliberately conservative and are not output ZIP size. Exactly the configured
limit fits; one additional byte fails. Errors name usage, limit and scope. These are distinct from the 1 MiB context and 64 KiB scalar limits.

Populated scalar controls may remain in the output. Variable wrappers are
consumed; a retained scalar control alone does not indicate unresolved content.


## Development support matrix

| Content | Conditional | Repeat | Repeated row |
|---|---|---|---|
| Text and supported scalar controls | Yes | Item-local | Item-local |
| Valid direct/style-inherited bullets | Yes | Rejected | Rejected |
| Numbered/unresolved lists | Rejected | Rejected | Rejected |
| Immediate repeat children | Yes | Rejected | Rejected |
| Conditional children | Rejected | Rejected | Rejected |
| Supported row prototypes / image slots | Yes | Yes | Rejected |

Scalar fan-out is explicitly opt-in; repeated `value:` tags are rejected by default.
Inspection includes every occurrence's story, paragraph path, offset and scope.
Populated document/conditional scalars retain their controls; repeat/row item
controls and variable wrappers are consumed.

### Pinned decoder allocation audit

Audited local Cargo.lock sources: `image 0.25.10`, its PNG adapter and JPEG adapter,
and `zune-jpeg 0.5.15`. The JPEG adapter copies encoded input into a `Vec` and
progressive decoding retains component coefficient buffers; the probe reserves
padded i16 coefficient planes for progressive JPEG. Baseline JPEG uses row/MCU
scratch storage. PNG expands palette pixels before reporting native output size.
The PNG adapter's `set_limits` checks dimensions but explicitly cannot update
internal PNG-reader allocation limits; constructing the reader with limits is
therefore also material. `Probe::decode` constructs a limited reader and retains
its pre-decode reservation check.

The reservation formula omits encoded-input copies, metadata and decoder scratch
allocations. It is a conservative estimate of the modeled pixel/conversion buffers,
not proof of a hard process-memory ceiling. Decoder-enforced limits are distinct
from corrupt data and the pre-decode reservation refusal. No decode-only budget
relaxation, resizing or recompression is introduced. Progressive-header tests
validate padded coefficient accounting. An original
complete progressive JPEG additionally covers successful decoding at the exact
reservation boundary, one-byte-under refusal, original media-byte preservation
and aspect-ratio preservation through native population and DOCX round-trip.
