#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <host>" >&2
  exit 1
fi

HOST="$1"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Building..."
rm -rf "$SCRIPT_DIR/dist" "$SCRIPT_DIR/.trunk"
nix develop "$WORKSPACE_ROOT" --builders "" --command bash -c "cd '$SCRIPT_DIR' && trunk build --release"

echo "Deploying to $HOST..."
scp -r "$SCRIPT_DIR/dist/." "$HOST":/var/www/bart-tender/

echo "Done."
