# Authoring Your First Word Template

This guide walks you through creating a FolioRivet template from scratch in Microsoft Word. You will learn how to enable the Developer tab, insert native Content Controls, and assign tags that FolioRivet resolves.

---

## What are Word Content Controls?

Microsoft Word **Content Controls** are native structured document elements (often called "Structured Document Tags" or `w:sdt` in the underlying OpenXML). Unlike crude text-search placeholders like `{{ name }}` or `%NAME%`, Content Controls:

- Protect surrounding formatting, fonts, colours, and paragraph styles.
- Support rich metadata: titles, tags, locking attributes, and placeholder text.
- Can wrap words, complete paragraphs, table rows, and picture slots.
- Do not break when Word splits text across internal OpenXML text runs (`w:r`).

FolioRivet reads the **Tag** property of each content control to determine how to populate it.

---

## Step 1: Enable the Developer Tab in Microsoft Word

By default, Microsoft Word hides the Developer tab. To enable it:

1. Open Microsoft Word on Windows or macOS.
2. Click **File** in the ribbon, then click **Options** (bottom-left).
3. In the Word Options dialogue, select **Customize Ribbon** from the left sidebar.
4. In the right-hand column under **Main Tabs**, locate and check the box next to **Developer**.
5. Click **OK**.

You should now see the **Developer** tab in your top Word ribbon.

```text
[Home]  [Insert]  [Layout]  ...  [Developer]
                                   ├── [Controls Group]
                                   │     ├── Rich Text (Aa)
                                   │     ├── Plain Text (Aa)
                                   │     ├── Picture (Icon)
                                   │     ├── Properties
                                   │     └── Design Mode
```

---

## Step 2: Create the Template Document

1. Create a **Blank Document** in Word.
2. Add your static layout, for example:
   - Header: Company letterhead or organisation title.
   - Body text:
     ```text
     Welcome to the team, [Name Placeholder]!

     Your current assignment is [Role Placeholder].
     ```
3. Apply any desired formatting (e.g. bold font, custom colour, specific font size).

---

## Step 3: Insert and Configure Content Controls

Now we will convert the bracketed placeholders into FolioRivet template targets.

### Adding the Name Target (`value:recipient.name`)

1. Select the text `[Name Placeholder]`.
2. On the **Developer** tab, in the **Controls** group, click the **Plain Text Content Control** icon (represented by `Aa`). A box will wrap the selected text.
3. With the control selected, click **Properties** in the Controls group.
4. In the **Content Control Properties** dialogue:
   - **Title**: Enter `Recipient Name` (this is a human-friendly label for Word users).
   - **Tag**: Enter `value:recipient.name` (this is the **exact target** FolioRivet reads).
   - **Locking**: Ensure **Contents cannot be edited** is **UNCHECKED**. (FolioRivet cannot populate edit-locked controls).
5. Click **OK**.

### Adding the Role Target (`value:recipient.role`)

1. Select `[Role Placeholder]`.
2. Click the **Plain Text Content Control** icon (`Aa`).
3. Click **Properties**.
4. Set:
   - **Title**: `Recipient Role`
   - **Tag**: `value:recipient.role`
   - Ensure the control is unlocked.
5. Click **OK**.

> [!IMPORTANT]
> **Title vs Tag**
> Word allows you to set both a **Title** and a **Tag**.
> FolioRivet **only** uses the **Tag** attribute to match data fields. The Title is purely for your visual convenience in Word. Always enter your syntax (e.g. `value:recipient.name`) in the **Tag** field.

---

## Step 4: Customising Placeholder Text (Optional)

By default, Word displays *"Click or tap here to enter text."* as placeholder text. You can customise this:

1. On the Developer tab, click **Design Mode** (it highlights in grey).
2. Word will now show the boundary tags: `[Recipient Name] Click or tap here... [/]`.
3. Highlight the placeholder text and change it to something descriptive, e.g. `Enter full name`.
4. Click **Design Mode** again to turn it off.

When FolioRivet populates a control with active placeholder text, it automatically clears the placeholder flag and applies your replacement data using the formatting of the first character.

---

## Step 5: Save the Template

Save your file as a standard Microsoft Word Document:

- **File Name**: `welcome.docx`
- **File Format**: `Word Document (*.docx)`

Do not save as `.doc` (Word 97-2003) or `.docm` (macro-enabled). FolioRivet operates strictly on standard OpenXML `.docx` files.

---

## Step 6: Validate Your Template with Python

You can immediately verify that FolioRivet discovers your new targets using `inspect_template`:

```python
from pathlib import Path
from foliorivet import inspect_template

inspection = inspect_template(Path("welcome.docx"))
for target in inspection["targets"]:
    print(f"Target: {target['tag']} | Kind: {target['kind']} | Supported: {target['supported']}")
```

Expected output:

```text
Target: value:recipient.name | Kind: plainText | Supported: True
Target: value:recipient.role | Kind: plainText | Supported: True
```

If `supported` is `True` and the tags match your planned JSON paths, your template is ready for generation!
