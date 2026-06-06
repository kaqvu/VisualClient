#!/bin/bash
cd "$(dirname "$0")/../.."

v1=$(grep '"version"' package.json | head -1 | awk -F '"' '{print $4}')
v2=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | awk -F '"' '{print $4}')
v3=$(grep '^version' src-tauri/Cargo.toml | head -1 | awk -F '"' '{print $2}')

if [ "$v1" == "$v2" ] && [ "$v2" == "$v3" ]; then
  new_version=$v1
else
  echo "Versions mismatch. Please use version-changer first."
  exit 1
fi

git add .
git commit -m "Release $new_version"
git push origin main
git tag v$new_version
git push origin v$new_version

echo "Done."
