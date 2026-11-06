# Frequently Asked Questions (FAQ)

---

### Does FolioRivet require Microsoft Word installed on the server?
**No.** FolioRivet is completely headless. The rendering engine is written in native Rust and directly parses and generates OpenXML (`.docx`) archives in memory. You do not need Microsoft Word, LibreOffice, Windows Server, or any desktop office software to generate documents.

---

### Can I use Jinja2-style syntax like `{{ name }}`?
**No.** Text-based templating engines often corrupt Microsoft Word documents because Word frequently splits text runs across multiple internal XML nodes (`<w:t>`). For example, Word might split `{{ name }}` into `<w:t>{</w:t><w:t>{ name</w:t><w:t>}}</w:t>`, causing regex replacers to fail.

FolioRivet uses native Microsoft Word **Content Controls** (`w:sdt`), which preserve OpenXML structure, protect character styles, and allow non-technical authors to edit templates safely in Microsoft Word.

---

### How does FolioRivet compare to other Python DOCX libraries?

| Feature | FolioRivet 0.1.0 | `python-docx` | `docxtpl` (Jinja2) | COM Automation |
|:---|:---:|:---:|:---:|:---:|
| **Engine Core** | Native Rust | Pure Python | Python + Jinja2 | MS Word Process |
| **Server Requirements** | Headless (None) | Headless (None) | Headless (None) | Windows + MS Office |
| **Template Syntax** | Word Content Controls | Manual Code / DOM | Text `{{ Jinja }}` | Office Macros / VBA |
| **Style Fidelity** | Preserves all Word styles | Basic styles | Often breaks styles | Full Word fidelity |
| **Preflight Validation** | Strict schema & limits | None | None | None |
| **Execution Speed** | Extremely fast (Rust) | Moderate | Moderate | Extremely slow |
| **Concurrency Safety** | Thread-safe & isolated | Thread-safe | Thread-safe | Not concurrency-safe |

---

### Why does `render_template` refuse to overwrite existing files?
To prevent accidental data loss in automated pipelines and race conditions in concurrent web workers, FolioRivet enforces `create_new(true)`. If a file already exists at the destination path, the operation aborts immediately. To overwrite an existing file, generate to a temporary staging path and use `Path.replace()` to atomically swap the file.

---

### Can I perform calculations or format currency inside the template?
**No.** FolioRivet intentionally keeps document templates purely declarative. All business calculations, rounding, currency symbol formatting (`£120.00`), and date formatting (`6 November 2026`) should be executed in your Python application code before passing the context to FolioRivet.

---

### Can I use Python 3.11 or Python 3.13?
FolioRivet 0.1.0 provides qualified wheels specifically for **CPython 3.12 x64**. Python versions below 3.12 or above 3.12 cannot load the prebuilt binary wheel due to CPython C-API ABI differences. Additional Python versions will be qualified in future releases.

---

### Is FolioRivet thread-safe?
**Yes.** Every invocation of `render_template`, `inspect_template`, and `preflight_template` constructs an isolated, independent Rust session. There is no shared global state or mutable cache, making FolioRivet safe to call across multiple worker threads or asynchronous task queues (such as Celery or RQ).

---

### Why does preflight fail when I provide extra keys in my dictionary?
FolioRivet enforces **strict bidirectional matching**. If your context dictionary contains keys that do not map to any template target, preflight raises an error. This prevents typos (such as `"custoemr"` instead of `"customer"`) or deleted template controls from silently swallowing data in production.
