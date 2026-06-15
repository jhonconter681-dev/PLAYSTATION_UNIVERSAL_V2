// ─────────────────────────────────────────────────────────────
// PUCE Device Detail Screen
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../providers/device_provider.dart';
import '../theme/app_colors.dart';
import '../theme/app_theme.dart';
import '../widgets/battery_indicator.dart';
import '../widgets/latency_chart.dart';

class DeviceDetailScreen extends ConsumerWidget {
  final String deviceId;

  const DeviceDetailScreen({super.key, required this.deviceId});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final devicesAsync = ref.watch(deviceListProvider);
    
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
        title: const Text('Device Details'),
      ),
      body: devicesAsync.when(
        data: (devices) {
          final device = devices.where((d) => d.id == deviceId).firstOrNull;
          if (device == null) return const Center(child: Text('Device disconnected'));

          return ListView(
            padding: const EdgeInsets.all(32),
            children: [
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Container(
                    width: 120,
                    height: 120,
                    decoration: AppTheme.glassCard(),
                    child: Icon(Icons.gamepad, size: 64, color: AppColors.primaryCyan),
                  ),
                  const SizedBox(width: 32),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(device.name, style: Theme.of(context).textTheme.displaySmall),
                        const SizedBox(height: 8),
                        Text(device.manufacturer, style: Theme.of(context).textTheme.titleMedium?.copyWith(color: AppColors.textSecondary)),
                        const SizedBox(height: 16),
                        Row(
                          children: [
                            BatteryIndicator(level: device.batteryLevel, isCharging: false),
                            const SizedBox(width: 24),
                            _buildInfoChip('VID', '0x${device.vendorId.toRadixString(16).padLeft(4, '0').toUpperCase()}'),
                            const SizedBox(width: 8),
                            _buildInfoChip('PID', '0x${device.productId.toRadixString(16).padLeft(4, '0').toUpperCase()}'),
                            const SizedBox(width: 8),
                            _buildInfoChip('Type', device.deviceType),
                          ],
                        ),
                      ],
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 48),

              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Capabilities', style: Theme.of(context).textTheme.titleLarge),
                        const SizedBox(height: 16),
                        Wrap(
                          spacing: 8,
                          runSpacing: 8,
                          children: device.capabilities.map((c) => Chip(
                            label: Text(c),
                            backgroundColor: AppColors.darkCard,
                            side: const BorderSide(color: AppColors.primaryCyan),
                          )).toList(),
                        ),
                        const SizedBox(height: 32),
                        Text('Sensors', style: Theme.of(context).textTheme.titleLarge),
                        const SizedBox(height: 16),
                        Wrap(
                          spacing: 8,
                          runSpacing: 8,
                          children: device.sensors.map((s) => Chip(
                            label: Text(s),
                            backgroundColor: AppColors.darkCard,
                          )).toList(),
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 32),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text('Performance', style: Theme.of(context).textTheme.titleLarge),
                        const SizedBox(height: 16),
                        Container(
                          height: 200,
                          decoration: AppTheme.glassCard(),
                          padding: const EdgeInsets.all(16),
                          child: const LatencyChart(),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ],
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, s) => Center(child: Text('Error: $e')),
      ),
    );
  }

  Widget _buildInfoChip(String label, String value) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: AppColors.darkCard,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: AppColors.borderSubtle),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text('$label:', style: const TextStyle(color: AppColors.textSecondary, fontSize: 12)),
          const SizedBox(width: 4),
          Text(value, style: const TextStyle(color: AppColors.textPrimary, fontWeight: FontWeight.bold, fontSize: 12)),
        ],
      ),
    );
  }
}
