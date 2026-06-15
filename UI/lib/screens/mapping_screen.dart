// ─────────────────────────────────────────────────────────────
// PUCE Mapping Screen
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../theme/app_colors.dart';
import '../theme/app_theme.dart';
import '../widgets/controller_visualizer.dart';
import '../widgets/mapping_row.dart';

class MappingScreen extends ConsumerWidget {
  const MappingScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.pop(),
        ),
        title: const Text('Input Mapping & AI'),
        actions: [
          TextButton.icon(
            icon: const Icon(Icons.auto_fix_high),
            label: const Text('AI Auto-Map'),
            onPressed: () {
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(content: Text('AI Auto-Map applied based on heuristics.'))
              );
            },
          ),
          const SizedBox(width: 8),
          ElevatedButton.icon(
            icon: const Icon(Icons.save),
            label: const Text('Save Profile'),
            onPressed: () {},
          ),
          const SizedBox(width: 16),
        ],
      ),
      body: Row(
        children: [
          // Left: Visualizer
          Expanded(
            flex: 1,
            child: Container(
              padding: const EdgeInsets.all(32),
              decoration: BoxDecoration(
                border: Border(right: BorderSide(color: AppColors.borderSubtle)),
              ),
              child: Column(
                children: [
                  const Text('Live Preview', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold)),
                  const SizedBox(height: 32),
                  const Expanded(
                    child: Center(
                      child: ControllerVisualizer(),
                    ),
                  ),
                ],
              ),
            ),
          ),

          // Right: Editor
          Expanded(
            flex: 1,
            child: ListView(
              padding: const EdgeInsets.all(32),
              children: [
                _buildSectionHeader('Buttons'),
                const MappingRow(sourceName: 'Button A', targetName: 'Cross', isAxis: false),
                const MappingRow(sourceName: 'Button B', targetName: 'Circle', isAxis: false),
                const MappingRow(sourceName: 'Button X', targetName: 'Square', isAxis: false),
                const MappingRow(sourceName: 'Button Y', targetName: 'Triangle', isAxis: false),
                const SizedBox(height: 32),

                _buildSectionHeader('Analog Sticks & AI Calibration'),
                Container(
                  padding: const EdgeInsets.all(16),
                  decoration: AppTheme.glassCard(),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          const Text('Left Stick Deadzone'),
                          Text('8%', style: TextStyle(color: AppColors.primaryCyan)),
                        ],
                      ),
                      Slider(value: 0.08, min: 0.0, max: 0.3, onChanged: (v) {}),
                      const SizedBox(height: 16),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          const Text('Response Curve'),
                          DropdownButton<String>(
                            value: 'S-Curve',
                            dropdownColor: AppColors.darkCard,
                            underline: const SizedBox(),
                            items: ['Linear', 'Ease-In', 'Ease-Out', 'S-Curve']
                                .map((e) => DropdownMenuItem(value: e, child: Text(e)))
                                .toList(),
                            onChanged: (v) {},
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 32),

                _buildSectionHeader('Gyroscope / Motion'),
                const MappingRow(sourceName: 'Mouse Movement', targetName: 'Right Stick', isAxis: true),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildSectionHeader(String title) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 16.0),
      child: Text(
        title,
        style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: AppColors.accentGold),
      ),
    );
  }
}
