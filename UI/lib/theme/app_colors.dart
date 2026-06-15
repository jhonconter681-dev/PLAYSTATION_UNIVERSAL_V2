// lib/theme/app_colors.dart
// PUCE - PlayStation Universal Controller Emulator
// Complete color palette organized by category

import 'package:flutter/material.dart';

/// Core brand colors — PlayStation-inspired palette
class AppColors {
  AppColors._(); // Prevent instantiation

  // ─── Background ───────────────────────────────────────────────────────────
  static const Color backgroundDark = Color(0xFF080C14);
  static const Color backgroundDark2 = Color(0xFF0A0F1E);
  static const Color surfaceDark = Color(0xFF0D1526);
  static const Color surfaceDark2 = Color(0xFF111D35);
  static const Color cardDark = Color(0xFF121E36);
  static const Color cardDark2 = Color(0xFF16243F);

  static const Color backgroundLight = Color(0xFFF0F4FF);
  static const Color surfaceLight = Color(0xFFFFFFFF);
  static const Color cardLight = Color(0xFFF8FAFF);

  // ─── Primary / Cyan ───────────────────────────────────────────────────────
  static const Color primary = Color(0xFF00B4D8);
  static const Color primaryLight = Color(0xFF48CAE4);
  static const Color primaryDark = Color(0xFF0096C7);
  static const Color primaryDarker = Color(0xFF0077B6);
  static const Color primaryGlow = Color(0x4000B4D8);
  static const Color primarySubtle = Color(0x1A00B4D8);

  // ─── Secondary / Gold ─────────────────────────────────────────────────────
  static const Color secondary = Color(0xFFFFB700);
  static const Color secondaryLight = Color(0xFFFFCC44);
  static const Color secondaryDark = Color(0xFFE09E00);
  static const Color secondaryGlow = Color(0x40FFB700);
  static const Color secondarySubtle = Color(0x1AFFB700);

  // ─── Accent / PlayStation Purple ──────────────────────────────────────────
  static const Color accent = Color(0xFF7B2FBE);
  static const Color accentLight = Color(0xFF9B59D0);
  static const Color accentDark = Color(0xFF5C1F9A);
  static const Color accentGlow = Color(0x407B2FBE);
  static const Color accentSubtle = Color(0x1A7B2FBE);

  // ─── PlayStation Button Colors ─────────────────────────────────────────────
  static const Color psCircle = Color(0xFFE74C3C);    // Red circle
  static const Color psCross = Color(0xFF4A90E2);     // Blue cross
  static const Color psSquare = Color(0xFFE91E8C);    // Pink/magenta square
  static const Color psTriangle = Color(0xFF27AE60);  // Green triangle

  // ─── PS Mode Brand Colors ─────────────────────────────────────────────────
  static const Color ps1Color = Color(0xFF9B9B9B);    // Gray
  static const Color ps2Color = Color(0xFF2C6EAB);    // Dark blue
  static const Color ps3Color = Color(0xFF1E3A5F);    // Navy
  static const Color ps4Color = Color(0xFF0070CC);    // PlayStation blue
  static const Color dualSenseColor = Color(0xFF6C6C82); // DualSense white-gray
  static const Color dualSenseEdgeColor = Color(0xFF1A1A2E); // Edge black

  // ─── Status / Semantic Colors ─────────────────────────────────────────────
  static const Color success = Color(0xFF00E676);
  static const Color successDark = Color(0xFF00C853);
  static const Color successGlow = Color(0x4000E676);
  static const Color successSubtle = Color(0x1A00E676);

  static const Color warning = Color(0xFFFFAB00);
  static const Color warningDark = Color(0xFFFF8F00);
  static const Color warningGlow = Color(0x40FFAB00);

  static const Color error = Color(0xFFFF1744);
  static const Color errorDark = Color(0xFFD50000);
  static const Color errorGlow = Color(0x40FF1744);
  static const Color errorSubtle = Color(0x1AFF1744);

  static const Color info = Color(0xFF2979FF);
  static const Color infoGlow = Color(0x402979FF);

  // ─── Latency Color Thresholds ─────────────────────────────────────────────
  static const Color latencyExcellent = Color(0xFF00E676); // < 2ms
  static const Color latencyGood = Color(0xFF76FF03);      // 2–5ms
  static const Color latencyFair = Color(0xFFFFD740);      // 5–10ms
  static const Color latencyPoor = Color(0xFFFF6D00);      // 10–20ms
  static const Color latencyBad = Color(0xFFFF1744);       // > 20ms

  // ─── Battery Color Thresholds ─────────────────────────────────────────────
  static const Color batteryFull = Color(0xFF00E676);   // > 80%
  static const Color batteryMid = Color(0xFFFFD740);    // 20–80%
  static const Color batteryLow = Color(0xFFFF6D00);    // 10–20%
  static const Color batteryCritical = Color(0xFFFF1744); // < 10%

  // ─── Connectivity ─────────────────────────────────────────────────────────
  static const Color bluetooth = Color(0xFF0057B8);
  static const Color bluetoothLight = Color(0xFF4A90FF);
  static const Color usb = Color(0xFF00B894);
  static const Color usbLight = Color(0xFF00D2A0);

  // ─── Text ─────────────────────────────────────────────────────────────────
  static const Color textPrimary = Color(0xFFEAEEFF);
  static const Color textSecondary = Color(0xFF8A99C0);
  static const Color textDisabled = Color(0xFF3D4F72);
  static const Color textHint = Color(0xFF4A5F80);
  static const Color textLight = Color(0xFF1A253D);
  static const Color textLightSecondary = Color(0xFF4A5F80);

  // ─── Border / Divider ─────────────────────────────────────────────────────
  static const Color borderDark = Color(0xFF1E3050);
  static const Color borderDarkSubtle = Color(0xFF162440);
  static const Color borderLight = Color(0xFFCCD6F6);
  static const Color dividerDark = Color(0xFF121E36);
  static const Color glassBorder = Color(0x2600B4D8);
  static const Color glassBorderLight = Color(0x4000B4D8);

  // ─── Gradients ────────────────────────────────────────────────────────────
  static const LinearGradient primaryGradient = LinearGradient(
    colors: [primary, primaryDarker],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
  );

  static const LinearGradient accentGradient = LinearGradient(
    colors: [accent, Color(0xFF4A00E0)],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
  );

  static const LinearGradient goldGradient = LinearGradient(
    colors: [secondary, Color(0xFFFF8C00)],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
  );

  static const LinearGradient cardGradient = LinearGradient(
    colors: [cardDark, cardDark2],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
  );

  static const LinearGradient backgroundGradient = LinearGradient(
    colors: [backgroundDark, backgroundDark2, Color(0xFF0C1428)],
    begin: Alignment.topCenter,
    end: Alignment.bottomCenter,
  );

  static const LinearGradient controllerGradient = LinearGradient(
    colors: [Color(0xFF1A2540), Color(0xFF0D1526)],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
  );

  static const LinearGradient glassGradient = LinearGradient(
    colors: [Color(0x1AFFFFFF), Color(0x08FFFFFF)],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
  );

  static const RadialGradient glowGradient = RadialGradient(
    colors: [Color(0x6600B4D8), Colors.transparent],
    radius: 1.0,
  );

  // ─── Overlays ─────────────────────────────────────────────────────────────
  static const Color overlayDark = Color(0x80000000);
  static const Color overlayLight = Color(0x40000000);
  static const Color scrimDark = Color(0xCC080C14);

  // ─── Shadow ───────────────────────────────────────────────────────────────
  static const Color shadowDark = Color(0x66000000);
  static const Color shadowPrimary = Color(0x4000B4D8);
  static const Color shadowGold = Color(0x40FFB700);

  // ─── Helper: get latency color ────────────────────────────────────────────
  static Color latencyColor(double ms) {
    if (ms < 2) return latencyExcellent;
    if (ms < 5) return latencyGood;
    if (ms < 10) return latencyFair;
    if (ms < 20) return latencyPoor;
    return latencyBad;
  }

  // ─── Helper: get battery color ────────────────────────────────────────────
  static Color batteryColor(double pct) {
    if (pct > 0.8) return batteryFull;
    if (pct > 0.2) return batteryMid;
    if (pct > 0.1) return batteryLow;
    return batteryCritical;
  }
}
