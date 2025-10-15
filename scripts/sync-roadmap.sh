#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(git rev-parse --show-toplevel)"
ROOT_ROADMAP="$ROOT_DIR/docs/root-archive/roadmap.md"
MIRROR_ROADMAP="$ROOT_DIR/docs/supplemental-docs/roadmap.md"
NOTE="<!-- Mirror of the canonical roadmap at ../root-archive/roadmap.md. Edit the archive file and copy changes here. -->"
if [[ ! -f "$ROOT_ROADMAP" ]]; then
  echo "error: $ROOT_ROADMAP is missing" >&2
  exit 1
fi
mkdir -p "$(dirname "$MIRROR_ROADMAP")"
{
  printf '%s\n' "$NOTE"
  cat "$ROOT_ROADMAP"
} > "$MIRROR_ROADMAP"
