# LocalRemoteShare — Design (v1)

**Date:** 2026-06-14
**Status:** Approved (design phase)

## Overview

A cross-platform (macOS + Windows) menu-bar/tray utility built with **Tauri**. The
user drops in a single file, optionally sets a password, and gets a public HTTPS link
backed by a Cloudflare quick tunnel. Recipients open a minimal landing page (file name,
size, optional password field, download button). Multiple active shares can run at once,
and any share can be revoked individually. All state is ephemeral: quitting the app
clears every share and tears down the tunnel.

## Goals

- One-action sharing of a local file to a public, temporary HTTPS link.
- Minimal, native-feeling desktop UX: tray/menu-bar presence + a small drag-and-drop window.
- "Secure enough": HTTPS transport, unguessable link, optional per-share password.
- Multiple concurrent shares behind a single tunnel.
- Instant revoke; full teardown on quit.

## Non-Goals (v1)

- Multi-file / folder shares.
- Link expiry and download-count limits (designed to be added later).
- Persistence of shares across app restart/reboot.
- Alternative tunnel backends such as ngrok (interface left pluggable, not implemented).
- Code-signing / notarization workflow details.

## Tech Stack

- **Shell / app framework:** Tauri (Rust core + web frontend).
- **Local HTTP server:** `axum` (async, Tokio) bound to `127.0.0.1`.
- **Tunnel:** Cloudflare `cloudflared` quick tunnel (`cloudflared tunnel --url http://127.0.0.1:<port>`).
- **Password hashing:** `argon2` (in memory only).
- **Frontend:** HTML/CSS/JS (framework choice deferred to implementation plan; kept minimal).

## Architecture

Three logical layers inside the Tauri app.

### UI (web frontend)
The minimal window:
- Drag-and-drop zone + "select file" fallback.
- Active-shares list, each row showing: file name, size, link with copy button,
  password toggle, download count, revoke button.
- Status indicators (tunnel up/down, link down).
Communicates with the Rust core exclusively through Tauri commands.

### Rust core
Owns all state and logic.

- **Local file server** — embedded `axum` server on `127.0.0.1:<random free port>`.
  Serves the landing page and file downloads keyed by `/d/<token>`.
- **Share registry** — in-memory map:
  `token -> { file_path, name, size, password_hash?, download_count, created_at }`.
- **Tunnel manager** — spawns and supervises one `cloudflared` child process; parses its
  stdout to capture the `https://<random>.trycloudflare.com` base URL; restarts/reports on
  crash. Lazy lifecycle: starts on first share, stops when the last share is revoked.

### Tray / menu-bar integration
Tauri tray API: icon, show/hide window, active-share count, quit (which tears everything down).

## Data Flow

### Adding a share
1. User drops `file.pdf` -> frontend sends the path to the Rust `create_share` command
   (with optional password).
2. Core generates a long random `token` (~128 bits), hashes the password if present
   (argon2), inserts into the registry.
3. If the tunnel is not running, the tunnel manager starts `cloudflared` and waits for the
   base URL.
4. Core returns `https://<base>.trycloudflare.com/d/<token>` to the UI; UI displays it with
   a copy button.

### Download flow
1. Recipient opens the link -> landing page shows file name and size.
2. If a password is set, the page prompts and POSTs it; the core verifies against the stored
   hash.
3. On success, the file streams back through the local server; `download_count` increments.

## Security Model

- Transport: HTTPS terminated by Cloudflare.
- Secret = unguessable tunnel subdomain + ~128-bit random path token.
- Optional per-share password, argon2-hashed in memory, never written to disk.
- Local server binds to `127.0.0.1` only; the tunnel is the sole ingress.
- Revoke = remove token from registry -> link instantly returns 404.
- Quit = kill the tunnel process + drop all in-memory state.

## cloudflared Distribution

**Decision:** Bundle the `cloudflared` binary inside the app installer (works offline, no
surprise setup step). If the bundled binary is missing or fails to launch, fall back to a
guided download with an integrity check.

## Error Handling

- `cloudflared` missing / fails to start -> clear UI error; shares remain in the registry
  but no link is shown until the tunnel is up.
- Tunnel process crashes -> mark affected shares as "link down," attempt one automatic
  restart, surface status in the UI.
- File moved or deleted after sharing -> download returns 410 Gone; UI flags the share.
- Local port conflict -> automatically pick another free port.

## Testing

- **Rust unit tests:** token generation, registry add/revoke, password hash/verify,
  landing-page rendering, download streaming + count increment.
- **Tunnel manager:** tested against a fake/mock `cloudflared` (stdout fixture) so tests do
  not hit the network.
- **Frontend:** component test for the drop -> share-row lifecycle.
- **Manual E2E:** one documented checklist exercising a real `cloudflared` tunnel end to end.

## Future Extensions (post-v1)

- Link expiry and one-time / max-download limits (auto-revoke).
- Multi-file and folder shares (zip download).
- Pluggable tunnel backends (ngrok, self-hosted) behind the existing tunnel interface.
- Optional persistence of shares across restarts.
