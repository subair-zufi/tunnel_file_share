# LocalRemoteShare — Manual E2E Checklist

These steps exercise the full app against a real Cloudflare tunnel. They require
a GUI and network access, so they are run manually (the automated suite uses a
mock cloudflared and never hits the network).

## Prerequisites

- `cloudflared` available — either the bundled sidecar (see
  [SIDECAR.md](SIDECAR.md)) or installed on PATH (`brew install cloudflared`).
- Rust toolchain + Tauri CLI (`. "$HOME/.cargo/env"` then `cargo tauri --version`).

## Launch

From the repo root:

```bash
. "$HOME/.cargo/env" && cd src-tauri && cargo tauri dev
```

Expect: the app window opens and a tray/menu-bar icon appears.

## Checklist

1. **Window + tray present.** The main window shows the drop zone; the tray icon
   is visible.
2. **Share a file (no password).** Drag a small file (e.g. a PDF) onto the drop
   zone — or click "choose a file".
   - A share row appears immediately.
   - Within ~10s a `https://*.trycloudflare.com/d/<token>` link is shown (status
     clears from "Starting tunnel…").
3. **Recipient download.** Click "Copy link" and open it in a browser (ideally
   another device/network).
   - Landing page shows the file name and human-readable size.
   - Click **Download** → the file downloads intact (compare bytes/hash).
   - The share row's download count increments to 1.
4. **Password-protected share.** Add a second file with "Password protect" checked
   and a password set.
   - Opening its link shows a password field.
   - Submitting a **wrong** password → "Incorrect password." (HTTP 401).
   - Submitting the **correct** password → the file downloads.
5. **Malicious file name is safe.** Share a file named e.g. `a<b>&"c.txt`.
   - The landing page renders the name literally (escaped); no broken markup, no
     script execution. The download still works.
6. **Revoke.** Click **Revoke** on a share.
   - Its link now returns 404 ("Link not found or revoked.").
7. **Tunnel lifecycle.** Revoke all shares.
   - The `cloudflared` process exits (the tunnel is torn down when the last share
     is revoked). Verify with `pgrep cloudflared` (no output).
8. **Multiple concurrent shares.** Add three files; confirm all three serve from
   the single tunnel at distinct `/d/<token>` paths simultaneously.
9. **File removed after sharing.** Share a file, then move/delete it on disk, then
   open its link and download → reports the file is gone (HTTP 410).
10. **Close vs quit.** Closing the window hides it (app keeps running, tray
    remains). Choosing **Quit** from the tray exits the app; confirm no
    `cloudflared` process remains (`pgrep cloudflared` is empty) and active shares
    are gone (ephemeral state).

## Notes

- Quick tunnels get a new random URL on each run, so links do not survive an app
  restart — shares are intentionally ephemeral.
- If a link never appears, check that `cloudflared` is resolvable (PATH or
  bundled sidecar) and that the machine has outbound network access.
