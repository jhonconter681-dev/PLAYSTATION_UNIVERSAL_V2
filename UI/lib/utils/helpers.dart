// lib/utils/helpers.dart
// PUCE - PlayStation Universal Controller Emulator
// Utility functions for formatting, icon selection, and color mapping

import 'package:flutter/material.dart';
import '../models/emulation_mode.dart';
import '../theme/app_colors.dart';

/// Format a latency value for display (e.g., "2.4 ms" or "< 1 ms")
String formatLatency(double ms) {
  if (ms < 0.5) return '< 1 ms';
  if (ms < 10) return '${ms.toStringAsFixed(1)} ms';
  return '${ms.toStringAsFixed(0)} ms';
}

/// Format battery level (0.0–1.0) for display (e.g., "87%")
String formatBattery(double level) {
  final pct = (level * 100).round();
  return '$pct%';
}

/// Format polling rate for display
String formatPollingRate(int hz) => '$hz Hz';

/// Format a USB/BT version string
String formatVersion(String? v) => v ?? 'N/A';

/// Format VID/PID as hex string
String formatHexId(int value) =>
    '0x${value.toRadixString(16).toUpperCase().padLeft(4, '0')}';

/// Returns an icon for the given device type string
IconData deviceTypeToIcon(String deviceType) {
  switch (deviceType.toLowerCase()) {
    case 'playstation':
    case 'ps4':
    case 'ps5':
    case 'dualshock':
    case 'dualsense':
      return Icons.sports_esports_rounded;
    case 'xbox':
      return Icons.gamepad_rounded;
    case 'nintendo':
    case 'switch':
    case 'joycon':
      return Icons.sports_esports_outlined;
    case 'keyboard':
      return Icons.keyboard_rounded;
    case 'mouse':
      return Icons.mouse_rounded;
    case 'arcade':
    case 'fightstick':
      return Icons.grid_view_rounded;
    case 'steering':
    case 'wheel':
      return Icons.track_changes_rounded;
    case 'flight':
    case 'joystick':
      return Icons.flight_takeoff_rounded;
    case 'guitar':
    case 'instrument':
      return Icons.music_note_rounded;
    default:
      return Icons.gamepad_outlined;
  }
}

/// Returns an icon for the given connection type string
IconData connectionTypeToIcon(String connectionType) {
  switch (connectionType.toLowerCase()) {
    case 'bluetooth':
      return Icons.bluetooth_rounded;
    case 'usb':
      return Icons.usb_rounded;
    case '2.4ghz':
    case 'wireless':
      return Icons.wifi_rounded;
    default:
      return Icons.device_unknown_rounded;
  }
}

/// Returns the brand color for a given PS mode
Color psModeToColor(EmulationMode mode) => mode.color;

/// Returns the glow color for a given PS mode
Color psModeToGlowColor(EmulationMode mode) => mode.glowColor;

/// Returns a color based on latency value
Color latencyToColor(double ms) => AppColors.latencyColor(ms);

/// Returns a color based on battery level
Color batteryToColor(double level) => AppColors.batteryColor(level);

/// Get friendly label for connection type
String connectionTypeLabel(String connectionType) {
  switch (connectionType.toLowerCase()) {
    case 'bluetooth':
      return 'Bluetooth';
    case 'usb':
      return 'USB';
    case '2.4ghz':
      return '2.4 GHz';
    case 'wireless':
      return 'Wireless';
    default:
      return 'Unknown';
  }
}

/// Get manufacturer logo icon (best approximation with material icons)
IconData manufacturerToIcon(String manufacturer) {
  final m = manufacturer.toLowerCase();
  if (m.contains('sony') || m.contains('playstation')) {
    return Icons.sports_esports_rounded;
  }
  if (m.contains('microsoft') || m.contains('xbox')) {
    return Icons.gamepad_rounded;
  }
  if (m.contains('nintendo')) {
    return Icons.sports_esports_outlined;
  }
  if (m.contains('8bitdo') || m.contains('8bit')) {
    return Icons.videogame_asset_rounded;
  }
  if (m.contains('logitech')) {
    return Icons.devices_rounded;
  }
  if (m.contains('razer')) {
    return Icons.computer_rounded;
  }
  if (m.contains('steelseries')) {
    return Icons.headset_rounded;
  }
  return Icons.developer_board_rounded;
}

/// Format a DateTime as a relative string (e.g., "2 minutes ago")
String formatRelativeTime(DateTime? dt) {
  if (dt == null) return 'Never';
  final diff = DateTime.now().difference(dt);
  if (diff.inSeconds < 10) return 'Just now';
  if (diff.inSeconds < 60) return '${diff.inSeconds}s ago';
  if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
  if (diff.inHours < 24) return '${diff.inHours}h ago';
  return '${diff.inDays}d ago';
}

/// Clamp and apply deadzone to an axis value
double applyDeadzone(double value, double deadzone) {
  if (value.abs() < deadzone) return 0.0;
  final sign = value.sign;
  final scaled = (value.abs() - deadzone) / (1.0 - deadzone);
  return sign * scaled.clamp(0.0, 1.0);
}

/// Generate a unique profile ID
String generateProfileId() {
  final now = DateTime.now().millisecondsSinceEpoch;
  return 'profile_$now';
}

/// Truncate a string to max length with ellipsis
String truncate(String s, int maxLen) {
  if (s.length <= maxLen) return s;
  return '${s.substring(0, maxLen - 1)}…';
}

/// Convert a sensor name to a display label + icon
({String label, IconData icon}) sensorMeta(String sensor) {
  final s = sensor.toLowerCase();
  if (s.contains('gyro')) {
    return (label: 'Gyroscope', icon: Icons.rotate_90_degrees_ccw_rounded);
  }
  if (s.contains('accel')) {
    return (label: 'Accelerometer', icon: Icons.speed_rounded);
  }
  if (s.contains('touch')) {
    return (label: 'Touchpad', icon: Icons.touch_app_rounded);
  }
  if (s.contains('light') || s.contains('led')) {
    return (label: 'Light Bar', icon: Icons.light_mode_rounded);
  }
  if (s.contains('mic') || s.contains('audio')) {
    return (label: 'Microphone', icon: Icons.mic_rounded);
  }
  if (s.contains('speak')) {
    return (label: 'Speaker', icon: Icons.volume_up_rounded);
  }
  if (s.contains('haptic') || s.contains('rumble')) {
    return (label: 'Haptic Feedback', icon: Icons.vibration_rounded);
  }
  if (s.contains('trigger') || s.contains('adaptive')) {
    return (label: 'Adaptive Triggers', icon: Icons.adjust_rounded);
  }
  return (label: sensor, icon: Icons.sensors_rounded);
}

/// Capability to display metadata
({String label, IconData icon, Color color}) capabilityMeta(String cap) {
  final c = cap.toLowerCase();
  if (c.contains('rumble') || c.contains('haptic')) {
    return (
      label: 'Haptic',
      icon: Icons.vibration_rounded,
      color: AppColors.accent
    );
  }
  if (c.contains('touch')) {
    return (
      label: 'Touchpad',
      icon: Icons.touch_app_rounded,
      color: AppColors.primary
    );
  }
  if (c.contains('motion') || c.contains('gyro')) {
    return (
      label: 'Motion',
      icon: Icons.rotate_90_degrees_ccw_rounded,
      color: AppColors.secondary
    );
  }
  if (c.contains('bluetooth') || c.contains('bt')) {
    return (
      label: 'Bluetooth',
      icon: Icons.bluetooth_rounded,
      color: AppColors.bluetooth
    );
  }
  if (c.contains('usb')) {
    return (
      label: 'USB',
      icon: Icons.usb_rounded,
      color: AppColors.usb
    );
  }
  if (c.contains('led') || c.contains('light')) {
    return (
      label: 'LED',
      icon: Icons.light_mode_rounded,
      color: AppColors.secondary
    );
  }
  if (c.contains('mic') || c.contains('audio')) {
    return (
      label: 'Audio',
      icon: Icons.mic_rounded,
      color: AppColors.psTriangle
    );
  }
  return (
    label: cap,
    icon: Icons.check_circle_outline_rounded,
    color: AppColors.success
  );
}
