#!/bin/zsh
# Baut Kompass, signiert mit Developer ID, lässt es von Apple notarisieren
# und erzeugt eine fertige DMG. Ergebnis: release/Kompass-<version>-arm64.dmg
#
# Voraussetzungen (einmalig):
#   - Zertifikat „Developer ID Application“ im Schlüsselbund
#   - xcrun notarytool store-credentials "kompass-notary" --apple-id … --team-id …
#
# Aufruf:  scripts/release-macos.sh
set -euo pipefail
cd "${0:A:h}/.."

IDENTITY="${APPLE_SIGNING_IDENTITY:-Developer ID Application: Andreas Klingenberger (585EA8WD3H)}"
PROFILE="${NOTARY_PROFILE:-kompass-notary}"
VERSION=$(python3 -c "import json; print(json.load(open('src-tauri/tauri.conf.json'))['version'])")
APP="src-tauri/target/release/bundle/macos/Kompass.app"
OUT="release"
DMG="$OUT/Kompass-$VERSION-arm64.dmg"

step() { print -P "\n%F{green}▶ $1%f"; }

step "Baue Kompass $VERSION (signiert mit: $IDENTITY)"
APPLE_SIGNING_IDENTITY="$IDENTITY" npx tauri build --bundles app

step "Prüfe Signatur und Hardened Runtime"
codesign --verify --deep --strict --verbose=2 "$APP"
codesign -dv "$APP" 2>&1 | grep -E "Authority=Developer ID|flags=.*runtime" || { echo "Signatur unvollständig"; exit 1; }

step "Notarisiere die App"
rm -rf "$OUT" && mkdir -p "$OUT"
ditto -c -k --keepParent "$APP" "$OUT/Kompass.zip"
xcrun notarytool submit "$OUT/Kompass.zip" --keychain-profile "$PROFILE" --wait
xcrun stapler staple "$APP"
rm "$OUT/Kompass.zip"

step "Erzeuge DMG"
STAGE=$(mktemp -d)
ditto "$APP" "$STAGE/Kompass.app"
ln -s /Applications "$STAGE/Applications"
hdiutil create -volname "Kompass" -srcfolder "$STAGE" -ov -format UDZO "$DMG" >/dev/null
rm -rf "$STAGE"
codesign --sign "$IDENTITY" --timestamp "$DMG"

step "Notarisiere die DMG"
xcrun notarytool submit "$DMG" --keychain-profile "$PROFILE" --wait
xcrun stapler staple "$DMG"

step "Gatekeeper-Prüfung"
spctl --assess --type open --context context:primary-signature --verbose=2 "$DMG"
spctl --assess --type execute --verbose=2 "$APP"

cp "$DMG" "$OUT/Kompass-macOS-arm64.dmg"
step "Fertig"
ls -lh "$OUT"
shasum -a 256 "$DMG"
