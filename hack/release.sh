#!/usr/bin/env bash
set -euo pipefail

version=${1}

sed -i "s/## Unreleased/## Unreleased\n\n## ${version}/" CHANGELOG.md
find -name Cargo.toml | xargs sed -i "s/version =.* # hack\/release.sh$/version = \"${version}\" # hack\/release.sh/"

cargo make dev-slow
