#!/bin/bash
cd "$(dirname "$0")/../.."

VERSION=$1
if [ -z "$VERSION" ]; then
    read -p "Enter version to remove: " VERSION
fi

if [ -z "$VERSION" ]; then
    echo "Error: No version specified."
    exit 1
fi

if [[ "$VERSION" == v* ]]; then
    VERSION="${VERSION#v}"
fi

git tag -d v$VERSION
git push origin :refs/tags/v$VERSION

echo "Done."
