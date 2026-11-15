# Security Policy

## Supported versions

Rally has not had a release yet. Only the latest commit on `main` is supported.

## Reporting a vulnerability

Please do not report security problems in a public issue or pull request.

Rally does not have a public repository or a private reporting address yet. Until one is
published here, contact the maintainer directly and share details only in private.

When you report a problem, please include what you found, the steps to reproduce it, the version or
commit you tested, and what an attacker could do with it.

## What to know before using Rally

- **History is stored in plain text.** Rally saves each request you send, including Bearer tokens,
  Basic Auth passwords and API keys, unencrypted, in `history.json` in your user data directory
  (`%APPDATA%\Rally` on Windows, `~/Library/Application Support/Rally` on macOS, `$XDG_DATA_HOME/rally`
  or `~/.local/share/rally` on Linux). Anyone who can read your user profile can read them. Do not use real production credentials on a shared
  computer, and delete the file or use **Clear** in the sidebar when you are done.
- **The console is in memory only.** Its log shows request and response headers in clear text,
  including `Authorization`, so that you can check what was sent. It is never written to disk and
  is gone when you close Rally.
- **TLS certificates are verified.** HTTPS requests use `rustls` with the bundled web PKI roots,
  and Rally has no option to skip certificate verification.
- **Network traffic.** Rally itself makes no network requests other than the ones you send.
