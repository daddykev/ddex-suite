#!/bin/sh
# Download the official DDEX ERN 3.8.2 / 4.2 / 4.3 XSDs (and the allowed-value-set schemas
# they import) into schemas/ at the repository root, verify pinned SHA-256 hashes, and point
# the imports at the local copies so `xmllint --schema` works offline.
#
# The XSDs are (c) DDEX and licensed under the DDEX Evaluation / Implementation Licences
# (https://kb.ddex.net/general-implementation-guidance/licensing-the-standards/), which do
# not grant redistribution, so they are fetched here and gitignored instead of vendored.
#
# Usage: scripts/fetch_ddex_schemas.sh [--force]
#   --force   download again even if the files are already present and verified
#
# ERN 3.8.2: DDEX no longer serves the ERN 3.x XSDs (http://service.ddex.net/xml/ern/ lists
# only 4.x since 2026-07). The official file is fetched from the Internet Archive's capture
# of its original URL, http://service.ddex.net/xml/ern/382/release-notification.xsd
# (2025-03-19), and must match the pinned hash of the DDEX original. A local copy of the
# official file can be used instead: DDEX_ERN382_XSD=/path/to/release-notification.xsd
# (it is checked against the same hash). ERN 3.8.1: no official copy is available, so it
# is not fetched (see KNOWN_ISSUES.md).
#
# Layout:
#   schemas/ern/382/release-notification.xsd  (imports ../../avs/avs_20161006.xsd)
#   schemas/ern/42/release-notification.xsd   (imports ../../avs/avs20200518.xsd)
#   schemas/ern/43/release-notification.xsd   (imports ../../avs/allowed-value-sets.xsd)
#   schemas/avs/avs_20161006.xsd
#   schemas/avs/avs20200518.xsd
#   schemas/avs/allowed-value-sets.xsd
#
# Re-pinning: the 4.3 schema imports allowed-value-sets.xsd from an UNVERSIONED URL, which
# DDEX updates when it publishes a new AVS version. If its hash no longer matches, check the
# new file (its AvsVersionId annotation), confirm the builder tests still pass against it,
# and update the matching *_SHA256 value below with the new hash this script prints.
set -eu

ERN382_ORIGINAL_URL="http://service.ddex.net/xml/ern/382/release-notification.xsd"
ERN382_URL="http://web.archive.org/web/20250319135233id_/$ERN382_ORIGINAL_URL"
ERN382_SHA256="366dfba9dc96cd5cc6567ad7fdf388303b624ff231fc28e67be2ca5751a60a14"
AVS2016_URL="http://ddex.net/xml/avs/avs_20161006.xsd"
AVS2016_SHA256="2c57cfed4e364355d49d2ea7eff414c2358c96217d53e6ba8af627e8253f38dc"
ERN42_URL="http://service.ddex.net/xml/ern/42/release-notification.xsd"
ERN42_SHA256="2882de727154bd66d9f3f6ce1e5da4be0958ac4a6291d8a88d4d69b3c8a1764b"
ERN43_URL="http://service.ddex.net/xml/ern/43/release-notification.xsd"
ERN43_SHA256="186d9ce6e9401c0bf47b63b7ae6dead01bf48324ba7308ec3074b32b7853f5e9"
AVS2020_URL="http://ddex.net/xml/avs/avs20200518.xsd"
AVS2020_SHA256="254c49a557b148a2c49ce423731316fedefb38cb6fa15df2543421dc278b945e"
# Unversioned: DDEX replaces this file with each AVS release (pinned: AvsVersionId 12).
AVS_URL="http://ddex.net/xml/allowed-value-sets/allowed-value-sets.xsd"
AVS_SHA256="03f290c5a6715e45dcdde5f4fa2b44821c3372331f5bec0b17c802d6fa0e8385"

ROOT=$(cd "$(dirname "$0")/.." && pwd)
DEST="$ROOT/schemas"
FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}

# fetch URL SHA256 DEST_FILE [LOCAL_FILE]: download (or copy LOCAL_FILE) to DEST_FILE.orig,
# verify, keep the original
fetch() {
    url=$1 expected=$2 out=$3 local_copy=${4:-}
    orig="$out.orig"
    if [ "$FORCE" -eq 0 ] && [ -f "$orig" ] && [ "$(sha256 "$orig")" = "$expected" ]; then
        return 0
    fi
    mkdir -p "$(dirname "$out")"
    tmp="$orig.download"
    if [ -n "$local_copy" ]; then
        url=$local_copy
        cp "$local_copy" "$tmp"
    elif ! curl -sSfL --retry 2 -o "$tmp" "$url"; then
        rm -f "$tmp"
        echo "error: could not download $url" >&2
        exit 1
    fi
    actual=$(sha256 "$tmp")
    if [ "$actual" != "$expected" ]; then
        rm -f "$tmp"
        cat >&2 <<MSG
error: SHA-256 mismatch for $url
  expected $expected
  got      $actual
DDEX may have published a new version of this file. This is expected from time to time for
the UNVERSIONED allowed-value-sets.xsd (a new AVS release). Inspect the new file, re-run the
builder's XSD tests against it, and if they pass, re-pin by setting the matching *_SHA256
value in scripts/fetch_ddex_schemas.sh to the hash above.
MSG
        exit 1
    fi
    mv "$tmp" "$orig"
}

fetch "$ERN382_URL" "$ERN382_SHA256" "$DEST/ern/382/release-notification.xsd" "${DDEX_ERN382_XSD:-}"
fetch "$AVS2016_URL" "$AVS2016_SHA256" "$DEST/avs/avs_20161006.xsd"
fetch "$ERN42_URL" "$ERN42_SHA256" "$DEST/ern/42/release-notification.xsd"
fetch "$ERN43_URL" "$ERN43_SHA256" "$DEST/ern/43/release-notification.xsd"
fetch "$AVS2020_URL" "$AVS2020_SHA256" "$DEST/avs/avs20200518.xsd"
fetch "$AVS_URL" "$AVS_SHA256" "$DEST/avs/allowed-value-sets.xsd"

# Working copies with the imports pointing at the local files (the .orig files stay as
# downloaded, so their hashes can be re-checked).
cp "$DEST/avs/avs_20161006.xsd.orig" "$DEST/avs/avs_20161006.xsd"
cp "$DEST/avs/avs20200518.xsd.orig" "$DEST/avs/avs20200518.xsd"
cp "$DEST/avs/allowed-value-sets.xsd.orig" "$DEST/avs/allowed-value-sets.xsd"
sed 's#schemaLocation="http://ddex.net/xml/avs/avs_20161006.xsd"#schemaLocation="../../avs/avs_20161006.xsd"#' \
    "$DEST/ern/382/release-notification.xsd.orig" > "$DEST/ern/382/release-notification.xsd"
sed 's#schemaLocation="http://ddex.net/xml/avs/avs20200518.xsd"#schemaLocation="../../avs/avs20200518.xsd"#' \
    "$DEST/ern/42/release-notification.xsd.orig" > "$DEST/ern/42/release-notification.xsd"
sed 's#schemaLocation="http://ddex.net/xml/allowed-value-sets/allowed-value-sets.xsd"#schemaLocation="../../avs/allowed-value-sets.xsd"#' \
    "$DEST/ern/43/release-notification.xsd.orig" > "$DEST/ern/43/release-notification.xsd"
for f in "$DEST/ern/382/release-notification.xsd" "$DEST/ern/42/release-notification.xsd" \
    "$DEST/ern/43/release-notification.xsd"; do
    if grep -q 'schemaLocation="http' "$f"; then
        echo "error: $f still imports a remote schema; the import rewrite needs updating" >&2
        exit 1
    fi
done

echo "DDEX schemas ready in $DEST"
