// ─────────────────────────────────────────────────────────────
// PUCE Widgets — Device Card
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../models/device_info.dart';
import '../theme/app_colors.dart';
import '../theme/app_theme.dart';
import 'battery_indicator.dart';

class DeviceCard extends StatelessWidget {
  final DeviceInfo device;

  const DeviceCard({super.key, required this.device});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: () => context.push('/device/${device.id}'),
      borderRadius: BorderRadius.circular(AppTheme.radiusLg),
      child: Container(
        decoration: AppTheme.glassCard(
          borderColor: device.isConnected ? AppColors.primaryCyan.withOpacity(0.5) : AppColors.borderSubtle,
          borderWidth: device.isConnected ? 2.0 : 1.0,
        ),
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Icon(
                  device.connectionType == 'Bluetooth' ? Icons.bluetooth : Icons.usb,
                  color: device.isConnected ? AppColors.primaryCyan : AppColors.textDisabled,
                ),
                BatteryIndicator(level: device.batteryLevel, isCharging: false),
              ],
            ),
            const Spacer(),
            Text(
              device.name,
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                fontWeight: FontWeight.bold,
              ),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
            const SizedBox(height: 8),
            Text(
              device.manufacturer,
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 16),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                _buildBadge(context, '${device.buttonCount} Btn / ${device.axisCount} Ax'),
                _buildLatencyBadge(context, device.latencyMs),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildBadge(BuildContext context, String text) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: AppColors.darkBackground,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: AppColors.borderSubtle),
      ),
      child: Text(text, style: const TextStyle(fontSize: 10, color: AppColors.textSecondary)),
    );
  }

  Widget _buildLatencyBadge(BuildContext context, double latency) {
    Color color = AppColors.successGreen;
    if (latency > 5.0) color = AppColors.accentGold;
    if (latency > 15.0) color = AppColors.errorRed;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: color.withOpacity(0.5)),
      ),
      child: Text('${latency.toStringAsFixed(1)} ms', style: TextStyle(fontSize: 10, color: color, fontWeight: FontWeight.bold)),
    );
  }
}
