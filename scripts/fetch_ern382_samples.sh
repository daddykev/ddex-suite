#!/bin/sh
# Download YouTube's published ERN 3.8.2 sample messages ("DDEX 3.8 reference resources",
# https://support.google.com/youtube/answer/3506749) into samples/youtube-ern382/ at the
# repository root and verify pinned SHA-256 hashes. Used by the builder's 3.8.2 round-trip
# test (packages/ddex-builder/tests/it/ern38_round_trip.rs).
#
# The files are Google's; no licence for redistribution is stated, so they are fetched here
# and gitignored instead of vendored. Only the 3.8.2 files are fetched: the page's
# "CreatorMusic" samples are ERN 3.8.3, which the builder does not target.
#
# Usage: scripts/fetch_ern382_samples.sh [--force]
set -eu

BASE="https://storage.googleapis.com/support-kms-prod"
ROOT=$(cd "$(dirname "$0")/.." && pwd)
DEST="$ROOT/samples/youtube-ern382"
FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1

sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d' ' -f1
    else
        shasum -a 256 "$1" | cut -d' ' -f1
    fi
}

mkdir -p "$DEST"
# name, storage id, sha256
while read -r name id expected; do
    [ -z "$name" ] && continue
    out="$DEST/$name"
    if [ "$FORCE" -eq 0 ] && [ -f "$out" ] && [ "$(sha256 "$out")" = "$expected" ]; then
        continue
    fi
    tmp="$out.download"
    if ! curl -sSfL --retry 2 -o "$tmp" "$BASE/$id"; then
        rm -f "$tmp"
        echo "error: could not download $name ($BASE/$id)" >&2
        exit 1
    fi
    actual=$(sha256 "$tmp")
    if [ "$actual" != "$expected" ]; then
        rm -f "$tmp"
        echo "error: SHA-256 mismatch for $name: expected $expected, got $actual." >&2
        echo "YouTube may have replaced the sample; inspect it and re-pin it here." >&2
        exit 1
    fi
    mv "$tmp" "$out"
done <<'LIST'
721620118165_yt_premium.xml c8yx7DxxAdSMqmMJNz7ZQEqlg7ARPKNS8m0U 4ed5448a9011d22db491a9d8e893ffa6e7bd99e0e96f72410722de75ee83c8fb
721620118165_yt_premium_takedown.xml TUIOKpNkU30LulhHhbvdfl9IueEoWhs3m8C8 1967bbf494b8029c2449fe3e5f0ce4090aee409b0decc2e6a005b71adc6cc761
721620118165_content_id.xml AekQu4qpMlsxNNx9roDnBrGAemqr5QVDt0sa 5f365a67e9eb61e2a6534be0832afd037316a6fd725fd5e2daf7c3f8f05ac36f
721620118165_content_id_takedown.xml PERr54Ny0muTwdHTwHGdvZ5VtlIjvV1BcI4r eb85c829f10756b743428e71609cfecb55312a8ad69d8e0542ef7f57cafa00be
721620118165_combined.xml dh7pAMMWVyuE4CXybzomQcXFYPAMbHUm3Egx 2ecfa68d04d9a19f935944e1adbc77cd6b7b827c0e7df377c69cfc15f4fe1867
721620118165_combined_takedown.xml Ie1JsHgxLkQLTsW18mXDXZRQLSBnVn0z83AS dd7ca2dab9719840da5d49159e9d93a704f6333eb68c0127bb825c026136602b
USGCV1172318_content_id.xml ZPiAtCaZ6tQgYfDZKH2PyUt5tFEZsK605G9K 2684696e9d5f5cbc623ce5fecb77660c5d50c2847e8d62a92719e68f6a57b01e
USGCV1172318_content_id_takedown.xml e0zg0czwmyF3ZouZMDf7tgWNUjO5IqmzSMhM ba3ffff092509d1aee7892d56f7acc366df6baf79081c4792cb717094cd895fe
music_video_only.xml huCNBLY2xLkFH3CHz3vPSMlV5XIS5W06IrdL 6a3e97bb4414870d05eb9dceee8b58eab7fd9ad2c123ed027d67ecc3a4426f92
music_video_only_update.xml bi6KR3je1oLmS2I9qGEIo37cgyBVu9NFzZzR 5a68a5214f34d3fc2074f742a19c6265ed689e5527cc784ad6d98133f3658cd7
QZ6RS1700001_music_video_content_id_only.xml 4E65bc5wB1Epq23j2289HbGp8r1bVHfOZUHs 2f6723e33f27c4da8c8ce98049e9556f3ab0e58f337ecd4974a62db5dac82260
QZ6RS1700001_music_video_content_id_only_update.xml ptwGdFQDIGA6vISVgygkZO3XSXDaSCsRJfEZ 1fbee5091b23eb4e70e778eba38b6d02c5d0731a65b7ccc2c43132ca8396702e
QZ6RS1700001_music_video_content_id_combined.xml Vzw7t3A1L0iNW3JESkCO54zvFHm3LSliaPhf a831206e7023831dcb4406eaef1a0ea3f07265514f7afee34d954f9881962b81
QZ6RS1700001_music_video_content_id_combined_update.xml NM4Gpb4BnJ2x5xGUTaswPLuFdpSmWPAZGyBI 6a8be2dae513ad64ccdbb02b6b6e53f9768cb2c37ec6472ad3bb49e35884bd3d
web_video_only.xml sN2L76hYgXBJVq1NaL9YOKzk6wFPs49fPXNg bce5ec976fc16f8a1b2b293ce36cf5bad265aa0c7dfe93ac84f0e431cc143c2e
web_video_only_update.xml glx18lXjFAiYvTzmqemSm9l4r8Allm1c6UGr 8b17837c9b86d69c6131ef2a986f57fe2aad95b80b9964d35c17a77c09a90451
web_video_content_id_only.xml UHzyX2MDsNtfsGPllyTuq4CeoL9chT2RSLmM 659f16657427784671d1a753354824dbe35e144fc8373e4a0680fa4fcf810836
web_video_content_id_only_update.xml 1R8GD5tIfrmvKhUpQ53O8CJacKbfwFkut7Cj e085dc52ecf7cf5449dd9886f82d1261a041edfb0ba9850e9a01b1812bed22ec
web_video_content_id_combined.xml RvJ8Ml9pEOIT89OcrCUMuYQuuC7Emh78yF9Z b9d120e38ea802b14980498854ea871c7168e6cc5332f1032ea8e3091c5b011f
web_video_content_id_combined_update.xml SrbMNCbcpQZxcuDOX4xrFuLbEbP8x7dOEN0R ccab8851152b13caaa10762aabaafe6cf5d7adab9d666cd5aeee66cdc953bdfe
LIST

echo "ERN 3.8.2 samples ready in $DEST"
