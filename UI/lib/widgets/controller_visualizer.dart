// ─────────────────────────────────────────────────────────────
// PUCE Widgets — Controller Visualizer
// ─────────────────────────────────────────────────────────────

import 'package:flutter/material.dart';
import '../theme/app_colors.dart';

class ControllerVisualizer extends StatelessWidget {
  const ControllerVisualizer({super.key});

  @override
  Widget build(BuildContext context) {
    // In a full implementation, this would be a complex CustomPainter or SVG
    // rendering the shape of a DualSense/DS4 with interactive glowing buttons.
    // For this prototype, we'll draw a schematic representation using basic shapes.
    return SizedBox(
      width: 400,
      height: 300,
      child: CustomPaint(
        painter: _ControllerPainter(),
      ),
    );
  }
}

class _ControllerPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = AppColors.darkCard
      ..style = PaintingStyle.fill;
      
    final borderPaint = Paint()
      ..color = AppColors.borderSubtle
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2;

    // Base body
    final path = Path();
    path.moveTo(size.width * 0.2, size.height * 0.3);
    path.quadraticBezierTo(size.width * 0.5, size.height * 0.2, size.width * 0.8, size.height * 0.3);
    path.quadraticBezierTo(size.width * 0.9, size.height * 0.5, size.width * 0.85, size.height * 0.8);
    path.quadraticBezierTo(size.width * 0.7, size.height * 0.6, size.width * 0.5, size.height * 0.6);
    path.quadraticBezierTo(size.width * 0.3, size.height * 0.6, size.width * 0.15, size.height * 0.8);
    path.quadraticBezierTo(size.width * 0.1, size.height * 0.5, size.width * 0.2, size.height * 0.3);
    
    canvas.drawPath(path, paint);
    canvas.drawPath(path, borderPaint);

    // D-Pad
    final dpadPaint = Paint()..color = AppColors.textSecondary;
    canvas.drawRect(Rect.fromLTWH(size.width * 0.25, size.height * 0.4, 15, 45), dpadPaint);
    canvas.drawRect(Rect.fromLTWH(size.width * 0.25 - 15, size.height * 0.4 + 15, 45, 15), dpadPaint);

    // Action Buttons
    final btnPaint = Paint()..color = AppColors.textSecondary;
    canvas.drawCircle(Offset(size.width * 0.75, size.height * 0.4 + 22.5), 8, btnPaint); // Cross
    canvas.drawCircle(Offset(size.width * 0.75 + 20, size.height * 0.4 + 5), 8, btnPaint); // Circle
    canvas.drawCircle(Offset(size.width * 0.75 - 20, size.height * 0.4 + 5), 8, btnPaint); // Square
    canvas.drawCircle(Offset(size.width * 0.75, size.height * 0.4 - 12.5), 8, btnPaint); // Triangle

    // Analog Sticks
    final stickPaint = Paint()..color = AppColors.borderSubtle..style = PaintingStyle.stroke..strokeWidth = 4;
    canvas.drawCircle(Offset(size.width * 0.35, size.height * 0.65), 20, paint);
    canvas.drawCircle(Offset(size.width * 0.35, size.height * 0.65), 20, stickPaint);
    canvas.drawCircle(Offset(size.width * 0.65, size.height * 0.65), 20, paint);
    canvas.drawCircle(Offset(size.width * 0.65, size.height * 0.65), 20, stickPaint);

    // Touchpad
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(size.width * 0.4, size.height * 0.3, size.width * 0.2, size.height * 0.15),
        const Radius.circular(8)
      ),
      paint
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(size.width * 0.4, size.height * 0.3, size.width * 0.2, size.height * 0.15),
        const Radius.circular(8)
      ),
      borderPaint
    );
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
