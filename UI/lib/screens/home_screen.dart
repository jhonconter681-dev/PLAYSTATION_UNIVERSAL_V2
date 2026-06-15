// ─────────────────────────────────────────────────────────────
// PUCE Home Screen
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:flutter_animate/flutter_animate.dart';

import '../providers/device_provider.dart';
import '../models/emulation_mode.dart';
import '../theme/app_colors.dart';
import '../theme/app_theme.dart';
import '../widgets/device_card.dart';
import '../widgets/ps_mode_selector.dart';
import '../widgets/connection_badge.dart';
import '../widgets/latency_chart.dart';

class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final devicesAsync = ref.watch(deviceListProvider);
    final selectedDevice = ref.watch(selectedDeviceProvider);

    return Scaffold(
      appBar: AppBar(
        title: Row(
          children: [
            // Simplified PS logo simulation
            Icon(Icons.gamepad, color: AppColors.primaryCyan, size: 28),
            const SizedBox(width: 12),
            const Text('P U C E', style: TextStyle(fontWeight: FontWeight.bold, letterSpacing: 2.0)),
          ],
        ),
        actions: [
          const ConnectionBadge(),
          IconButton(
            icon: const Icon(Icons.settings),
            onPressed: () => context.push('/settings'),
          ),
          const SizedBox(width: 16),
        ],
      ),
      body: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Sidebar / Menu
          _buildSidebar(context),
          
          // Main Content
          Expanded(
            child: devicesAsync.when(
              data: (devices) {
                if (devices.isEmpty) {
                  return _buildEmptyState(context);
                }

                // Auto-select first device if none selected
                WidgetsBinding.instance.addPostFrameCallback((_) {
                  if (ref.read(selectedDeviceIdProvider) == null && devices.isNotEmpty) {
                    ref.read(selectedDeviceIdProvider.notifier).state = devices.first.id;
                  }
                });

                return _buildDashboard(context, ref, devices, selectedDevice);
              },
              loading: () => const Center(child: CircularProgressIndicator(color: AppColors.primaryCyan)),
              error: (err, stack) => Center(child: Text('Error: $err', style: TextStyle(color: AppColors.errorRed))),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSidebar(BuildContext context) {
    return Container(
      width: 250,
      decoration: BoxDecoration(
        color: AppColors.darkCard,
        border: Border(right: BorderSide(color: AppColors.borderSubtle)),
      ),
      child: Column(
        children: [
          const SizedBox(height: 24),
          _buildNavTile(context, 'Dashboard', Icons.dashboard, '/', true),
          _buildNavTile(context, 'Mapping & AI', Icons.account_tree, '/mapping', false),
          _buildNavTile(context, 'Profiles', Icons.save, '/profiles', false),
          _buildNavTile(context, 'Plugins', Icons.extension, '/settings', false),
          const Spacer(),
          Padding(
            padding: const EdgeInsets.all(16.0),
            child: Container(
              padding: const EdgeInsets.all(16),
              decoration: AppTheme.glassCard(),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Core Status', style: Theme.of(context).textTheme.labelLarge),
                  const SizedBox(height: 8),
                  Row(
                    children: [
                      Container(width: 8, height: 8, decoration: const BoxDecoration(color: Colors.green, shape: BoxShape.circle)),
                      const SizedBox(width: 8),
                      Text('Engine Running', style: Theme.of(context).textTheme.bodySmall),
                    ],
                  ),
                  const SizedBox(height: 4),
                  Row(
                    children: [
                      Container(width: 8, height: 8, decoration: const BoxDecoration(color: AppColors.primaryCyan, shape: BoxShape.circle)),
                      const SizedBox(width: 8),
                      Text('AI Active', style: Theme.of(context).textTheme.bodySmall),
                    ],
                  )
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildNavTile(BuildContext context, String title, IconData icon, String route, bool isActive) {
    return ListTile(
      leading: Icon(icon, color: isActive ? AppColors.primaryCyan : AppColors.textSecondary),
      title: Text(
        title, 
        style: TextStyle(
          color: isActive ? AppColors.textPrimary : AppColors.textSecondary,
          fontWeight: isActive ? FontWeight.bold : FontWeight.normal,
        ),
      ),
      selected: isActive,
      selectedTileColor: AppColors.primaryCyan.withOpacity(0.1),
      onTap: () {
        if (!isActive) context.push(route);
      },
    );
  }

  Widget _buildEmptyState(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.gamepad_outlined, size: 80, color: AppColors.textDisabled)
              .animate(onPlay: (controller) => controller.repeat())
              .shimmer(duration: const Duration(seconds: 2), color: AppColors.primaryCyan.withOpacity(0.5)),
          const SizedBox(height: 24),
          Text(
            'Waiting for Controller...',
            style: Theme.of(context).textTheme.headlineMedium,
          ),
          const SizedBox(height: 12),
          Text(
            'Connect via USB or Bluetooth.\nPUCE supports all generic inputs.',
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.bodyLarge?.copyWith(color: AppColors.textSecondary),
          ),
        ],
      ),
    );
  }

  Widget _buildDashboard(BuildContext context, WidgetRef ref, List<dynamic> devices, dynamic selectedDevice) {
    return ListView(
      padding: const EdgeInsets.all(32),
      children: [
        // Top Section: Device Selector & Status
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Left: Device Card
            Expanded(
              flex: 2,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Connected Device', style: Theme.of(context).textTheme.headlineSmall),
                  const SizedBox(height: 16),
                  if (selectedDevice != null)
                    DeviceCard(device: selectedDevice)
                        .animate()
                        .fadeIn(duration: AppTheme.normalAnim)
                        .slideY(begin: 0.1, end: 0),
                ],
              ),
            ),
            const SizedBox(width: 32),
            // Right: Real-time latency chart
            Expanded(
              flex: 3,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('Input Latency', style: Theme.of(context).textTheme.headlineSmall),
                  const SizedBox(height: 16),
                  Container(
                    height: 160,
                    decoration: AppTheme.glassCard(),
                    padding: const EdgeInsets.all(16),
                    child: const LatencyChart(),
                  ).animate().fadeIn().slideX(begin: 0.1, end: 0),
                ],
              ),
            ),
          ],
        ),

        const SizedBox(height: 48),

        // Emulation Mode Selector
        Text('Emulation Mode', style: Theme.of(context).textTheme.headlineSmall),
        const SizedBox(height: 16),
        PSModeSelector(
          selectedMode: selectedDevice != null 
              ? ref.watch(emulationModeProvider)[selectedDevice.id] ?? EmulationMode.dualShock4
              : EmulationMode.dualShock4,
          onModeSelected: (mode) {
            if (selectedDevice != null) {
              ref.read(emulationModeProvider.notifier).setMode(selectedDevice.id, mode);
            }
          },
        ).animate().fadeIn(delay: const Duration(milliseconds: 200)),

        const SizedBox(height: 48),

        // Bottom Action Bar
        Center(
          child: ElevatedButton.icon(
            icon: const Icon(Icons.power_settings_new, size: 28),
            label: const Text('EMULATE CONTROLLER', style: TextStyle(fontSize: 18, letterSpacing: 1.5)),
            style: ElevatedButton.styleFrom(
              padding: const EdgeInsets.symmetric(horizontal: 48, vertical: 20),
              backgroundColor: AppColors.primaryCyan,
              foregroundColor: AppColors.darkBackground,
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(30)),
            ),
            onPressed: () {
              ScaffoldMessenger.of(context).showSnackBar(
                SnackBar(
                  content: Text('Virtual Controller Activated! System now sees a PlayStation controller.'),
                  backgroundColor: AppColors.primaryCyan,
                  behavior: SnackBarBehavior.floating,
                )
              );
            },
          ).animate(onPlay: (controller) => controller.repeat(reverse: true))
           .scaleXY(end: 1.05, duration: const Duration(seconds: 1)),
        ),
      ],
    );
  }
}
