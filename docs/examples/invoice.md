# Example 2: Invoice & Quotation

This tutorial walks through the **Commercial Invoice & Quotation** example in `bindings/python/examples/invoice/`. It demonstrates dynamic table row expansion (`rows:`), multiline cell text, and financial calculations precomputed in Python.

---

## Business Scenario

A consulting agency generates formal client quotations and invoices:

- **Header Metadata**: Customer organisation name and quotation reference ID.
- **Line Items Table**: A dynamic table expanding to display varying numbers of billable items, quantities, hourly/unit rates, and total line amounts.
- **Financial Totals**: Subtotal, applicable VAT/tax, and grand total precomputed in Python.

---

## 1. Directory Structure

The example is located at `bindings/python/examples/invoice/`:

```text
invoice/
├── template.docx         # Word template with table row prototype
├── input.json            # Synthetic quotation data
├── generate.py           # Generation script
├── README.md             # Execution instructions
├── compatibility.json    # Compatibility audit record
└── generated.docx        # Pre-generated reference output
```

---

## 2. Template Structure in Microsoft Word

In `template.docx`, the document layout consists of header scalar controls, a dynamic table, and footer scalar controls:

```text
===========================================================
INVOICE / QUOTATION
===========================================================
Quote ID: [value:quote_id]
Customer: [value:customer]

-----------------------------------------------------------
TABLE:
[Header Row (Static)]
  Description           | Qty | Unit Price | Amount
-----------------------------------------------------------
[Prototype Row: rows:lines]
  [value:item.description] | [value:item.quantity] | [value:item.unit_price] | [value:item.amount]
-----------------------------------------------------------

Subtotal: [value:subtotal]
Tax:      [value:tax]
Total:    [value:total]
===========================================================
```

### Key Elements of the Table

1. **Header Row**: Static row containing column headers (*Description*, *Qty*, *Unit Price*, *Amount*). This row has Word's table header property enabled (`header: true`).
2. **Prototype Row**: Wrapped with a **Rich Text Content Control** tagged `rows:lines`.
3. **Inner Cells**:
   - Description cell: Rich Text control tagged `value:item.description`. (Uses Rich Text to permit multiline text).
   - Quantity cell: Rich Text control tagged `value:item.quantity`.
   - Unit Price cell: Rich Text control tagged `value:item.unit_price`.
   - Amount cell: Rich Text control tagged `value:item.amount`.

---

## 3. Input Data (`input.json`)

```json
{
  "quote_id": "SYN-0042",
  "customer": "Example Research Studio",
  "lines": [
    {
      "description": "Template preparation",
      "quantity": "2",
      "unit_price": "GBP 75.00",
      "amount": "GBP 150.00"
    },
    {
      "description": "Compatibility review — café",
      "quantity": "1",
      "unit_price": "GBP 120.00",
      "amount": "GBP 120.00"
    },
    {
      "description": "Documentation\nEditable handoff",
      "quantity": "1",
      "unit_price": "GBP 30.00",
      "amount": "GBP 30.00"
    }
  ],
  "subtotal": "GBP 300.00",
  "tax": "GBP 60.00",
  "total": "GBP 360.00"
}
```

### Noticeable Data Design Details

- **Currency and Numeric Formatting**: Formatted strings like `"GBP 75.00"` and `"GBP 360.00"` are constructed in Python. FolioRivet does not perform arithmetic or currency formatting.
- **Multiline Descriptions**: The third item contains `"Documentation\nEditable handoff"`. Because the control is Rich Text, the `\n` is converted to an in-cell line break without breaking table row layout.
- **Unicode Resilience**: The second item contains an em-dash and accented character (`"Compatibility review — café"`). FolioRivet preserves UTF-8 strings without encoding loss.

---

## 4. The Python Generator (`generate.py`)

```python
"""Generate Commercial Quotation Document."""

import json
from pathlib import Path
import sys
from foliorivet import preflight_template, render_template

here = Path(__file__).resolve().parent
template_path = here / "template.docx"
input_path = here / "input.json"
output_path = Path(sys.argv[1]) if len(sys.argv) > 1 else here / "output.docx"

context = json.loads(input_path.read_text(encoding="utf-8"))

# 1. Run Preflight
preflight = preflight_template(template_path, context)
print(f"Preflight passed: {preflight['row_count']} line items validated.")

# 2. Render Document
output_path.parent.mkdir(parents=True, exist_ok=True)
rendered = render_template(
    template_path=template_path,
    context_json=context,
    assets_dir="",
    output_path=output_path
)
print(f"Quotation written to: {rendered}")
```

---

## 5. Running and Testing Line-Item Expansion

Execute the script:

```powershell
python bindings/python/examples/invoice/generate.py staging/quotation.docx
```

Expected output:

```text
Preflight passed: 3 line items validated.
Quotation written to: .../staging/quotation.docx
```

### Zero-Row Test (Empty Array `[]`)
If you test passing `"lines": []`:
- The prototype row is removed.
- The table header remains.
- Preflight reports `row_count: 0`.
- The document remains structurally valid in Microsoft Word.

---

## 6. Table Layout Restrictions

- The target table cannot use vertical merges (`w:vMerge`).
- Prototype rows cannot be header rows.
- The table must retain at least one static non-prototype row.
- Cells cannot contain nested tables.
