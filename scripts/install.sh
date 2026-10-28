#!/bin/sh
set -eu
cd "$(dirname "$0")"
base="${XDG_DATA_HOME:-$HOME/.local/share}"
app="$base/stamina/app"
mkdir -p "$app" "$base/applications" "$base/icons/hicolor/scalable/apps"
cp stamina "$app/stamina"
chmod +x "$app/stamina"
cp stamina.svg "$base/icons/hicolor/scalable/apps/stamina.svg"
case "$app" in *'"'*|*'`'*|*'$'*|*'\'*) echo 'Unsupported characters in installation path' >&2; exit 1;; esac
cat > "$base/applications/stamina.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Stamina
Comment=Touch typing practice — Russian, English, Python, C++
Exec="$app/stamina"
Icon=stamina
Terminal=false
Categories=Education;
Keywords=typing;keyboard;training;
DESKTOP
printf 'Stamina installed. Open Stamina from your applications menu.\n'
