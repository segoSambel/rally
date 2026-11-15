# Rally

![Rust](https://img.shields.io/badge/Rust-2024_edition-000000?style=flat&logo=rust&logoColor=white)
![GPUI Kit](https://img.shields.io/badge/gpui--kit-0.7-6366f1?style=flat)
![Windows](https://img.shields.io/badge/Windows-x64-0078D6?style=flat&logo=windows&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green?style=flat)

---

## Overview

Rally is a lightweight desktop API client, in the spirit of Postman and Bruno but deliberately small: one window, no accounts, no cloud. It is written in Rust with [GPUI](https://www.gpui.rs) (through `gpui-kit`).

Rally is developed and tested on Windows, cross-compiled from WSL with `cargo-xwin`. Linux and macOS are the next targets: Rally's own code has only a few platform-specific details, all guarded, and a [CI workflow](.github/workflows/ci.yml) builds on all three systems. Linux and macOS have not been run by hand yet.

The workspace is split into three crates:

- **`rally-core`** — everything that is not UI: request and response types, the HTTP transport, URL and auth building, history storage and the console log. It has no GPUI dependency.
- **`rally-ui`** — the GPUI app: theme, request editors, tabs, history sidebar and console panel.
- **`rally`** — a thin binary that starts the UI and embeds the app icon.

---

## Features

- **Requests** — `GET`, `POST`, `PUT`, `PATCH` and `DELETE` from a single bordered method and URL field. Requests run on GPUI's background executor with a blocking [`ureq`](https://crates.io/crates/ureq) client and a 30 second timeout, so the window never blocks.
- **Query params** — a key/value table with per-row enable checkboxes and descriptions. Enabled rows are percent-encoded and appended to the URL, keeping any existing query and `#fragment`.
- **Headers** — the same table, with autocomplete for header names (25 common ones) and for values (14 headers, for example `Content-Type` or `Accept-Encoding`).
- **Authorization** — None, Bearer Token, Basic Auth and API Key (sent as a header or a query parameter). A header you set yourself always wins over the one the auth scheme generates.
- **Body** — None, JSON, Text and Form (`application/x-www-form-urlencoded`). The matching `Content-Type` is added when there is a body, unless you set one.
- **Response viewer** — a read-only code editor with line numbers, folding, soft wrapping and JSON syntax highlighting (tree-sitter). JSON bodies are pretty-printed. A Headers tab, the status, duration and size, and a copy button that copies what the active tab shows.
- **Tabs** — several requests open at once. Each tab keeps its own request, response and selected panes, and a request in flight finishes into the tab that sent it.
- **History** — the last 100 requests are saved and shown in a sidebar. Clicking an entry restores the whole editor.
- **Console** — a bottom panel with the raw request and response of every request sent this session (last 200), error details for failures such as DNS or TLS errors, and **Copy cURL** to repeat a request in a terminal. It lives in memory only.
- **Themes** — a dark theme by default and a light theme, with an indigo accent. Toggle from the status bar.

---

## Architecture

```
                 ┌────────────────────────────────────────────┐
  rally.exe      │ rally  (binary: window icon, rally_ui::run)│
                 └──────────────────────┬─────────────────────┘
                                        │
                 ┌──────────────────────▼─────────────────────┐
                 │ rally-ui  (GPUI)                           │
                 │  client/   tabs, request and response view │
                 │  widgets/  key/value table, auth, body     │
                 │  history/  sidebar     console/  panel     │
                 │  theme.rs  Rally light and dark themes     │
                 └──────────────────────┬─────────────────────┘
                                        │
                 ┌──────────────────────▼─────────────────────┐
                 │ rally-core  (no GPUI)                      │
                 │  request ─► transport (ureq) ─► response   │
                 │  draft, auth, url, format, headers         │
                 │  history (JSON on disk)   console (memory) │
                 └────────────────────────────────────────────┘
```

Editor state is held as a `RequestDraft`: what the user typed, not what is sent. The same draft is used to build the request, to save history and to switch tabs, so what is sent and what is stored cannot drift apart.

### Module Breakdown

| Module | Responsibility |
|--------|----------------|
| `rally-core::request` / `response` | `Method`, `ApiRequest` and `ApiResponse` |
| `rally-core::transport` | The blocking `send` call over `ureq` |
| `rally-core::draft` | `RequestDraft`, key/value rows, auth and body drafts, and building the request |
| `rally-core::auth` / `url` | Auth headers (Bearer, Basic, API key) and query-string building |
| `rally-core::history` | Capped, atomic JSON history file in the user data directory, with the location chosen per platform |
| `rally-core::console` | The in-memory request log and the cURL export |
| `rally-core::format` / `headers` | Body, size and duration formatting; header name and value suggestions |
| `rally-ui::client` | The root view, request tabs, and the tab strip and status bar |
| `rally-ui::widgets` | Key/value table with autocomplete, auth editor, body editor, method select |
| `rally-ui::history` / `console` | The sidebar and the bottom panel with their models |
| `rally-ui::theme` | Loads `assets/theme/rally.json` and starts in dark mode |

---

## Prerequisites

### Cross-compiling to Windows (Linux or WSL)

This is how Rally is developed today.

| Requirement | Notes |
|-------------|-------|
| Rust (stable) | Edition 2024, so Rust 1.85 or newer, with the `x86_64-pc-windows-msvc` target installed |
| [`cargo-xwin`](https://github.com/rust-cross/cargo-xwin) | Cross-compiles for MSVC. On first use it downloads the Windows CRT and SDK (about 1 GB) into `~/.cache/cargo-xwin` |
| `clang` and `lld` | Used by `cargo-xwin` to compile C dependencies and to link |
| `llvm` | Provides `llvm-rc`, which embeds the app icon at build time |
| Windows 10 or newer | To run the result. WSL can launch it directly |

On Debian or Ubuntu:

```bash
sudo apt install clang lld llvm
rustup target add x86_64-pc-windows-msvc
cargo install --locked cargo-xwin
```

### Building natively

The CI workflow builds natively on all three systems with a plain `cargo build`. None of these have been tried by hand yet.

| System | Requirements |
|--------|--------------|
| Windows | Rust with the MSVC toolchain (Visual Studio Build Tools) |
| Linux | Rust and the native libraries GPUI links against. On Debian or Ubuntu: `sudo apt install libfontconfig-dev libfreetype-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libxcb1-dev libx11-xcb-dev`. At run time GPUI renders through `wgpu`, which needs a working GPU driver |
| macOS | Rust and the Xcode command line tools. Further requirements, if any, are not verified yet |

---

## Quick Start

### 1. Build

```bash
cargo xwin build --target x86_64-pc-windows-msvc
```

The executable is written to `target/x86_64-pc-windows-msvc/debug/rally.exe`. Always pass the Windows target through `cargo xwin`: a plain `cargo build` produces a Linux binary, which is the wrong target.

### Building natively (Linux, macOS, Windows)

```bash
cargo run
```

`cargo run` builds for the machine you are on and starts the app. Use `cargo build` to only build, which writes `target/debug/rally` (`rally.exe` on Windows).

### 2. Run the cross-compiled Windows build

From a WSL shell, inside the repository:

```bash
./target/x86_64-pc-windows-msvc/debug/rally.exe
```

See [Known limitations](#known-limitations) for why it has to be started from a WSL directory.

### 3. Try it

Enter `https://httpbin.org/get` in the URL field and press **Send** (or Enter). The response appears on the right, the request is listed in the sidebar, and **Console** in the status bar shows the raw exchange.

---

## Configuration

Rally has no configuration file. History is saved in the user's data directory, following each system's convention:

| System | History file | Read from |
|--------|--------------|-----------|
| Windows | `%APPDATA%\Rally\history.json` | `APPDATA` |
| macOS | `~/Library/Application Support/Rally/history.json` | `HOME` |
| Linux | `$XDG_DATA_HOME/rally/history.json`, or `~/.local/share/rally/history.json` | `XDG_DATA_HOME`, `HOME` |

Empty variables count as unset, and a relative `XDG_DATA_HOME` is ignored, as the XDG specification says. If no location can be found, Rally still runs but does not save history.

History is written atomically (a temporary file, then a rename). A history file that cannot be parsed is moved aside to `history.json.bak` and Rally starts with an empty history.

---

## Project Structure

```
rally/
├── Cargo.toml                  # workspace: shared versions, features and dependencies
├── .github/workflows/ci.yml    # fmt, clippy and build on Windows, Linux and macOS
├── crates/
│   ├── rally-core/             # no GPUI: requests, auth, history, console
│   │   └── src/
│   ├── rally-ui/               # the GPUI app
│   │   ├── assets/theme/       # Rally light and dark themes (JSON)
│   │   └── src/
│   │       ├── client/         # root view, tabs, chrome, request model
│   │       ├── widgets/        # reusable editors
│   │       ├── history/        # sidebar
│   │       └── console/        # bottom panel
│   └── rally/                  # the binary
│       ├── assets/icon/        # rally.svg and the generated rally.ico
│       ├── rally.rc            # resource script (icon, ID 1)
│       └── build.rs            # embeds the icon with llvm-rc
└── .vscode/settings.json       # rust-analyzer checks through cargo-xwin
```

---

## Build and Check Commands

| Task | Command |
|------|---------|
| Build | `cargo xwin build --target x86_64-pc-windows-msvc` |
| Type check | `cargo xwin check --target x86_64-pc-windows-msvc` |
| Lint | `cargo xwin clippy --workspace --target x86_64-pc-windows-msvc -- -D warnings` |
| Format check | `cargo fmt --all --check` |
| Native build, lint (Linux, macOS, Windows) | `cargo build --workspace`, `cargo clippy --workspace -- -D warnings` |

`cargo fmt` does not depend on the target. There are no automated tests yet. Because `rally-core` has no GPUI dependency, it can be tested on the host without cross-compiling.

---

## Known limitations

- **Linux and macOS have not been run by hand yet.** They are built in CI only, and the CI workflow itself has not been run yet.
- **No release build when cross-compiling from WSL.** gpui's build script compiles its shaders only when the build machine is Windows, so a cross-compiled release build is missing them. A native Windows build does not have this limit, but it has not been tried.
- **The cross-compiled debug executable must be started from a WSL directory.** In a debug build gpui compiles its shaders at runtime from a source path recorded at build time, which is a WSL path. Started from a normal Windows folder (for example by double-clicking), it stops at startup with `The system cannot find the path specified`.
- **History stores secrets in plain text.** Bearer tokens, Basic Auth passwords and API keys are saved unencrypted in `history.json`. See [SECURITY.md](SECURITY.md).
- **Copy cURL uses POSIX shell quoting** (bash, zsh). The command will not paste correctly into `cmd.exe` or PowerShell.
- **Not supported yet:** `HEAD` and `OPTIONS`, multipart and binary bodies, environment variables, collections, cancelling a request in flight, and saving tabs between runs.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Notable changes are listed in [CHANGELOG.md](CHANGELOG.md).

## License

Rally is released under the [MIT License](LICENSE).
