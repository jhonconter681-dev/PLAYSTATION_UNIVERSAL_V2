// ─────────────────────────────────────────────────────────────
// PUCE Settings Provider
// ─────────────────────────────────────────────────────────────

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

// Provides the SharedPreferences instance synchronously (must be initialized in main)
final sharedPreferencesProvider = Provider<SharedPreferences>((ref) {
  throw UnimplementedError('sharedPreferencesProvider must be overridden');
});

class AppSettings {
  final bool darkMode;
  final String accentColor;
  final String defaultPsMode;
  final int pollingRateHz;
  final bool aiDriftCorrection;
  final bool aiAutoCalibrate;
  final bool aiAutoMap;
  final bool vibrationEnabled;
  final int hapticIntensity;
  final String latencyMode;
  final bool requireSignedPlugins;

  AppSettings({
    this.darkMode = true,
    this.accentColor = '#00B4D8',
    this.defaultPsMode = 'DualShock4',
    this.pollingRateHz = 1000,
    this.aiDriftCorrection = true,
    this.aiAutoCalibrate = true,
    this.aiAutoMap = true,
    this.vibrationEnabled = true,
    this.hapticIntensity = 80,
    this.latencyMode = 'standard',
    this.requireSignedPlugins = true,
  });

  AppSettings copyWith({
    bool? darkMode,
    String? accentColor,
    String? defaultPsMode,
    int? pollingRateHz,
    bool? aiDriftCorrection,
    bool? aiAutoCalibrate,
    bool? aiAutoMap,
    bool? vibrationEnabled,
    int? hapticIntensity,
    String? latencyMode,
    bool? requireSignedPlugins,
  }) {
    return AppSettings(
      darkMode: darkMode ?? this.darkMode,
      accentColor: accentColor ?? this.accentColor,
      defaultPsMode: defaultPsMode ?? this.defaultPsMode,
      pollingRateHz: pollingRateHz ?? this.pollingRateHz,
      aiDriftCorrection: aiDriftCorrection ?? this.aiDriftCorrection,
      aiAutoCalibrate: aiAutoCalibrate ?? this.aiAutoCalibrate,
      aiAutoMap: aiAutoMap ?? this.aiAutoMap,
      vibrationEnabled: vibrationEnabled ?? this.vibrationEnabled,
      hapticIntensity: hapticIntensity ?? this.hapticIntensity,
      latencyMode: latencyMode ?? this.latencyMode,
      requireSignedPlugins: requireSignedPlugins ?? this.requireSignedPlugins,
    );
  }
}

class SettingsNotifier extends StateNotifier<AppSettings> {
  SettingsNotifier() : super(AppSettings());

  // In a real app, this would read from SharedPreferences or SQLite database.
  // For now, we update the state directly.

  void toggleTheme() {
    state = state.copyWith(darkMode: !state.darkMode);
  }

  void updateSettings(AppSettings newSettings) {
    state = newSettings;
  }
}

final settingsProvider = StateNotifierProvider<SettingsNotifier, AppSettings>((ref) {
  return SettingsNotifier();
});
