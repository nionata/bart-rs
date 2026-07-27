#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "Usage: $0 <host> <public-url> [dest]" >&2
  exit 1
fi

HOST="$1"
PUBLIC_URL="$2"
DEST="${3:-/var/www/bart}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Building..."
rm -rf "$SCRIPT_DIR/dist" "$SCRIPT_DIR/.trunk"
nix develop "$WORKSPACE_ROOT" --builders "" --command bash -c "cd '$SCRIPT_DIR' && trunk build --release --public-url '$PUBLIC_URL'"

echo "Deploying to $HOST:$DEST..."
scp -r "$SCRIPT_DIR/dist/." "$HOST":"$DEST/"

echo "Done."
