// ─────────────────────────────────────────────────────────────
// PUCE Widgets — Mapping Row
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import '../theme/app_colors.dart';

class MappingRow extends StatelessWidget {
  final String sourceName;
  final String targetName;
  final bool isAxis;

  const MappingRow({
    super.key,
    required this.sourceName,
    required this.targetName,
    this.isAxis = false,
  });

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12.0),
      child: Row(
        children: [
          Expanded(
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              decoration: BoxDecoration(
                color: AppColors.darkCard,
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: AppColors.borderSubtle),
              ),
              child: Text(sourceName, style: const TextStyle(color: AppColors.textSecondary)),
            ),
          ),
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16.0),
            child: Icon(Icons.arrow_forward, color: AppColors.primaryCyan),
          ),
          Expanded(
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
              decoration: BoxDecoration(
                color: AppColors.primaryCyan.withOpacity(0.1),
                borderRadius: BorderRadius.circular(8),
                border: Border.all(color: AppColors.primaryCyan),
              ),
              child: Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Text(targetName, style: const TextStyle(fontWeight: FontWeight.bold)),
                  const Icon(Icons.arrow_drop_down, color: AppColors.primaryCyan),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
