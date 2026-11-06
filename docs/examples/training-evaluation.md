# Example 1: Training & Evaluation Report

This tutorial walks through the **Training & Evaluation** example included in the FolioRivet repository. It demonstrates scalar replacements, repeated block sections, and conditional block inclusion.

---

## Business Scenario

An automated machine learning pipeline evaluates trained classification models and produces an official qualification summary document in Word format:

- **Model Overview**: Displays the model identifier and synthetic qualification metadata.
- **Class Metrics**: Repeats a structured section for each classified category, listing precision scores and editable reviewer commentary fields.
- **Evaluation Summary**: An optional section included only when validation benchmarks have been formally executed.

---

## 1. Directory Structure

The example is located at `bindings/python/examples/training-evaluation/`:

```text
training-evaluation/
├── template.docx         # Original Word template with Content Controls
├── input.json            # Synthetic input data
├── generate.py           # Generation script
├── README.md             # Execution notes
├── compatibility.json    # Verified compatibility audit record
└── generated.docx        # Pre-generated reference output
```

---

## 2. Template Structure in Microsoft Word

The template `template.docx` defines three target zones:

```text
===========================================================
MODEL EVALUATION REPORT
===========================================================
Model Name: [value:model]

-----------------------------------------------------------
[repeat:classes]
  Class: [value:item.name]
  Precision: [value:item.precision]
  Reviewer Notes: [Static text run remains editable]
[/repeat:classes]
-----------------------------------------------------------

[if:has_evaluation]
  Section: Formal Benchmark Evaluation
  Details: [value:evaluation]
[/if:has_evaluation]
===========================================================
```

### Control Breakdown

1. `value:model`: Plain Text Content Control for the model's name.
2. `repeat:classes`: Rich Text Content Control wrapping paragraphs 4 through 6 (heading, metrics, and editable notes).
3. `value:item.name`: Plain Text control inside `repeat:classes` reading `item["name"]`.
4. `value:item.precision`: Plain Text control inside `repeat:classes` reading `item["precision"]`.
5. `if:has_evaluation`: Rich Text Content Control wrapping the evaluation section.
6. `value:evaluation`: Plain Text control inside `if:has_evaluation`.

---

## 3. Input Data (`input.json`)

```json
{
  "model": "Synthetic classifier 01",
  "classes": [
    {
      "name": "Class A",
      "precision": "94.0%"
    },
    {
      "name": "Class B",
      "precision": "91.5%"
    }
  ],
  "has_evaluation": true,
  "evaluation": "Held-out accuracy: 92.8%. Illustrative data only."
}
```

Notice that:
- Calculations (e.g. `94.0%` precision, `92.8%` held-out accuracy) are precalculated in Python. FolioRivet does not evaluate percentages or math formulas.
- `has_evaluation` is an explicit boolean (`true`).
- When `has_evaluation` is `true`, `evaluation` must be provided.

---

## 4. The Python Generator Script (`generate.py`)

```python
"""Generate Training and Evaluation Report."""

import json
from pathlib import Path
import sys
from foliorivet import preflight_template, render_template

here = Path(__file__).resolve().parent
template_path = here / "template.docx"
input_path = here / "input.json"
output_path = Path(sys.argv[1]) if len(sys.argv) > 1 else here / "output.docx"

# 1. Load precomputed data
context = json.loads(input_path.read_text(encoding="utf-8"))

# 2. Run preflight validation
summary = preflight_template(template_path, context)
print("Preflight passed:", summary)

# 3. Ensure parent directory exists and render
output_path.parent.mkdir(parents=True, exist_ok=True)
rendered = render_template(
    template_path=template_path,
    context_json=context,
    assets_dir="",
    output_path=output_path
)
print(f"Generated: {rendered}")
```

---

## 5. Running the Example

Run the script from your terminal:

```powershell
python bindings/python/examples/training-evaluation/generate.py staging/evaluation-report.docx
```

Expected terminal output:

```text
Preflight passed: {'image_count': 0, 'included_optional_blocks': 1, 'repeated_items': 2, 'row_count': 0, 'save_verified': False, 'scalar_count': 6}
Generated: .../staging/evaluation-report.docx
```

---

## 6. Verifying the Generated Document

Open `staging/evaluation-report.docx` in Microsoft Word:

1. **Model Title**: `"Synthetic classifier 01"` replaces `[value:model]`.
2. **Repeated Sections**: Two complete class sections exist: one for `"Class A"` (`94.0%`) and one for `"Class B"` (`91.5%`). The editable reviewer note areas below each class are retained.
3. **Conditional Appendix**: The *Formal Benchmark Evaluation* section is present with `"Held-out accuracy: 92.8%. Illustrative data only."`.
4. **Clean OpenXML**: If you inspect the underlying XML, all `repeat:` and `if:` wrapper tags have been consumed, leaving standard Word paragraphs.

### Testing Condition Removal
Edit `input.json` and set `"has_evaluation": false`, and remove `"evaluation"`. Re-run the generator with a new output filename. Notice that the entire *Formal Benchmark Evaluation* section disappears completely without leaving any blank paragraphs or gaps.

---

## 7. Relevant Limitations

- `repeat:classes` cannot enclose another `repeat:` or an `if:` block.
- Class metric labels must use `value:item.*`.
- The template requires non-header text paragraphs for the block boundaries.
