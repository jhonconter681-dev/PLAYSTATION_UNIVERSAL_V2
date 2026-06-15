// ─────────────────────────────────────────────────────────────
// PUCE Settings Screen
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../providers/settings_provider.dart';
import '../theme/app_colors.dart';
import '../theme/app_theme.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(settingsProvider);
    final notifier = ref.read(settingsProvider.notifier);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
        title: const Text('Settings'),
      ),
      body: ListView(
        padding: const EdgeInsets.all(32),
        children: [
          _buildSectionTitle('Appearance', context),
          _buildSettingsCard([
            SwitchListTile(
              title: const Text('Dark Mode'),
              subtitle: const Text('Use PlayStation-inspired dark theme'),
              value: settings.darkMode,
              onChanged: (val) => notifier.toggleTheme(),
            ),
          ]),
          const SizedBox(height: 32),

          _buildSectionTitle('Performance & Emulation', context),
          _buildSettingsCard([
            ListTile(
              title: const Text('Polling Rate'),
              subtitle: Text('${settings.pollingRateHz} Hz'),
              trailing: DropdownButton<int>(
                value: settings.pollingRateHz,
                items: [125, 250, 500, 1000].map((e) => DropdownMenuItem(value: e, child: Text('$e Hz'))).toList(),
                onChanged: (val) {
                  if (val != null) notifier.updateSettings(settings.copyWith(pollingRateHz: val));
                },
              ),
            ),
            const Divider(),
            SwitchListTile(
              title: const Text('AI Drift Correction'),
              subtitle: const Text('Automatically detect and correct stick drift in real-time'),
              value: settings.aiDriftCorrection,
              onChanged: (val) => notifier.updateSettings(settings.copyWith(aiDriftCorrection: val)),
            ),
            const Divider(),
            SwitchListTile(
              title: const Text('AI Auto-Map'),
              subtitle: const Text('Automatically map new generic controllers using heuristics'),
              value: settings.aiAutoMap,
              onChanged: (val) => notifier.updateSettings(settings.copyWith(aiAutoMap: val)),
            ),
          ]),
          const SizedBox(height: 32),

          _buildSectionTitle('Security & Plugins', context),
          _buildSettingsCard([
            SwitchListTile(
              title: const Text('Require Signed Plugins'),
              subtitle: const Text('Only load plugins verified by the PUCE team (Recommended)'),
              value: settings.requireSignedPlugins,
              onChanged: (val) => notifier.updateSettings(settings.copyWith(requireSignedPlugins: val)),
            ),
            const Divider(),
            ListTile(
              title: const Text('Manage Plugins'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {},
            ),
          ]),
        ],
      ),
    );
  }

  Widget _buildSectionTitle(String title, BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 16.0, left: 8.0),
      child: Text(
        title,
        style: Theme.of(context).textTheme.titleLarge?.copyWith(color: AppColors.primaryCyan),
      ),
    );
  }

  Widget _buildSettingsCard(List<Widget> children) {
    return Container(
      decoration: AppTheme.glassCard(),
      child: Column(children: children),
    );
  }
}
