#!/usr/bin/env bash
# Construit une release de L'Atlas des harmoniques :
#   vérifications, archives cargo-dist, arbre installé, paquet Arch, paquet rpm.
#
# Prérequis : cargo, cargo-dist (cargo install cargo-dist), et selon la distribution
# makepkg (Arch) ou rpmbuild (rpm). macOS et Windows demandent une CI sur place :
# voir dist-workspace.toml.
#
#   tools/release.sh              # construit dans dist/
#   tools/release.sh --quick      # sans tests, clippy ni parité
#   tools/release.sh --publish    # publie sur Forgejo (FORGEJO_URL, FORGEJO_TOKEN, FORGEJO_REPO)
set -euo pipefail
cd "$(dirname "$0")/.."

APP_ID="com.lecompositeur.adh"
TARGET="x86_64-unknown-linux-gnu"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
DISTRIB="target/distrib"
OUT="dist"
STAGE="$OUT/stage"

quick=0
publish=0
for arg in "$@"; do
    case "$arg" in
        --quick) quick=1 ;;
        --publish) publish=1 ;;
        *) echo "argument inconnu : $arg" >&2; exit 2 ;;
    esac
done

if ! command -v dist > /dev/null; then
    echo "cargo-dist introuvable : cargo install cargo-dist" >&2
    exit 1
fi

echo "=== L'Atlas des harmoniques $VERSION ==="

if [ "$quick" -eq 0 ]; then
    echo '[1/6] vérifications'
    cargo test --workspace
    cargo clippy --workspace --all-targets
    ./tools/parity.sh
else
    echo '[1/6] vérifications ignorées (--quick)'
fi

echo '[2/6] archives cargo-dist'
dist build
mkdir -p "$OUT"
rm -f "$OUT"/*.tar.xz "$OUT"/*.sha256 "$OUT"/sha256.sum
cp "$DISTRIB"/*.tar.xz "$DISTRIB"/*.tar.xz.sha256 "$OUT"/
cp "$DISTRIB/sha256.sum" "$OUT"/

echo '[3/6] arbre installé'
rm -rf "$STAGE"
mkdir -p "$STAGE/usr/bin" \
         "$STAGE/usr/share/applications" \
         "$STAGE/usr/share/metainfo" \
         "$STAGE/usr/share/icons/hicolor/256x256/apps" \
         "$STAGE/usr/share/icons/hicolor/scalable/apps" \
         "$STAGE/usr/share/doc/adh"
cp "$DISTRIB/adh-app-$TARGET/adh-app" "$STAGE/usr/bin/adh-app"
cp "$DISTRIB/adh-core-$TARGET/adh-dump" "$STAGE/usr/bin/adh-dump"
cp "packaging/$APP_ID.desktop" "$STAGE/usr/share/applications/$APP_ID.desktop"
cp "packaging/$APP_ID.appdata.xml" "$STAGE/usr/share/metainfo/$APP_ID.appdata.xml"
cp crates/adh-assets/assets/logo.png "$STAGE/usr/share/icons/hicolor/256x256/apps/$APP_ID.png"
cp crates/adh-assets/assets/logo.svg "$STAGE/usr/share/icons/hicolor/scalable/apps/$APP_ID.svg"
cp LICENSE-MIT LICENSE-APACHE LICENSE-OFL LICENSE-assets.md README.md "$STAGE/usr/share/doc/adh/"

echo '[4/6] archive portable'
tar -cf - -C "$STAGE" usr | gzip -9 > "$OUT/adh-$VERSION-$TARGET.tar.gz"

echo '[5/6] paquet Arch'
if command -v makepkg > /dev/null; then
    builddir="$(mktemp -d)"
    cp packaging/PKGBUILD "$builddir/"
    cp "$OUT/adh-$VERSION-$TARGET.tar.gz" "$builddir/adh.tar.gz"
    (cd "$builddir" && ADH_VERSION="$VERSION" makepkg -f --noconfirm --skippgpcheck)
    mv "$builddir"/adh-*.pkg.tar.zst "$OUT"/
    rm -rf "$builddir"
else
    echo '  makepkg absent : paquet Arch non construit'
fi

echo '[6/6] paquet rpm'
if command -v rpmbuild > /dev/null; then
    rm -rf "$OUT/rpm"
    rpmbuild --define "_topdir $PWD/$OUT/rpm" \
             --define "adh_version $VERSION" \
             --define "adh_stage $PWD/$STAGE" \
             -bb packaging/adh.spec
    find "$OUT/rpm/RPMS" -name '*.rpm' -exec mv {} "$OUT"/ \;
    rm -rf "$OUT/rpm"
else
    echo '  rpmbuild absent : paquet rpm non construit'
fi

echo '=== empreintes ==='
(
    cd "$OUT"
    : > "adh-$VERSION.sha256"
    for artifact in adh-*.tar.gz adh-*.tar.xz adh-*.pkg.tar.zst adh-*.rpm; do
        [ -e "$artifact" ] || continue
        sha256sum "$artifact" >> "adh-$VERSION.sha256"
    done
    cat "adh-$VERSION.sha256"
)

if [ "$publish" -eq 1 ]; then
    : "${FORGEJO_URL:?FORGEJO_URL, ex. http://192.168.71.118:3000}"
    : "${FORGEJO_TOKEN:?FORGEJO_TOKEN}"
    : "${FORGEJO_REPO:?FORGEJO_REPO, ex. tristan/adh}"
    echo '=== publication Forgejo ==='
    release=$(curl -fsS -X POST \
        -H "Authorization: token $FORGEJO_TOKEN" \
        -H 'Content-Type: application/json' \
        -d "{\"tag_name\":\"$VERSION\",\"name\":\"L'Atlas des harmoniques $VERSION\",\"body\":\"Archives et paquets construits par tools/release.sh.\",\"draft\":false,\"prerelease\":false}" \
        "$FORGEJO_URL/api/v1/repos/$FORGEJO_REPO/releases")
    id=$(printf '%s' "$release" | sed -n 's/.*"id":\([0-9]*\).*/\1/p')
    for artifact in "$OUT"/adh-*; do
        [ -f "$artifact" ] || continue
        echo "  $(basename "$artifact")"
        curl -fsS -X POST -H "Authorization: token $FORGEJO_TOKEN" \
            -F "file=@$artifact" \
            "$FORGEJO_URL/api/v1/repos/$FORGEJO_REPO/releases/$id/assets?name=$(basename "$artifact")" > /dev/null
    done
fi

echo "=== terminé : voir $OUT/ ==="
