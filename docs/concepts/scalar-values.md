# Scalar Values (`value:`)

Scalar replacement is the most fundamental operation in FolioRivet. It replaces a content control's text with a dynamic value while preserving surrounding typography and paragraph layout.

---

## 1. Authoring Scalar Targets in Microsoft Word

To create a scalar target in Word:

1. Place your cursor where the text should appear, or highlight a placeholder word.
2. In the **Developer** tab, click either:
   - **Plain Text Content Control** (`Aa`)
   - **Rich Text Content Control** (`Aa`)
3. Click **Properties**.
4. Set the **Tag** to `value:[dotted_path]`, for example:
   ```text
   value:metadata.report_id
   ```
5. Ensure **Contents cannot be edited** is **unlocked** (unchecked).
6. Click **OK**.

---

## 2. Python Input Structure and Types

FolioRivet maps dotted paths in the tag directly to nested dictionaries or JSON objects:

```python
context = {
    "metadata": {
        "report_id": "REP-2026-X8",
        "generated_by": "Automated Pipeline",
        "version": 4,
        "is_draft": False
    }
}
```

### Supported Value Types

| Python Type | JSON Representation | Populated Text in Word | Notes |
|:---|:---|:---|:---|
| `str` | `"Alice"` | `Alice` | Full UTF-8 Unicode supported (accents, emojis, CJK characters). |
| `int` | `42` | `42` | Automatically converted to string representation. |
| `float` | `98.6` | `98.6` | Formatted to standard string representation. |
| `bool` | `True` / `False` | `true` / `false` | Serialised as lowercase string. |

### Unsupported Leaf Types (Preflight Rejection)

FolioRivet strictly rejects complex types at scalar targets:

```python
# ✗ REJECTED: None / null
context = {"customer": None}
# Raises: TemplateError: expected a string, number or boolean; preformat unavailable values in Python

# ✗ REJECTED: List / Array at scalar position
context = {"customer": ["Acme", "Corp"]}
# Raises: TemplateError: expected a string, number or boolean

# ✗ REJECTED: Dictionary / Object at scalar leaf
context = {"customer": {"first": "Jane"}}  # When tag is value:customer
# Raises: TemplateError: expected a string, number or boolean
```

> [!TIP]
> **Preformat in Python**
> If a value may be absent, empty, or requires specific decimal formatting (such as `£1,250.00` or `12.5%`), preformat the string in Python before passing it to FolioRivet. Do not expect the engine to format raw numbers or interpret `None`.

---

## 3. Multiline Values and Line Breaks

When populating multiline text (strings containing `\n`):

1. **Rich Text Content Controls** natively accept line breaks (`\n`) without special configuration.
2. **Plain Text Content Controls** in Microsoft Word default to single-line mode. If you attempt to populate a string containing `\n` into a single-line plain text control, preflight fails:
   ```text
   TemplateError: preflight: target="value:notes", scope=document: enable multiline on this plain text control to insert line breaks
   ```

### Enabling Multiline in Word
To enable multiline on a Plain Text Content Control in Word:
1. Select the control and click **Properties**.
2. Check the box labelled **Allow carriage returns (multiple paragraphs)**.
3. Click **OK**.

---

## 4. Typography and Formatting Preservation

When FolioRivet replaces text in a scalar control:

- **Run Formatting Inheritance**: The replacement text inherits the run formatting (font family, font size, bold, italic, colour, highlight) of the **first character** inside the control.
- **Placeholder Handling**: If the control displays Word's default grey placeholder text (*"Click or tap here..."*), FolioRivet removes the placeholder styling and reapplies any custom run properties configured for the control.
- **Surrounding Inlines**: Text, spaces, punctuation, footnotes, and inline formatting outside the content control boundaries are preserved exactly as laid out in Word.

---

## 5. Character Safety and Limits

- **Maximum String Length**: An individual scalar replacement can contain up to `65,536` bytes (64 KiB) of UTF-8 text.
- **Line Breaks and Tabs**: `\n` is converted to an OpenXML line break (`w:br`); `\t` is converted to a tab character (`w:tab`).
- **Forbidden Characters**: Null bytes (`\0`), XML control characters, and Unicode replacement markers (`\u{fffc}`, `\u{fffe}`, `\u{ffff}`) are rejected during preflight.
- **Empty Strings**: `""` is fully supported and cleanly clears the control text while keeping the paragraph geometry intact.

---

## 6. Strict Field Validation

FolioRivet enforces bidirectional field matching for scalar targets:

1. **Every declared template target must receive a value**:
   If the template contains `value:user.email` and `email` is missing from `context["user"]`, generation fails immediately.
2. **Every leaf field in context must map to a target**:
   If your Python dictionary contains `{"user": {"email": "...", "unused_token": "..."}}` and `value:user.unused_token` does not exist in the template, generation fails:
   ```text
   TemplateError: preflight: target="value:user.unused_token", scope=document: missing required target; add this tag to the template or remove the unused input field
   ```

This strictness guarantees that accidental typos in Python keys (`"emial"` instead of `"email"`) or deleted Word controls are caught immediately rather than silently failing in production.
