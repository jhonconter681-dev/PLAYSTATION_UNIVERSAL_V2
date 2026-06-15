// ─────────────────────────────────────────────────────────────
// PUCE Widgets — Connection Badge
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../providers/device_provider.dart';
import '../theme/app_colors.dart';

class ConnectionBadge extends ConsumerWidget {
  const ConnectionBadge({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final statusAsync = ref.watch(systemStatusProvider);
    
    return statusAsync.when(
      data: (status) {
        final isRunning = status['running'] == true;
        
        return Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            color: isRunning ? AppColors.successGreen.withOpacity(0.1) : AppColors.errorRed.withOpacity(0.1),
            borderRadius: BorderRadius.circular(16),
            border: Border.all(color: isRunning ? AppColors.successGreen : AppColors.errorRed),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Container(
                width: 8,
                height: 8,
                decoration: BoxDecoration(
                  color: isRunning ? AppColors.successGreen : AppColors.errorRed,
                  shape: BoxShape.circle,
                ),
              ).animate(onPlay: (controller) => controller.repeat())
               .fade(duration: const Duration(seconds: 1)),
              const SizedBox(width: 8),
              Text(
                isRunning ? 'ENGINE ONLINE' : 'ENGINE OFFLINE',
                style: TextStyle(
                  color: isRunning ? AppColors.successGreen : AppColors.errorRed,
                  fontSize: 10,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ],
          ),
        );
      },
      loading: () => const SizedBox(),
      error: (_, __) => const SizedBox(),
    );
  }
}
