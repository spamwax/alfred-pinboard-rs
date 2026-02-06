#!/usr/bin/env bash
set -e

BIN="${BINARY_PATH:-}"

if [ -n "$BIN" ] && [ -x "$BIN" ]; then
  exec "$BIN" "$@"
fi

CANDIDATES=(
  "./alfred-chrome-bookmarks"
  "$HOME/.local/bin/alfred-chrome-bookmarks"
  "$HOME/.cargo/bin/alfred-chrome-bookmarks"
  "/usr/local/bin/alfred-chrome-bookmarks"
  "/opt/homebrew/bin/alfred-chrome-bookmarks"
)

for candidate in "${CANDIDATES[@]}"; do
  if [ -x "$candidate" ]; then
    exec "$candidate" "$@"
  fi
done

cat <<JSON
{"items":[{"title":"Binary not found","subtitle":"Set BINARY_PATH in workflow variables or place alfred-chrome-bookmarks in the workflow folder","valid":false}]}
JSON
