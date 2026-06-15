// ─────────────────────────────────────────────────────────────
// PUCE Flutter UI — main.dart
// Entry point for the PlayStation Universal Controller Emulator app.
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:google_fonts/google_fonts.dart';

import 'screens/home_screen.dart';
import 'screens/mapping_screen.dart';
import 'screens/profiles_screen.dart';
import 'screens/settings_screen.dart';
import 'screens/device_detail_screen.dart';
import 'theme/app_theme.dart';
import 'providers/settings_provider.dart';
import 'ffi/puce_bridge.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Initialize the native PUCE core library
  await PuceBridge.instance.initialize();

  // Lock orientation to landscape on TV platforms
  await SystemChrome.setPreferredOrientations([
    DeviceOrientation.landscapeLeft,
    DeviceOrientation.landscapeRight,
    DeviceOrientation.portraitUp,
  ]);

  // Full-screen immersive mode for TV
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);

  runApp(
    const ProviderScope(
      child: PuceApp(),
    ),
  );
}

// ─────────────────────────────────────────────────────────────
// Router
// ─────────────────────────────────────────────────────────────

final _router = GoRouter(
  initialLocation: '/',
  routes: [
    GoRoute(
      path: '/',
      name: 'home',
      builder: (context, state) => const HomeScreen(),
    ),
    GoRoute(
      path: '/mapping',
      name: 'mapping',
      builder: (context, state) => const MappingScreen(),
    ),
    GoRoute(
      path: '/profiles',
      name: 'profiles',
      builder: (context, state) => const ProfilesScreen(),
    ),
    GoRoute(
      path: '/settings',
      name: 'settings',
      builder: (context, state) => const SettingsScreen(),
    ),
    GoRoute(
      path: '/device/:id',
      name: 'device-detail',
      builder: (context, state) => DeviceDetailScreen(
        deviceId: state.pathParameters['id'] ?? '',
      ),
    ),
  ],
  errorBuilder: (context, state) => Scaffold(
    body: Center(
      child: Text('Page not found: ${state.uri}'),
    ),
  ),
);

// ─────────────────────────────────────────────────────────────
// Root App Widget
// ─────────────────────────────────────────────────────────────

class PuceApp extends ConsumerWidget {
  const PuceApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(settingsProvider);
    final themeMode = settings.darkMode ? ThemeMode.dark : ThemeMode.light;

    return MaterialApp.router(
      title: 'PUCE — PlayStation Universal Controller Emulator',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.lightTheme(),
      darkTheme: AppTheme.darkTheme(),
      themeMode: themeMode,
      routerConfig: _router,
      builder: (context, child) {
        // Wrap in FocusTraversalGroup for TV D-pad navigation
        return FocusTraversalGroup(
          policy: ReadingOrderTraversalPolicy(),
          child: child ?? const SizedBox.shrink(),
        );
      },
    );
  }
}
