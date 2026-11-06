# Installation

This guide explains how to install the **FolioRivet 0.1.0a1** Python bindings and verify that the native extension functions correctly on your platform.

---

## System Requirements

Before installing FolioRivet, ensure your host environment meets the qualified requirements:

| Requirement | Specification | Notes |
|:---|:---|:---|
| **Python Runtime** | **CPython 3.12 x64** | CPython 3.12.0 or newer (up to 3.12.x). PyPy, 32-bit Python, and Python versions below 3.12 or above 3.12 are **not supported** in 0.1.0a1. |
| **Operating System** | **Windows x64** (Windows 10/11, Server 2019+) | Standard MSVC runtime environment. |
| **Operating System** | **Linux x64** (Ubuntu 22.04 LTS or newer) | Requires `glibc >= 2.35` (e.g. `manylinux_2_35_x86_64`). |
| **Microsoft Word** | **Not required for generation** | FolioRivet renders DOCX headlessly. Word is only needed by template designers who author or visually inspect templates. |
| **Rust Toolchain** | **Not required for binary wheels** | Precompiled wheels bundle the native Rust binary extension (`_native`). A Rust compiler is only needed if building from source. |

> [!IMPORTANT]
> **Public Preview Notice**
> FolioRivet 0.1.0a1 is an experimental alpha on PyPI. APIs and behavior may change; production adoption remains unqualified.

---

## Step 1: Create a Clean Virtual Environment

Always install FolioRivet in an isolated Python 3.12 virtual environment to prevent dependency conflicts.

=== "Windows (PowerShell)"

    ```powershell
    # Ensure you are using Python 3.12
    py -3.12 --version

    # Create a new virtual environment
    py -3.12 -m venv .venv

    # Activate the environment
    .\.venv\Scripts\Activate.ps1
    ```

=== "Ubuntu / Linux (bash)"

    ```bash
    # Ensure you are using Python 3.12
    python3.12 --version

    # Create a new virtual environment
    python3.12 -m venv .venv

    # Activate the environment
    source .venv/bin/activate
    ```

---

## Step 2: Install from PyPI

```bash
python -m pip install --only-binary=:all: foliorivet==0.1.0a1
```

For a locally built candidate, install its matching wheel with
`python -m pip install --no-index PATH_TO_WHEEL`. Source-build instructions are
in the repository's docs/release/BUILD.md. No sdist is qualified.

---

## Step 3: Verify the Installation

To confirm that the native CPython extension was installed and linked correctly:

1. Launch Python in isolated mode (`python -I`) outside of the source directory.
2. Import `foliorivet` and inspect its public symbols:

```powershell
python -c "import foliorivet; print('FolioRivet symbols:', foliorivet.__all__)"
```

Expected output:

```text
FolioRivet symbols: ['TemplateError', 'render_template', 'inspect_template', 'preflight_template']
```

Next, test that the underlying native module (`foliorivet._native`) loaded successfully:

```powershell
python -c "from foliorivet import _native; print('Native extension loaded from:', _native.__file__)"
```

If this prints a path inside your virtual environment's `site-packages` ending with `.pyd` (on Windows) or `.so` (on Linux), the native engine is linked and ready.

---

## Troubleshooting Installation Issues

### 1. `ERROR: *.whl is not a supported wheel on this platform`

**Causes:**
- You are running a Python version other than 3.12 (e.g. Python 3.11 or Python 3.13). FolioRivet 0.1.0a1 uses CPython 3.12-specific ABI bindings.
- You are using 32-bit (x86) Python instead of 64-bit (x64/AMD64).
- On Linux, your distribution has an older glibc version than 2.35 (e.g. Ubuntu 20.04).

**Resolution:**
Run `python -c "import platform, struct; print(platform.python_version(), struct.calcsize('P') * 8)"`. Ensure it outputs `3.12.x 64`.

### 2. `ImportError: DLL load failed while importing _native` (Windows)

**Cause:**
Missing Microsoft Visual C++ Redistributable runtime libraries on Windows.

**Resolution:**
Install the latest [Microsoft Visual C++ Redistributable (x64)](https://aka.ms/vs/17/release/vc_redist.x64.exe).

### 3. `GLIBC_2.35 not found` (Linux)

**Cause:**
Running the `manylinux_2_35_x86_64` wheel on a Linux distribution with an older C library (such as CentOS 7, Debian 10/11, or Ubuntu 20.04).

**Resolution:**
Deploy on Ubuntu 22.04 LTS or newer, or Debian 12 (Bookworm), which provide glibc 2.35+.

---

## Next Steps

Now that FolioRivet is installed, proceed to the [Quick Start Guide](quickstart.md) to generate your first document.
