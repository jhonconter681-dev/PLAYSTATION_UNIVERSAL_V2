// ─────────────────────────────────────────────────────────────
// PUCE Widgets — Battery Indicator
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import '../theme/app_colors.dart';

class BatteryIndicator extends StatelessWidget {
  final double level; // 0.0 to 1.0
  final bool isCharging;

  const BatteryIndicator({
    super.key,
    required this.level,
    this.isCharging = false,
  });

  @override
  Widget build(BuildContext context) {
    Color color = AppColors.successGreen;
    if (level < 0.2) color = AppColors.errorRed;
    else if (level < 0.5) color = AppColors.accentGold;

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (isCharging)
          const Icon(Icons.bolt, color: AppColors.accentGold, size: 16),
        const SizedBox(width: 4),
        Container(
          width: 24,
          height: 12,
          decoration: BoxDecoration(
            border: Border.all(color: AppColors.textSecondary),
            borderRadius: BorderRadius.circular(2),
          ),
          child: Row(
            children: [
              Container(
                width: 22 * level,
                height: 10,
                color: color,
              ),
            ],
          ),
        ),
        Container(
          width: 2,
          height: 4,
          decoration: const BoxDecoration(
            color: AppColors.textSecondary,
            borderRadius: BorderRadius.horizontal(right: Radius.circular(2)),
          ),
        ),
        const SizedBox(width: 8),
        Text('${(level * 100).toInt()}%', style: const TextStyle(fontSize: 10, color: AppColors.textSecondary)),
      ],
    );
  }
}
