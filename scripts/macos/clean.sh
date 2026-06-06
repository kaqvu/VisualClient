#!/bin/bash
cd "$(dirname "$0")/../.."

echo "Cleaning..."
rm -rf node_modules
rm -rf dist
rm -rf src-tauri/target
rm -rf src-tauri/gen
rm -f package-lock.json
rm -f pnpm-lock.yaml
rm -f yarn.lock
rm -f src-tauri/Cargo.lock
echo "Done."
