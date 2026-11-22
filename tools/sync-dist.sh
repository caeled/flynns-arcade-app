#!/usr/bin/env bash
# Assemble dist/ for the Flynn's Arcade desktop app:
# the collection's flynns-arcade project + the app-only interface pages.
set -euo pipefail
APP="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$HOME/workspace/everyday-cool/projects/flynns-arcade"
rm -rf "$APP/dist"
mkdir -p "$APP/dist"
cp -r "$SRC"/. "$APP/dist"/
# The collection paperwork is not part of the app bundle.
rm -f "$APP/dist/README.md" "$APP/dist/COLLECTION.md" "$APP/dist/THIRD-PARTY.md" \
      "$APP/dist/LICENSE" "$APP/dist/LICENSE-CONTENT.md"
cp "$APP/interface/splash.html" "$APP/dist/splash.html"
cp "$APP/interface/about.html" "$APP/dist/about.html"
node "$APP/tools/build-documents.js"
echo "dist assembled: $(find "$APP/dist" -type f | wc -l) files"
