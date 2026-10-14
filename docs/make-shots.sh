#!/bin/zsh
# Erzeugt die README-Bilder aus docs/shots.html mit Chrome im Hintergrund (2-fache Auflösung).
cd "${0:A:h}"
CHROME="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
for shot in banner guard help missions; do
  "$CHROME" --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=2 \
    --window-size=1280,720 --virtual-time-budget=1500 \
    --screenshot="$PWD/$shot.png" "file://$PWD/shots.html#$shot" 2>/dev/null
  # PNG verkleinern/komprimieren (sips ist auf jedem Mac dabei)
  sips -Z 1600 "$shot.png" >/dev/null
  echo "✓ $shot.png"
done
