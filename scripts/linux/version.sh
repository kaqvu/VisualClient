#!/bin/bash
cd "$(dirname "$0")/../.."

echo "Current versions:"
echo "package.json:    $(grep '"version"' package.json | head -1 | awk -F '"' '{print $4}')"
echo "tauri.conf.json: $(grep '"version"' src-tauri/tauri.conf.json | head -1 | awk -F '"' '{print $4}')"
echo "Cargo.toml:      $(grep '^version' src-tauri/Cargo.toml | head -1 | awk -F '"' '{print $2}')"
echo ""

read -p "Enter new version: " new_version

if [ -z "$new_version" ]; then
  echo "Version cannot be empty."
  exit 1
fi

sed -i "s/\"version\": \".*\"/\"version\": \"$new_version\"/" package.json
sed -i "s/\"version\": \".*\"/\"version\": \"$new_version\"/" src-tauri/tauri.conf.json
sed -i "s/^version = \".*\"/version = \"$new_version\"/" src-tauri/Cargo.toml

echo "Done."
