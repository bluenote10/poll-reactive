#!/bin/bash

set -e
set -o pipefail

# https://stackoverflow.com/a/75023425/1804173

tag=$(cargo metadata --format-version=1 --no-deps | jq --raw-output --exit-status '.packages[0].version')
echo "Adding tag: $tag"

git tag "$tag"
git push origin "$tag"
