// lib/utils/constants.dart
// PUCE - PlayStation Universal Controller Emulator
// Application-wide constants

/// App version and metadata
class AppConstants {
  AppConstants._();

  static const String appName = 'PUCE';
  static const String appFullName = 'PlayStation Universal Controller Emulator';
  static const String appVersion = '1.0.0';
  static const String appBuildNumber = '1';
  static const String appTagline = 'Any Controller. Any PlayStation.';

  static const String githubUrl = 'https://github.com/puce-emulator/puce';
  static const String docsUrl = 'https://puce-emulator.github.io/docs';
  static const String discordUrl = 'https://discord.gg/puce';

  // Native library names
  static const String nativeLibWindows = 'puce_core.dll';
  static const String nativeLibLinux = 'libpuce_core.so';
  static const String nativeLibMacOS = 'libpuce_core.dylib';

  // Shared preferences keys
  static const String prefThemeMode = 'theme_mode';
  static const String prefAccentColor = 'accent_color';
  static const String prefPollingRate = 'polling_rate';
  static const String prefDefaultPsMode = 'default_ps_mode';
  static const String prefEnableAi = 'enable_ai';
  static const String prefLatencyMode = 'latency_mode';
  static const String prefAutoUpdate = 'auto_update';
  static const String prefSelectedProfile = 'selected_profile';
  static const String prefProfilesJson = 'profiles_json';
  static const String prefLastDevice = 'last_device';

  // Polling / timing
  static const int pollingRate125Hz = 125;   // 8ms interval
  static const int pollingRate250Hz = 250;   // 4ms interval
  static const int pollingRate500Hz = 500;   // 2ms interval
  static const int pollingRate1000Hz = 1000; // 1ms interval

  static const List<int> pollingRates = [125, 250, 500, 1000];

  static int pollingIntervalMs(int hz) => (1000 / hz).round();

  // Device polling (UI refresh)
  static const Duration deviceRefreshInterval = Duration(seconds: 1);
  static const Duration statusRefreshInterval = Duration(milliseconds: 100);
  static const Duration latencyGraphInterval = Duration(milliseconds: 50);

  // Latency chart
  static const int latencyHistoryLength = 60; // samples to keep

  // Latency thresholds (ms)
  static const double latencyExcellentMs = 2.0;
  static const double latencyGoodMs = 5.0;
  static const double latencyFairMs = 10.0;
  static const double latencyPoorMs = 20.0;

  // Battery thresholds (0.0–1.0)
  static const double batteryFullPct = 0.80;
  static const double batteryMidPct = 0.20;
  static const double batteryLowPct = 0.10;

  // API timeouts
  static const Duration apiTimeout = Duration(seconds: 5);
  static const Duration initTimeout = Duration(seconds: 10);

  // File extensions
  static const List<String> profileExtensions = ['json', 'puce'];
  static const List<String> pluginExtensions = ['dll', 'so', 'dylib'];

  // Animation
  static const double cardElevation = 0;
  static const double modalElevation = 24;
  static const double borderRadius = 16;
  static const double smallBorderRadius = 8;
  static const double largeBorderRadius = 24;

  // Layout
  static const double sidebarWidthCollapsed = 72;
  static const double sidebarWidthExpanded = 220;
  static const double topBarHeight = 64;
  static const double bottomBarHeight = 80;
  static const double cardSpacing = 16;
  static const double sectionSpacing = 32;
  static const double pageHorizontalPadding = 24;
  static const double pageVerticalPadding = 24;

  // Controller canvas
  static const double controllerAspectRatio = 1.618; // Golden ratio
}
