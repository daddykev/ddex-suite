#!/usr/bin/env bash
# Verify every file downloaded from a Release build run before publishing it.
#
# For each .crate, .tgz, .whl, sdist (.tar.gz), .node and SBOM (.cdx.json) under DIR:
# `gh attestation verify` must find a build-provenance attestation signed by this
# repository's release-build.yml. Each package (.crate, .tgz, .whl, .tar.gz) must also have
# a CycloneDX SBOM attestation. Any failure stops with a non-zero exit code.
#
# Usage: scripts/verify_release_artifacts.sh DIR
#   DIR: the output of `gh run download <run-id> --repo daddykev/ddex-suite --dir DIR`
# Needs gh 2.49 or later, logged in (`gh auth login`).

set -euo pipefail

dir="${1:?usage: $0 DIR (a downloaded Release build run)}"
repo="daddykev/ddex-suite"
workflow="$repo/.github/workflows/release-build.yml"

files=()
while IFS= read -r -d '' f; do
  files+=("$f")
done < <(find "$dir" -type f \( -name '*.crate' -o -name '*.tgz' -o -name '*.whl' \
  -o -name '*.tar.gz' -o -name '*.node' -o -name '*.cdx.json' \) -print0 | sort -z)

if [ "${#files[@]}" -eq 0 ]; then
  echo "no release artifacts under $dir" >&2
  exit 1
fi

for f in "${files[@]}"; do
  gh attestation verify "$f" --repo "$repo" --signer-workflow "$workflow" >/dev/null
  echo "provenance OK  $f"
  case "$f" in
    *.crate | *.tgz | *.whl | *.tar.gz)
      gh attestation verify "$f" --repo "$repo" --signer-workflow "$workflow" \
        --predicate-type https://cyclonedx.org/bom >/dev/null
      echo "SBOM OK        $f"
      ;;
  esac
done
echo "all ${#files[@]} files verified"
