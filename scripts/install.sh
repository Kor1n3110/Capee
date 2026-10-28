#!/bin/sh
set -eu
cd "$(dirname "$0")"
base="${XDG_DATA_HOME:-$HOME/.local/share}"
app="$base/capee/app"
mkdir -p "$app" "$base/applications" "$base/icons/hicolor/scalable/apps"
cp capee "$app/capee"
chmod +x "$app/capee"
cp capee.svg "$base/icons/hicolor/scalable/apps/capee.svg"
case "$app" in *'"'*|*'`'*|*'$'*|*'\'*) echo 'Unsupported characters in installation path' >&2; exit 1;; esac
cat > "$base/applications/capee.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Capee
Comment=Touch typing practice — Russian, English, Python, C++
Exec="$app/capee"
Icon=capee
Terminal=false
Categories=Education;
Keywords=typing;keyboard;training;
DESKTOP
printf 'Capee installed. Open Capee from your applications menu.\n'
