# Known Limitations

To maintain mathematical determinism, high execution speed, and bulletproof output integrity, FolioRivet 0.1.0 operates within an explicitly bounded scope.

This document lists all known structural and functional limitations of the 0.1.0 release.

---

## 1. Template Structure & Nesting

- **No Nested Variable Blocks**:
  A repeating block (`repeat:`) cannot enclose another `repeat:` block or an `if:` block. An `if:` block cannot enclose another `if:` block or a `repeat:` block. (Note: A `repeat:` block *can* contain a `rows:` table or an `image:` slot).
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
- **Restarting Numbered Lists**:
  Complex auto-numbered list definitions with restart rules inside repeated blocks are unsupported. Use explicit Python-formatted numbering strings (e.g. `1.`, `2.`, `A.`) instead.

---

## 5. Expression & Calculation Limitations

- **No Embedded Expressions**:
  FolioRivet does not evaluate arbitrary programming expressions, conditional math, filters, or formatters inside template tags. All mathematical aggregations, currency formatting, date transformations, and boolean evaluations must be performed in your Python application prior to calling FolioRivet.
- **Strict Bidirectional Key Mapping**:
  All declared template targets must receive values, and all leaf keys in your context dictionary must map to declared targets. Supplying extra, unused keys in your context raises an error.
