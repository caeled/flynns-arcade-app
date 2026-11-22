# Flynn's Arcade — desktop app

The local-install edition of **Flynn's Arcade**: twenty cabinets built in one
night (21 November 2026) for Flynn and David Powell — arcade classics, BBS door
games reskinned for the AI age, a card table with honest odds, a bulletin
board, and a wandering hub. Every cabinet carries its story, its lesson, and a
Play button.

This is a Tauri v2 shell over the collection's `projects/flynns-arcade/`
(the same files the web PWA serves). `tools/sync-dist.sh` assembles `dist/`
from the collection project plus the app-only pages in `interface/`
(splash, About), and generates `documents.html` — the Cabinet Documents page
behind Help → Cabinet Documents — from the hub's own cabinet data.

- File → Clear Cache: the standing cure for a stale webview profile
  (flag-file + restart; saved progress is never touched).
- Help → Cabinet Documents / GitHub Repository / About.
- Tagging `v*` builds Windows and macOS bundles in GitHub Actions and
  attaches them to the release.

The web/PWA edition lives in the Everyday Cool collection:
https://caeled.github.io/everyday-cool/projects/flynns-arcade/
