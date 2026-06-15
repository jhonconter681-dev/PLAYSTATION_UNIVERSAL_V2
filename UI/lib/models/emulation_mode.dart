// lib/models/emulation_mode.dart
// PUCE - PlayStation Universal Controller Emulator
// PlayStation emulation mode enum with metadata

import 'package:flutter/material.dart';
import '../theme/app_colors.dart';

/// All supported PlayStation emulation modes
enum EmulationMode {
  ps1,
  ps2,
  ps3,
  ps4,
  dualSense,
  dualSenseEdge,
}

/// Rich metadata extension for EmulationMode
extension EmulationModeExtension on EmulationMode {
  // ─── Display Name ───────────────────────────────────────────────────────
  String get displayName {
    switch (this) {
      case EmulationMode.ps1:
        return 'PlayStation 1';
      case EmulationMode.ps2:
        return 'PlayStation 2';
      case EmulationMode.ps3:
        return 'PlayStation 3';
      case EmulationMode.ps4:
        return 'PlayStation 4';
      case EmulationMode.dualSense:
        return 'DualSense';
      case EmulationMode.dualSenseEdge:
        return 'DualSense Edge';
    }
  }

  // ─── Short Name ─────────────────────────────────────────────────────────
  String get shortName {
    switch (this) {
      case EmulationMode.ps1:
        return 'PS1';
      case EmulationMode.ps2:
        return 'PS2';
      case EmulationMode.ps3:
        return 'PS3';
      case EmulationMode.ps4:
        return 'PS4';
      case EmulationMode.dualSense:
        return 'DS5';
      case EmulationMode.dualSenseEdge:
        return 'Edge';
    }
  }

  // ─── Controller Model Name ──────────────────────────────────────────────
  String get controllerName {
    switch (this) {
      case EmulationMode.ps1:
        return 'DualShock';
      case EmulationMode.ps2:
        return 'DualShock 2';
      case EmulationMode.ps3:
        return 'DualShock 3';
      case EmulationMode.ps4:
        return 'DualShock 4';
      case EmulationMode.dualSense:
        return 'DualSense';
      case EmulationMode.dualSenseEdge:
        return 'DualSense Edge';
    }
  }

  // ─── Description ────────────────────────────────────────────────────────
  String get description {
    switch (this) {
      case EmulationMode.ps1:
        return 'Original PlayStation controller with digital buttons and analog sticks. Compatible with PSX emulators.';
      case EmulationMode.ps2:
        return 'DualShock 2 with pressure-sensitive buttons and dual analog sticks. Full PS2 compatibility.';
      case EmulationMode.ps3:
        return 'DualShock 3 with USB/Bluetooth, SIXAXIS motion, rumble, and pressure-sensitive buttons.';
      case EmulationMode.ps4:
        return 'DualShock 4 with touchpad, light bar, speaker, Share button, and enhanced motion sensors.';
      case EmulationMode.dualSense:
        return 'PS5 DualSense with haptic feedback, adaptive triggers, built-in microphone, and Create button.';
      case EmulationMode.dualSenseEdge:
        return 'Professional DualSense Edge with back buttons, customizable sticks, and advanced haptics.';
    }
  }

  // ─── Icon ───────────────────────────────────────────────────────────────
  IconData get icon {
    switch (this) {
      case EmulationMode.ps1:
        return Icons.videogame_asset_outlined;
      case EmulationMode.ps2:
        return Icons.gamepad_outlined;
      case EmulationMode.ps3:
        return Icons.sports_esports_outlined;
      case EmulationMode.ps4:
        return Icons.gamepad_rounded;
      case EmulationMode.dualSense:
        return Icons.sports_esports_rounded;
      case EmulationMode.dualSenseEdge:
        return Icons.settings_input_composite_rounded;
    }
  }

  // ─── Primary Color ──────────────────────────────────────────────────────
  Color get color {
    switch (this) {
      case EmulationMode.ps1:
        return AppColors.ps1Color;
      case EmulationMode.ps2:
        return AppColors.ps2Color;
      case EmulationMode.ps3:
        return AppColors.ps3Color;
      case EmulationMode.ps4:
        return AppColors.ps4Color;
      case EmulationMode.dualSense:
        return AppColors.dualSenseColor;
      case EmulationMode.dualSenseEdge:
        return AppColors.dualSenseEdgeColor;
    }
  }

  // ─── Accent / Glow Color ────────────────────────────────────────────────
  Color get glowColor {
    switch (this) {
      case EmulationMode.ps1:
        return const Color(0xFF9B9B9B);
      case EmulationMode.ps2:
        return const Color(0xFF2C6EAB);
      case EmulationMode.ps3:
        return const Color(0xFF0070CC);
      case EmulationMode.ps4:
        return AppColors.primary;
      case EmulationMode.dualSense:
        return const Color(0xFFE0E0FF);
      case EmulationMode.dualSenseEdge:
        return AppColors.secondary;
    }
  }

  // ─── API / native string ID ──────────────────────────────────────────────
  String get apiId {
    switch (this) {
      case EmulationMode.ps1:
        return 'ps1';
      case EmulationMode.ps2:
        return 'ps2';
      case EmulationMode.ps3:
        return 'ps3';
      case EmulationMode.ps4:
        return 'ps4';
      case EmulationMode.dualSense:
        return 'dualsense';
      case EmulationMode.dualSenseEdge:
        return 'dualsense_edge';
    }
  }

  // ─── Features supported by this mode ────────────────────────────────────
  List<String> get features {
    switch (this) {
      case EmulationMode.ps1:
        return ['Digital buttons', 'Analog sticks', 'Rumble'];
      case EmulationMode.ps2:
        return [
          'Digital buttons',
          'Pressure-sensitive buttons',
          'Analog sticks',
          'Rumble'
        ];
      case EmulationMode.ps3:
        return [
          'Digital buttons',
          'Pressure-sensitive buttons',
          'Analog sticks',
          'SIXAXIS motion',
          'Rumble',
          'USB',
          'Bluetooth'
        ];
      case EmulationMode.ps4:
        return [
          'Digital buttons',
          'Analog sticks',
          'Touchpad',
          'Light bar',
          'Speaker',
          'Motion sensors',
          'Share button',
          'USB',
          'Bluetooth'
        ];
      case EmulationMode.dualSense:
        return [
          'Digital buttons',
          'Analog sticks',
          'Touchpad',
          'Haptic feedback',
          'Adaptive triggers',
          'Built-in mic',
          'Motion sensors',
          'Create button',
          'USB',
          'Bluetooth'
        ];
      case EmulationMode.dualSenseEdge:
        return [
          'All DualSense features',
          'Back buttons',
          'Replaceable stick modules',
          'Adjustable trigger travel',
          'Custom button remapping',
          'Enhanced haptics',
          'Fn buttons'
        ];
    }
  }

  // ─── Parse from API string ───────────────────────────────────────────────
  static EmulationMode fromApiId(String id) {
    return EmulationMode.values.firstWhere(
      (m) => m.apiId == id,
      orElse: () => EmulationMode.ps4,
    );
  }
}
