# Contributing to Rally

Thanks for your interest in Rally. This guide covers how to set up, what to run before you commit,
and where code belongs.

## Getting set up

Rally is built on Linux or WSL and cross-compiled to Windows. Follow the
[Prerequisites](README.md#prerequisites) and [Quick Start](README.md#quick-start) in the README.
Every cargo command that builds or lints the app goes through `cargo xwin` with
`--target x86_64-pc-windows-msvc`. A plain `cargo build` produces a Linux binary, which is the
wrong target.

## Before you commit

Run all of these. Each commit should build and pass them.

```bash
cargo fmt --all --check
cargo xwin clippy --workspace --target x86_64-pc-windows-msvc -- -D warnings
cargo xwin build --target x86_64-pc-windows-msvc
```

There are no automated tests yet. Start the app and try the change by hand: GPUI views are not
covered by anything else. Anything in `rally-core` has no GPUI dependency and can be tested on the
host with plain `cargo test -p rally-core`.

## Where code goes

| Crate | Put here | Not here |
|-------|----------|----------|
| `rally-core` | Anything that does not need a window: request building, auth, URL and body logic, formatting, history and console data, the HTTP transport | GPUI types, colors, layout |
| `rally-ui` | Views, editors, theme, models that hold GPUI state | Logic that can live in `rally-core` |
| `rally` | The binary entry point and the icon | Everything else |

- Keep logic in `rally-core` and keep the UI thin. For example the UI collects a `RequestDraft`, and
  `RequestDraft::build` decides what is actually sent.
- `rally-ui` exports only `run()`. Keep everything else private to the crate.
- Follow the existing split in `rally-ui`: a model (state and writes) next to a view or panel
  (rendering).

## Commits

- Use [Conventional Commits](https://www.conventionalcommits.org/) prefixes without a scope:
  `feat:`, `fix:`, `chore:`, `docs:`, `refactor:` or `test:`. For example `feat: add request tabs`.
- Keep the subject under 72 characters and in the imperative mood.
- One logical change per commit. Keep refactors, formatting and behavior changes in separate
  commits.
- Never commit secrets, `.env` files or build output.

## Line endings

`.gitattributes` stores text as LF. If git warns about line endings, make sure the file is present
and your editor is not forcing CRLF.

## Reporting security problems

Please do not report vulnerabilities in a public issue. See [SECURITY.md](SECURITY.md).

## License

By contributing you agree that your contribution is licensed under the [MIT License](LICENSE).
