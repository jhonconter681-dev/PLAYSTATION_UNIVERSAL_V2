// ─────────────────────────────────────────────────────────────
// PUCE Widgets — Latency Chart
// ─────────────────────────────────────────────────────────────

import 'dart:math';
import 'package:flutter/material.dart';
import 'package:fl_chart/fl_chart.dart';
import '../theme/app_colors.dart';

class LatencyChart extends StatefulWidget {
  const LatencyChart({super.key});

  @override
  State<LatencyChart> createState() => _LatencyChartState();
}

class _LatencyChartState extends State<LatencyChart> {
  final List<FlSpot> _spots = [];
  int _counter = 0;
  final Random _rnd = Random();

  @override
  void initState() {
    super.initState();
    // Simulate initial data
    for (int i = 0; i < 60; i++) {
      _spots.add(FlSpot(i.toDouble(), 1.0 + _rnd.nextDouble() * 2.0));
    }
    _counter = 60;
    
    // Simulate real-time updates
    _simulateUpdates();
  }

  void _simulateUpdates() async {
    while (mounted) {
      await Future.delayed(const Duration(milliseconds: 100));
      if (!mounted) break;
      
      setState(() {
        _spots.removeAt(0);
        // Random latency between 0.8ms and 3.5ms (simulating a good connection)
        double newLatency = 0.8 + _rnd.nextDouble() * 2.7;
        
        // Occasional spike
        if (_rnd.nextDouble() > 0.95) {
          newLatency += 5.0 + _rnd.nextDouble() * 10.0;
        }

        _spots.add(FlSpot(_counter.toDouble(), newLatency));
        _counter++;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return LineChart(
      LineChartData(
        minY: 0,
        maxY: 20,
        minX: _spots.first.x,
        maxX: _spots.last.x,
        gridData: FlGridData(
          show: true,
          drawVerticalLine: false,
          horizontalInterval: 5,
          getDrawingHorizontalLine: (value) {
            return FlLine(
              color: AppColors.borderSubtle.withOpacity(0.5),
              strokeWidth: 1,
            );
          },
        ),
        titlesData: FlTitlesData(
          show: true,
          rightTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
          topTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
          bottomTitles: const AxisTitles(sideTitles: SideTitles(showTitles: false)),
          leftTitles: AxisTitles(
            sideTitles: SideTitles(
              showTitles: true,
              interval: 5,
              reservedSize: 30,
              getTitlesWidget: (value, meta) {
                return Text(
                  '${value.toInt()}ms',
                  style: const TextStyle(color: AppColors.textSecondary, fontSize: 10),
                );
              },
            ),
          ),
        ),
        borderData: FlBorderData(show: false),
        lineBarsData: [
          LineChartBarData(
            spots: _spots,
            isCurved: true,
            color: AppColors.primaryCyan,
            barWidth: 2,
            isStrokeCapRound: true,
            dotData: const FlDotData(show: false),
            belowBarData: BarAreaData(
              show: true,
              gradient: LinearGradient(
                colors: [
                  AppColors.primaryCyan.withOpacity(0.3),
                  AppColors.primaryCyan.withOpacity(0.0),
                ],
                begin: Alignment.topCenter,
                end: Alignment.bottomCenter,
              ),
            ),
          ),
        ],
        extraLinesData: ExtraLinesData(
          horizontalLines: [
            HorizontalLine(
              y: 5.0,
              color: AppColors.successGreen.withOpacity(0.5),
              strokeWidth: 1,
              dashArray: [5, 5],
            ),
            HorizontalLine(
              y: 15.0,
              color: AppColors.errorRed.withOpacity(0.5),
              strokeWidth: 1,
              dashArray: [5, 5],
            ),
          ],
        ),
      ),
    );
  }
}
