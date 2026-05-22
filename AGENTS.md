# Toolé — Agentic Context

> Conforme aux standards de l'Agentic AI Foundation (AAF).
> Ce fichier est la source unique de vérité pour tout agent IA travaillant sur ce codebase.

---

## 1. Project Identity

- **Name**: Toolé
- **Purpose**: P2P local file transfer over LAN, no internet required
- **Language**: C (backend) + Python/CustomTkinter (UI)
- **Standard**: C11, Python 3.14
- **License**: MIT, 2026 Burkina Open Place
- **Repository**: `https://github.com/anomalyco/Toole`

---

## 2. Agent Directives

### 2.1 Immutable Rules

These rules MUST NOT be violated under any circumstances:

1. **Do NOT remove, modify, or relocate** any `Hello le BOP` comment. There are 25 occurrences across 16 files. They are team signposts documenting intent and design decisions.
2. **Do NOT write new files** when editing existing ones is possible.
3. **Do NOT update git config, force-push, use `-i`, create empty commits, or commit secrets.**
4. **Do NOT create README or documentation files** unless explicitly requested.
5. **Do NOT use emojis** in code or documentation unless explicitly asked.
6. **Do NOT add explanatory comments** to code. The existing `Hello le BOP` comments are the exception — they must stay but you may add new ones if truly valuable.

### 2.2 Always Do

1. **Before editing**: Understand the file's conventions, mimic code style, use existing libraries and utilities.
2. **Before committing**: Run `git status`, `git diff`, `git log --oneline -10`; stage only intended files.
3. **After implementation**: Run lint/typecheck if available (`cmake --build build` for C, `python -m py_compile` for Python).
4. **For errors**: Return negative error codes in C (0 = success). Use `(bool, str)` pattern in Python.
5. **For threads**: Always lock `devices[]` mutex before access. Use the platform's mutex abstraction (`toole_mutex_*`).

---

## 3. Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Python UI Layer                       │
│  app.py → screens/{connection,transfer,reception}.py    │
│         → controller/controller.py                      │
│         → bridge_ctypes.py (ctypes loader)              │
└──────────────────────┬──────────────────────────────────┘
                       │ ctypes
┌──────────────────────▼──────────────────────────────────┐
│                  C Bridge Layer                          │
│  src/core/bridge.c → libtoole_bridge.so                 │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│                  C Core Layer                            │
│  src/core/app.c (state machine)                         │
│  includes/*.h (contract interfaces)                     │
└──────────────────────┬──────────────────────────────────┘
                       │
┌──────────────────────▼──────────────────────────────────┐
│               C Platform Layer                           │
│  src/platform/linux/{discovery,network,                  │
│                      file_transfert,server_runtime}.c    │
│  src/platform/windows/*.c (blocked)                     │
└─────────────────────────────────────────────────────────┘
```

### 3.1 Three-Layer Network Protocol

| Layer | Protocol | Port | Purpose |
|---|---|---|---|
| 1 — Discovery | UDP broadcast | 47272 | Beacon every ~1s, `toole\|id\|...\|message` pipe-delimited |
| 2 — Control | TCP | 42422 (default) | HELLO, HEARTBEAT, MASTER_ANNOUNCE, RELAY_REQUEST/RESPONSE, ELECTION |
| 3 — File Transfer | TCP (same socket) | — | `file_struct` (name_len+file_size), chunks (4096B), CRC32 trailer |

### 3.2 State Machine

```
State: DISCOVERING → CLIENT → ELECTION → (CLIENT or MASTER)
                    → MASTER
Initial: DISCOVERING or MASTER (configurable)

DISCOVERING:
  - Master found → connect + HELLO → CLIENT
  - Beacon interval + 200ms AND smallest ID → become_master() → MASTER

CLIENT:
  - Heartbeat every ~1.5s, poll for MASTER_ANNOUNCE
  - Timeout/error → ELECTION

MASTER:
  - Broadcast MASTER_ANNOUNCE every ~1s
  - Accept clients, handle RELAY_REQUEST, detect file transfers

ELECTION:
  - Smallest ID wins (lexicographic string comparison)
  - Winner → MASTER, losers → reconnect → CLIENT
  - Fail → DISCOVERING
```

### 3.3 Master Election

- **Deterministic**: `elect_master_smallest_id()` compares node IDs lexicographically
- **Triggered by**: client timeout detecting master loss
- **Algorithm**: `runtime_elect_master_from_devices()` — collect self + device list, elect smallest ID
- **Failover**: `runtime_client_failover()` — direct reconnect to winner, or via neighbor relay

---

## 4. Build System

### 4.1 Root CMake (`CMakeLists.txt`)

- Produces **3 libraries** (no executable):
  - `toole_platform` (STATIC) — platform sources + pthread
  - `toole_core` (STATIC) — `src/core/app.c`
  - `toole_bridge` (SHARED) — `src/core/bridge.c` → `build/libtoole_bridge.so`
- **Windows**: blocked by `if(WIN32) message(WARNING ...) return()`

### 4.2 Test CMake (`tests/CMakeLists.txt`)

- Standalone project, rebuilds platform sources as `toole_platform_test`
- 4 test executables: `discovery_test`, `network_test`, `file_transfer_test`, `server_runtime_test`

### 4.3 Commands

```bash
# Build libraries
cmake -S . -B build && cmake --build build

# Build and run tests
cmake -S tests -B build/tests && cmake --build build/tests && ctest --test-dir build/tests

# Run UI (dev)
PYTHONPATH=src/ui .venv/bin/python src/ui/app.py

# Release (Linux standalone)
pip install pyinstaller
cmake -S . -B build && cmake --build build
pyinstaller --onefile --name toole \
  --add-data "$(pwd)/src/ui/logo.png:." \
  --add-data "$(pwd)/src/ui/logo.ico:." \
  --add-data "$(pwd)/build/libtoole_bridge.so:." \
  --distpath dist src/ui/app.py
mkdir -p releases/toole-$VERSION-linux
cp dist/toole releases/toole-$VERSION-linux/
tar czf releases/toole-$VERSION-linux.tar.gz -C releases toole-$VERSION-linux
rm -rf releases/toole-$VERSION-linux
```

---

## 5. C Coding Conventions

- **Naming**: `snake_case` for functions and variables
- **Headers**: include guards `#ifndef FILENAME_H / #define FILENAME_H`
- **Error handling**: return negative for errors, 0 for success; set `last_error` string in bridge
- **Memory**: manual `malloc`/`free` with NULL checks
- **Comments**: French; `// Hello le BOP` markers are immutable
- **Thread safety**: mutex protects `devices[]` across discovery thread and main thread
- **Network I/O**: robust `write_n()`/`read_n()` loops with `EINTR` handling; `MSG_PEEK` for traffic type detection

### 5.1 Error Codes (bridge.h)

| Constant | Value | Meaning |
|---|---|---|
| `TOOLE_BRIDGE_OK` | 0 | Success |
| `TOOLE_BRIDGE_ERR_INVALID_ARG` | -1 | Null pointers, bad values |
| `TOOLE_BRIDGE_ERR_INIT` | -2 | `app_init` failed |
| `TOOLE_BRIDGE_ERR_RUNTIME` | -3 | `app_start`/`tick`/connect failed |
| `TOOLE_BRIDGE_ERR_IO` | -4 | File send/receive failed |

### 5.2 File Transfer Protocol

```
send_struct: [name_len(uint32)][file_size(uint64)][filename(name_len bytes)]
send_file:  [chunk(4096B)]... [crc32(uint32)]
recv_file:  receives name, size, chunks, verifies CRC32 → deletes on mismatch
```

Path traversal is rejected via `is_safe_filename()` (blocks `/`, `\`, `..`, control chars). Duplicate filenames get `_1`, `_2`, ... suffixes via `build_unique_path()`.

---

## 6. Python Coding Conventions

- **Naming**: `CamelCase` classes, `snake_case` methods/variables
- **Types**: full type hints (Python 3.10+ syntax)
- **Private**: underscore prefix (`_make_config`, `_peer_label`)
- **Error handling**: try/except with fallbacks; `(bool, str)` return pattern
- **UI**: CustomTkinter event-driven, no global state, responsive layout via `<Configure>` events

### 6.1 Bridge Loading (bridge_ctypes.py)

Search order for `libtoole_bridge.so`:
1. `sys._MEIPASS` (PyInstaller bundle)
2. `TOOLE_BRIDGE_PATH` env var
3. `build/libtoole_bridge.so`
4. `./libtoole_bridge.so`
5. Same directory as script

### 6.2 Theme (theme.py)

```python
THEME = {
    "bg_dark": "#0B0F19",
    "bg_card": "#152238",
    "bg_subcard": "#1E2D4A",
    "border": "#243B5A",
    "primary": "#6366F1",
    "primary_hover": "#4F46E5",
    "success": "#10B981",
    "success_hover": "#059669",
    "warning": "#F59E0B",
    "danger": "#EF4444",
    "danger_hover": "#DC2626",
    "text_main": "#F3F4F6",
    "text_muted": "#9CA3AF",
    "text_dim": "#6B7280",
}
FONT_FAMILY = "Segoe UI"
```

---

## 7. Tests

All tests use `fork()` for client/server pairs on localhost.

| Test | Coverage | Method |
|---|---|---|
| `discovery_test` | UDP beacon send + receive | Fork: child sends beacon, parent hears and validates |
| `network_test` | TCP HELLO/HB/RELAY/ELECTION | Fork: client sends all message types, server validates |
| `file_transfer_test` | File transfer CRC32 + path traversal rejection | Fork: client sends file, server receives, binary comparison |
| `server_runtime_test` | Election, timeout, failover reconnect | 3 sub-tests: election picks smallest id, client detects timeout, failover reconnects |

Test output uses `[STEP][OK]/[STEP][KO]` format with `printf`.

---

## 8. Release Artifacts

- **Linux**: `releases/toole-v{version}-linux.tar.gz` containing `toole` (PyInstaller standalone binary) + `install.sh`
- **Windows**: `Toole.exe` (PyInstaller one-file, when backend is synchronized)
- **Location**: `releases/` directory, plain archives (no extracted folders)
- **Content**: No build steps required by end user — the binary is fully self-contained

---

## 9. Important File Paths

| Path | Purpose |
|---|---|
| `CMakeLists.txt` | Root build (libraries only, Windows blocked at line 9) |
| `tests/CMakeLists.txt` | Standalone test build |
| `includes/bridge.h` | Public C API contract (ctypes interface) |
| `includes/states.h` | Role/state enums, `info` struct |
| `src/core/app.c` | State machine, main loop |
| `src/core/bridge.c` | C↔Python bridge wrapper |
| `src/platform/linux/` | Linux platform implementation |
| `src/platform/windows/` | Windows stubs (blocked, incompatible API) |
| `src/ui/app.py` | Main window, runtime loop |
| `src/ui/bridge_ctypes.py` | ctypes loader, Bridge class |
| `src/ui/controller/controller.py` | Python backend controller |
| `src/ui/theme.py` | Theme colors and fonts |
| `src/ui/screens/` | UI tabs (connection, transfer, reception) |
| `dist/toole` | Latest PyInstaller standalone build |
| `releases/` | Release archives |
| `CONTRIBUTING.md` | Contribution guide |
| `AGENTS.md` | This file |

---

## 10. .gitignore

```
/build
__pycache__/
.venv/
/dist
received/
compile_commands.json
*.spec
```

---

## 11. Quick Reference — Common Operations

```bash
# Full rebuild
cmake -S . -B build && cmake --build build

# Run all tests and show verbose output
cmake -S tests -B build/tests && cmake --build build/tests && ctest --test-dir build/tests -V

# Run UI with bridge
PYTHONPATH=src/ui .venv/bin/python src/ui/app.py

# Build standalone release binary
pyinstaller --onefile --name toole \
  --add-data "$(pwd)/src/ui/logo.png:." \
  --add-data "$(pwd)/src/ui/logo.ico:." \
  --add-data "$(pwd)/build/libtoole_bridge.so:." \
  --distpath dist src/ui/app.py
```
