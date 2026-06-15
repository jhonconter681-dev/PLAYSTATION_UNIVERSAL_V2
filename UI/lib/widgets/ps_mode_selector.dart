// ─────────────────────────────────────────────────────────────
// PUCE Widgets — PS Mode Selector
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import '../models/emulation_mode.dart';
import '../theme/app_colors.dart';
import '../theme/app_theme.dart';

class PSModeSelector extends StatelessWidget {
  final EmulationMode selectedMode;
  final ValueChanged<EmulationMode> onModeSelected;

  const PSModeSelector({
    super.key,
    required this.selectedMode,
    required this.onModeSelected,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 120,
      child: ListView.separated(
        scrollDirection: Axis.horizontal,
        itemCount: EmulationMode.values.length,
        separatorBuilder: (context, index) => const SizedBox(width: 16),
        itemBuilder: (context, index) {
          final mode = EmulationMode.values[index];
          final isSelected = mode == selectedMode;

          return GestureDetector(
            onTap: () => onModeSelected(mode),
            child: AnimatedContainer(
              duration: AppTheme.fastAnim,
              width: 140,
              padding: const EdgeInsets.all(16),
              decoration: isSelected
                  ? AppTheme.glowBorder(color: mode.color)
                  : AppTheme.glassCard(borderColor: AppColors.borderSubtle),
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(
                    mode.icon,
                    size: 36,
                    color: isSelected ? mode.color : AppColors.textSecondary,
                  ),
                  const SizedBox(height: 12),
                  Text(
                    mode.displayName,
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      color: isSelected ? AppColors.textPrimary : AppColors.textSecondary,
                      fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                      fontSize: 12,
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}
