# Changelog

All notable changes to Rally are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Rally has not
had a release yet, so everything so far is listed under Unreleased.

## [Unreleased]

### Added

- HTTP client: send `GET`, `POST`, `PUT`, `PATCH` and `DELETE` requests from a merged method and
  URL field, on a background thread with a 30 second timeout.
- Response viewer: a read-only code editor with line numbers, folding, wrapping and JSON syntax
  highlighting; JSON bodies are pretty-printed; headers tab; status, duration and size; a button
  that copies the body or the headers.
- Query params and headers as key/value tables with enable checkboxes and descriptions, with tab
  counts.
- Header name autocomplete (25 common names) and header value autocomplete (14 headers).
- Authorization tab: Bearer Token, Basic Auth and API Key (header or query parameter).
- Body types: None, JSON, Text and Form (`application/x-www-form-urlencoded`), with a default
  `Content-Type` that a header you set yourself overrides.
- Request tabs: each keeps its own request, response and selected panes, and a request in flight
  finishes into the tab that sent it.
- Request history: the last 100 requests are saved to `%APPDATA%\Rally\history.json` and shown in
  a sidebar; clicking one restores the whole editor.
- Console panel: the raw request and response of each request sent this session (last 200), error
  details for failed requests, and Copy cURL.
- Rally light and dark themes with an indigo accent, dark by default, with a toggle in the status
  bar.
- Window layout: tab strip, status bar and a response format pill.
- App icon, embedded in the Windows executable.
- README, MIT license, contributing guide, security policy and `.gitattributes`.

### Changed

- Replaced the counter and todo demo with the API client.
- Renamed the project from `learn-gpui-windows` to `rally`.
- Split the single crate into a Cargo workspace: `rally-core` (no GPUI), `rally-ui` and `rally`.

### Known issues

- Release builds cannot be produced from WSL, and the debug executable only runs when started from
  a WSL directory. See the README for details.
- History saves Bearer tokens, Basic Auth passwords and API keys in plain text.
