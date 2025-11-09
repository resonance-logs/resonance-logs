#!/bin/sh
set -eu

# Usage: sh scripts/generate_updater_json.sh [target_dir]
# Generates a minimal Tauri-compatible updater.json in the target directory
# Expects to be run from the repository root. Safe to call even if no .exe files

TARGET_DIR=${1:-deploy/downloads}

if [ ! -d "$TARGET_DIR" ]; then
  echo "Directory $TARGET_DIR does not exist; skipping updater.json generation"
  exit 0
fi

echo "Generating $TARGET_DIR/updater.json"
cd "$TARGET_DIR"

# Find the newest .exe (by mtime)
latest=$(ls -1t -- *.exe 2>/dev/null | head -n1 || true)
if [ -z "$latest" ]; then
  echo "No .exe files found in $TARGET_DIR; skipping updater.json"
  exit 0
fi

# Compute sha256 (prefer sha256sum, fallback to shasum -a 256)
if command -v sha256sum >/dev/null 2>&1; then
  sha=$(sha256sum "$latest" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
  sha=$(shasum -a 256 "$latest" | awk '{print $1}')
else
  echo "No sha256 tool available; skipping updater.json"
  exit 0
fi

pub_date=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
if [ -n "${CI_COMMIT_TAG:-}" ]; then
  version="$CI_COMMIT_TAG"
else
  version="$latest"
fi

cat > updater.json.tmp <<EOF
{
  "version": "${version}",
  "notes": "",
  "pub_date": "${pub_date}",
  "platforms": {
    "windows": [
      {
        "url": "/downloads/${latest}",
        "sha256": "${sha}"
      }
    ]
  }
}
EOF

mv updater.json.tmp updater.json
echo "Wrote updater.json -> $TARGET_DIR/updater.json"

cd - >/dev/null || true
