// ─────────────────────────────────────────────────────────────
// PUCE Profiles Screen
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../providers/mapping_provider.dart';
import '../theme/app_colors.dart';
import '../theme/app_theme.dart';

class ProfilesScreen extends ConsumerWidget {
  const ProfilesScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final profiles = ref.watch(savedProfilesProvider);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
        title: const Text('Saved Profiles'),
        actions: [
          IconButton(
            icon: const Icon(Icons.file_upload),
            tooltip: 'Import Profile',
            onPressed: () {},
          ),
          const SizedBox(width: 16),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () {},
        icon: const Icon(Icons.add),
        label: const Text('New Profile'),
        backgroundColor: AppColors.primaryCyan,
      ),
      body: profiles.isEmpty
          ? const Center(child: Text('No profiles saved.'))
          : GridView.builder(
              padding: const EdgeInsets.all(32),
              gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                crossAxisCount: 3,
                crossAxisSpacing: 24,
                mainAxisSpacing: 24,
                childAspectRatio: 1.5,
              ),
              itemCount: profiles.length,
              itemBuilder: (context, index) {
                final p = profiles[index];
                return Container(
                  decoration: AppTheme.glassCard(),
                  padding: const EdgeInsets.all(20),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Icon(Icons.gamepad, color: AppColors.textPrimary, size: 32),
                          if (p.isDefault)
                            Container(
                              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                              decoration: BoxDecoration(
                                color: AppColors.primaryCyan.withOpacity(0.2),
                                borderRadius: BorderRadius.circular(4),
                              ),
                              child: const Text('ACTIVE', style: TextStyle(color: AppColors.primaryCyan, fontSize: 10, fontWeight: FontWeight.bold)),
                            ),
                        ],
                      ),
                      const Spacer(),
                      Text(p.name, style: Theme.of(context).textTheme.titleLarge),
                      const SizedBox(height: 4),
                      Text('Mode: ${p.psMode}', style: Theme.of(context).textTheme.bodySmall),
                      const SizedBox(height: 16),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.end,
                        children: [
                          IconButton(icon: const Icon(Icons.edit, size: 20), onPressed: () {}),
                          IconButton(icon: const Icon(Icons.share, size: 20), onPressed: () {}),
                          IconButton(icon: const Icon(Icons.delete, size: 20, color: AppColors.errorRed), onPressed: () {}),
                        ],
                      )
                    ],
                  ),
                );
              },
            ),
    );
  }
}
