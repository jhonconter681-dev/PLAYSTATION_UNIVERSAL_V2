# PUCE Roadmap

This document outlines the strategic goals and feature additions planned for PUCE.

## v1.0.0 (Current)
- Complete Rust Core (Detection, Emulation, Virtual Drivers).
- Stunning Flutter UI.
- Local AI drift correction and auto-calibration.
- Dynamic plugin system loading (C-ABI).
- Installers for Windows (NSIS) and Linux (AppImage).

## v1.1.0
- **Android Port**: Finalize the Kotlin virtual input service and deploy the Flutter UI to Android.
- **macOS DMG**: Complete IOKit driver signing and release macOS binaries.
- **Cloud Profiles**: Ability to share and download community controller profiles.

## v2.0.0 (Next-Gen Features)
- **PSVR2 Sense Controller Support**: Reverse engineering of PSVR2 controllers for use in standard PC VR games.
- **Advanced AI Context Mapping**: The AI engine will read screen pixels (via OCR/Computer Vision) to automatically switch mappings depending on the game being played.
- **True Network Virtualization**: Emulate a controller over the network (e.g., using a phone as a PS5 controller for the PC via Wi-Fi direct).

## Long-Term Vision
- Guarantee 100% compatibility with all future PlayStation hardware iterations (e.g., "PS6") through OTA plugin updates.
- Achieve < 0.5ms end-to-end latency for competitive e-sports emulation.
