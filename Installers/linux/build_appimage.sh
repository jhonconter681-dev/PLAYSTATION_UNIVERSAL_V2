#!/bin/bash
set -e

VERSION="1.0.0"
APPDIR="PUCE.AppDir"

# Create AppDir structure
mkdir -p ${APPDIR}/usr/bin
mkdir -p ${APPDIR}/usr/lib
mkdir -p ${APPDIR}/usr/share/applications
mkdir -p ${APPDIR}/usr/share/icons/hicolor/256x256/apps

# Copy AppRun
cp AppRun ${APPDIR}/
chmod +x ${APPDIR}/AppRun

# Copy Desktop Entry
cat > ${APPDIR}/PUCE.desktop <<EOF
[Desktop Entry]
Name=PUCE
Exec=PUCE
Icon=PUCE
Type=Application
Categories=Utility;HardwareSettings;
Comment=PlayStation Universal Controller Emulator
EOF

# Copy binaries (assuming they are built)
# cp ../../target/release/PUCE ${APPDIR}/usr/bin/
# cp ../../target/release/libpuce_core.so ${APPDIR}/usr/lib/

# Download linuxdeploy if needed and build AppImage
# wget -c https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
# chmod +x linuxdeploy-x86_64.AppImage
# ./linuxdeploy-x86_64.AppImage --appdir ${APPDIR} --output appimage
