// lib/theme/app_theme.dart
// PUCE - PlayStation Universal Controller Emulator
// Complete dark/light theme with glassmorphism, custom typography,
// button styles, and animation constants.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:google_fonts/google_fonts.dart';
import 'app_colors.dart';

// ─── Animation Duration Constants ─────────────────────────────────────────────
class AppDurations {
  AppDurations._();
  static const Duration instant = Duration.zero;
  static const Duration veryFast = Duration(milliseconds: 100);
  static const Duration fast = Duration(milliseconds: 180);
  static const Duration normal = Duration(milliseconds: 280);
  static const Duration medium = Duration(milliseconds: 400);
  static const Duration slow = Duration(milliseconds: 600);
  static const Duration verySlow = Duration(milliseconds: 900);
  static const Duration xSlow = Duration(milliseconds: 1200);
  static const Duration pulse = Duration(milliseconds: 1800);
  static const Duration glow = Duration(seconds: 2);
}

// ─── Responsive Breakpoints ───────────────────────────────────────────────────
class AppBreakpoints {
  AppBreakpoints._();
  static const double phone = 600;
  static const double tablet = 900;
  static const double desktop = 1200;
  static const double tv = 1800;

  static bool isPhone(BuildContext ctx) =>
      MediaQuery.sizeOf(ctx).width < phone;
  static bool isTablet(BuildContext ctx) =>
      MediaQuery.sizeOf(ctx).width >= phone &&
      MediaQuery.sizeOf(ctx).width < tablet;
  static bool isDesktop(BuildContext ctx) =>
      MediaQuery.sizeOf(ctx).width >= tablet &&
      MediaQuery.sizeOf(ctx).width < tv;
  static bool isTV(BuildContext ctx) =>
      MediaQuery.sizeOf(ctx).width >= tv;
}

// ─── Glassmorphism Decoration ─────────────────────────────────────────────────
BoxDecoration glassDecoration({
  Color borderColor = AppColors.glassBorder,
  double borderWidth = 1.0,
  double borderRadius = 16,
  List<Color>? gradientColors,
  List<BoxShadow>? shadows,
}) {
  return BoxDecoration(
    gradient: LinearGradient(
      colors: gradientColors ?? [
        const Color(0x1AFFFFFF),
        const Color(0x08FFFFFF),
      ],
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
    ),
    borderRadius: BorderRadius.circular(borderRadius),
    border: Border.all(color: borderColor, width: borderWidth),
    boxShadow: shadows ?? [
      BoxShadow(
        color: AppColors.shadowDark,
        blurRadius: 20,
        offset: const Offset(0, 8),
      ),
    ],
  );
}

// ─── Glow Box Decoration ──────────────────────────────────────────────────────
BoxDecoration glowDecoration({
  required Color glowColor,
  double borderRadius = 16,
  double blurRadius = 24,
  double spreadRadius = 0,
}) {
  return BoxDecoration(
    borderRadius: BorderRadius.circular(borderRadius),
    boxShadow: [
      BoxShadow(
        color: glowColor.withOpacity(0.4),
        blurRadius: blurRadius,
        spreadRadius: spreadRadius,
        offset: Offset.zero,
      ),
      BoxShadow(
        color: glowColor.withOpacity(0.15),
        blurRadius: blurRadius * 2,
        spreadRadius: spreadRadius,
        offset: Offset.zero,
      ),
    ],
  );
}

// ─── App Theme ────────────────────────────────────────────────────────────────
class AppTheme {
  AppTheme._();

  // ─── Text Themes ──────────────────────────────────────────────────────────

  static TextTheme _buildTextTheme({required bool dark}) {
    final baseColor =
        dark ? AppColors.textPrimary : AppColors.textLight;
    final secondaryColor =
        dark ? AppColors.textSecondary : AppColors.textLightSecondary;

    return TextTheme(
      // Display — Outfit ExtraBold
      displayLarge: GoogleFonts.outfit(
        fontSize: 57,
        fontWeight: FontWeight.w800,
        color: baseColor,
        letterSpacing: -1.5,
        height: 1.1,
      ),
      displayMedium: GoogleFonts.outfit(
        fontSize: 45,
        fontWeight: FontWeight.w800,
        color: baseColor,
        letterSpacing: -0.5,
        height: 1.15,
      ),
      displaySmall: GoogleFonts.outfit(
        fontSize: 36,
        fontWeight: FontWeight.w700,
        color: baseColor,
        height: 1.2,
      ),

      // Headline — Outfit Bold
      headlineLarge: GoogleFonts.outfit(
        fontSize: 32,
        fontWeight: FontWeight.w700,
        color: baseColor,
        height: 1.25,
      ),
      headlineMedium: GoogleFonts.outfit(
        fontSize: 28,
        fontWeight: FontWeight.w700,
        color: baseColor,
        height: 1.3,
      ),
      headlineSmall: GoogleFonts.outfit(
        fontSize: 24,
        fontWeight: FontWeight.w600,
        color: baseColor,
        height: 1.35,
      ),

      // Title — Outfit SemiBold
      titleLarge: GoogleFonts.outfit(
        fontSize: 22,
        fontWeight: FontWeight.w600,
        color: baseColor,
        height: 1.35,
      ),
      titleMedium: GoogleFonts.outfit(
        fontSize: 16,
        fontWeight: FontWeight.w600,
        color: baseColor,
        letterSpacing: 0.15,
      ),
      titleSmall: GoogleFonts.outfit(
        fontSize: 14,
        fontWeight: FontWeight.w600,
        color: secondaryColor,
        letterSpacing: 0.1,
      ),

      // Body — Inter
      bodyLarge: GoogleFonts.inter(
        fontSize: 16,
        fontWeight: FontWeight.w400,
        color: baseColor,
        height: 1.5,
        letterSpacing: 0.15,
      ),
      bodyMedium: GoogleFonts.inter(
        fontSize: 14,
        fontWeight: FontWeight.w400,
        color: baseColor,
        height: 1.5,
        letterSpacing: 0.25,
      ),
      bodySmall: GoogleFonts.inter(
        fontSize: 12,
        fontWeight: FontWeight.w400,
        color: secondaryColor,
        height: 1.4,
        letterSpacing: 0.4,
      ),

      // Label — Inter Medium
      labelLarge: GoogleFonts.inter(
        fontSize: 14,
        fontWeight: FontWeight.w500,
        color: baseColor,
        letterSpacing: 0.1,
      ),
      labelMedium: GoogleFonts.inter(
        fontSize: 12,
        fontWeight: FontWeight.w500,
        color: secondaryColor,
        letterSpacing: 0.5,
      ),
      labelSmall: GoogleFonts.inter(
        fontSize: 11,
        fontWeight: FontWeight.w500,
        color: secondaryColor,
        letterSpacing: 0.5,
      ),
    );
  }

  // ─── Elevated Button Theme ────────────────────────────────────────────────
  static ElevatedButtonThemeData _elevatedButtonTheme({required bool dark}) {
    return ElevatedButtonThemeData(
      style: ButtonStyle(
        backgroundColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.disabled)) {
            return dark
                ? AppColors.surfaceDark2
                : AppColors.borderLight;
          }
          if (states.contains(WidgetState.pressed)) {
            return AppColors.primaryDark;
          }
          return AppColors.primary;
        }),
        foregroundColor: WidgetStateProperty.all(Colors.white),
        overlayColor: WidgetStateProperty.all(Colors.white.withOpacity(0.15)),
        elevation: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.pressed)) return 2;
          if (states.contains(WidgetState.hovered)) return 8;
          return 4;
        }),
        shadowColor: WidgetStateProperty.all(AppColors.shadowPrimary),
        padding: WidgetStateProperty.all(
          const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
        ),
        shape: WidgetStateProperty.all(
          RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        ),
        textStyle: WidgetStateProperty.all(
          GoogleFonts.outfit(
            fontSize: 15,
            fontWeight: FontWeight.w600,
            letterSpacing: 0.5,
          ),
        ),
        animationDuration: AppDurations.fast,
      ),
    );
  }

  // ─── Outlined Button Theme ────────────────────────────────────────────────
  static OutlinedButtonThemeData _outlinedButtonTheme({required bool dark}) {
    return OutlinedButtonThemeData(
      style: ButtonStyle(
        foregroundColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.disabled)) {
            return AppColors.textDisabled;
          }
          return AppColors.primary;
        }),
        overlayColor: WidgetStateProperty.all(AppColors.primarySubtle),
        side: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.disabled)) {
            return const BorderSide(color: AppColors.textDisabled, width: 1.5);
          }
          if (states.contains(WidgetState.focused) ||
              states.contains(WidgetState.hovered)) {
            return const BorderSide(color: AppColors.primaryLight, width: 1.5);
          }
          return const BorderSide(color: AppColors.primary, width: 1.5);
        }),
        padding: WidgetStateProperty.all(
          const EdgeInsets.symmetric(horizontal: 24, vertical: 14),
        ),
        shape: WidgetStateProperty.all(
          RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
        ),
        textStyle: WidgetStateProperty.all(
          GoogleFonts.outfit(
            fontSize: 15,
            fontWeight: FontWeight.w600,
            letterSpacing: 0.5,
          ),
        ),
        animationDuration: AppDurations.fast,
      ),
    );
  }

  // ─── Text Button Theme ────────────────────────────────────────────────────
  static TextButtonThemeData _textButtonTheme({required bool dark}) {
    return TextButtonThemeData(
      style: ButtonStyle(
        foregroundColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.disabled)) {
            return AppColors.textDisabled;
          }
          if (states.contains(WidgetState.hovered)) {
            return AppColors.primaryLight;
          }
          return AppColors.primary;
        }),
        overlayColor: WidgetStateProperty.all(AppColors.primarySubtle),
        padding: WidgetStateProperty.all(
          const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
        ),
        shape: WidgetStateProperty.all(
          RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
        ),
        textStyle: WidgetStateProperty.all(
          GoogleFonts.outfit(
            fontSize: 14,
            fontWeight: FontWeight.w600,
            letterSpacing: 0.5,
          ),
        ),
        animationDuration: AppDurations.fast,
      ),
    );
  }

  // ─── Icon Button Theme ────────────────────────────────────────────────────
  static IconButtonThemeData _iconButtonTheme({required bool dark}) {
    return IconButtonThemeData(
      style: ButtonStyle(
        foregroundColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.disabled)) {
            return AppColors.textDisabled;
          }
          if (states.contains(WidgetState.hovered)) {
            return AppColors.primaryLight;
          }
          return dark ? AppColors.textSecondary : AppColors.textLightSecondary;
        }),
        overlayColor: WidgetStateProperty.all(AppColors.primarySubtle),
        shape: WidgetStateProperty.all(
          RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
        ),
        animationDuration: AppDurations.fast,
      ),
    );
  }

  // ─── Input Decoration Theme ───────────────────────────────────────────────
  static InputDecorationTheme _inputDecorationTheme({required bool dark}) {
    final fill =
        dark ? AppColors.surfaceDark2 : AppColors.backgroundLight;
    final border = dark ? AppColors.borderDark : AppColors.borderLight;

    return InputDecorationTheme(
      filled: true,
      fillColor: fill,
      contentPadding:
          const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: BorderSide(color: border, width: 1.5),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: BorderSide(color: border, width: 1.5),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide:
            const BorderSide(color: AppColors.primary, width: 2.0),
      ),
      errorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: AppColors.error, width: 1.5),
      ),
      focusedErrorBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(12),
        borderSide: const BorderSide(color: AppColors.error, width: 2.0),
      ),
      labelStyle: GoogleFonts.inter(
        color:
            dark ? AppColors.textSecondary : AppColors.textLightSecondary,
        fontSize: 14,
        fontWeight: FontWeight.w500,
      ),
      hintStyle: GoogleFonts.inter(
        color: dark ? AppColors.textHint : AppColors.textDisabled,
        fontSize: 14,
      ),
      prefixIconColor: dark
          ? AppColors.textSecondary
          : AppColors.textLightSecondary,
      suffixIconColor: dark
          ? AppColors.textSecondary
          : AppColors.textLightSecondary,
    );
  }

  // ─── Card Theme ───────────────────────────────────────────────────────────
  static CardTheme _cardTheme({required bool dark}) {
    return CardTheme(
      color: dark ? AppColors.cardDark : AppColors.cardLight,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
        side: BorderSide(
          color: dark ? AppColors.borderDark : AppColors.borderLight,
          width: 1,
        ),
      ),
      margin: const EdgeInsets.all(0),
      clipBehavior: Clip.antiAlias,
    );
  }

  // ─── Chip Theme ───────────────────────────────────────────────────────────
  static ChipThemeData _chipTheme({required bool dark}) {
    return ChipThemeData(
      backgroundColor:
          dark ? AppColors.surfaceDark2 : AppColors.backgroundLight,
      selectedColor: AppColors.primarySubtle,
      disabledColor:
          dark ? AppColors.surfaceDark : AppColors.backgroundLight,
      labelStyle: GoogleFonts.inter(
        fontSize: 13,
        fontWeight: FontWeight.w500,
        color: dark ? AppColors.textSecondary : AppColors.textLightSecondary,
      ),
      side: BorderSide(
        color: dark ? AppColors.borderDark : AppColors.borderLight,
        width: 1,
      ),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      elevation: 0,
      pressElevation: 0,
      showCheckmark: false,
    );
  }

  // ─── Slider Theme ─────────────────────────────────────────────────────────
  static SliderThemeData _sliderTheme({required bool dark}) {
    return SliderThemeData(
      activeTrackColor: AppColors.primary,
      inactiveTrackColor:
          dark ? AppColors.surfaceDark2 : AppColors.borderLight,
      thumbColor: AppColors.primary,
      overlayColor: AppColors.primarySubtle,
      valueIndicatorColor: AppColors.primaryDark,
      trackHeight: 4,
      thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 8),
      overlayShape: const RoundSliderOverlayShape(overlayRadius: 18),
      valueIndicatorShape: const PaddleSliderValueIndicatorShape(),
      valueIndicatorTextStyle: GoogleFonts.inter(
        fontSize: 12,
        fontWeight: FontWeight.w600,
        color: Colors.white,
      ),
    );
  }

  // ─── Switch Theme ─────────────────────────────────────────────────────────
  static SwitchThemeData _switchTheme({required bool dark}) {
    return SwitchThemeData(
      thumbColor: WidgetStateProperty.resolveWith((states) {
        if (states.contains(WidgetState.selected)) return Colors.white;
        return dark ? AppColors.textDisabled : Colors.white;
      }),
      trackColor: WidgetStateProperty.resolveWith((states) {
        if (states.contains(WidgetState.selected)) return AppColors.primary;
        return dark ? AppColors.surfaceDark2 : AppColors.borderLight;
      }),
      trackOutlineColor: WidgetStateProperty.all(Colors.transparent),
    );
  }

  // ─── AppBar Theme ─────────────────────────────────────────────────────────
  static AppBarTheme _appBarTheme({required bool dark}) {
    return AppBarTheme(
      backgroundColor: Colors.transparent,
      elevation: 0,
      scrolledUnderElevation: 0,
      centerTitle: false,
      systemOverlayStyle: dark
          ? SystemUiOverlayStyle.light
          : SystemUiOverlayStyle.dark,
      titleTextStyle: GoogleFonts.outfit(
        fontSize: 20,
        fontWeight: FontWeight.w700,
        color: dark ? AppColors.textPrimary : AppColors.textLight,
      ),
      iconTheme: IconThemeData(
        color: dark ? AppColors.textSecondary : AppColors.textLightSecondary,
        size: 24,
      ),
      actionsIconTheme: IconThemeData(
        color: dark ? AppColors.textSecondary : AppColors.textLightSecondary,
        size: 24,
      ),
    );
  }

  // ─── Navigation Rail Theme ────────────────────────────────────────────────
  static NavigationRailThemeData _navRailTheme({required bool dark}) {
    return NavigationRailThemeData(
      backgroundColor:
          dark ? AppColors.surfaceDark : Colors.white,
      elevation: 0,
      selectedIconTheme: const IconThemeData(color: AppColors.primary, size: 24),
      unselectedIconTheme: IconThemeData(
        color: dark
            ? AppColors.textSecondary
            : AppColors.textLightSecondary,
        size: 24,
      ),
      selectedLabelTextStyle: GoogleFonts.inter(
        color: AppColors.primary,
        fontSize: 12,
        fontWeight: FontWeight.w600,
      ),
      unselectedLabelTextStyle: GoogleFonts.inter(
        color: dark
            ? AppColors.textSecondary
            : AppColors.textLightSecondary,
        fontSize: 12,
        fontWeight: FontWeight.w500,
      ),
      indicatorColor: AppColors.primarySubtle,
      indicatorShape:
          RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
      minWidth: 72,
      minExtendedWidth: 220,
      useIndicator: true,
      labelType: NavigationRailLabelType.all,
    );
  }

  // ─── Tab Bar Theme ────────────────────────────────────────────────────────
  static TabBarTheme _tabBarTheme({required bool dark}) {
    return TabBarTheme(
      labelColor: AppColors.primary,
      unselectedLabelColor:
          dark ? AppColors.textSecondary : AppColors.textLightSecondary,
      indicatorColor: AppColors.primary,
      indicatorSize: TabBarIndicatorSize.label,
      labelStyle: GoogleFonts.outfit(
        fontSize: 14,
        fontWeight: FontWeight.w600,
      ),
      unselectedLabelStyle: GoogleFonts.outfit(
        fontSize: 14,
        fontWeight: FontWeight.w500,
      ),
      overlayColor: WidgetStateProperty.all(AppColors.primarySubtle),
      dividerColor: Colors.transparent,
    );
  }

  // ─── Tooltip Theme ────────────────────────────────────────────────────────
  static TooltipThemeData _tooltipTheme({required bool dark}) {
    return TooltipThemeData(
      decoration: BoxDecoration(
        color: dark ? AppColors.cardDark2 : AppColors.textLight,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: dark ? AppColors.borderDark : AppColors.borderLight,
        ),
        boxShadow: [
          BoxShadow(
            color: AppColors.shadowDark,
            blurRadius: 12,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      textStyle: GoogleFonts.inter(
        fontSize: 12,
        fontWeight: FontWeight.w500,
        color: dark ? AppColors.textPrimary : Colors.white,
      ),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      waitDuration: const Duration(milliseconds: 500),
    );
  }

  // ─── Dialog Theme ─────────────────────────────────────────────────────────
  static DialogTheme _dialogTheme({required bool dark}) {
    return DialogTheme(
      backgroundColor: dark ? AppColors.surfaceDark : Colors.white,
      elevation: 24,
      shadowColor: AppColors.shadowDark,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(20),
        side: BorderSide(
          color: dark ? AppColors.borderDark : AppColors.borderLight,
          width: 1,
        ),
      ),
      titleTextStyle: GoogleFonts.outfit(
        fontSize: 20,
        fontWeight: FontWeight.w700,
        color: dark ? AppColors.textPrimary : AppColors.textLight,
      ),
      contentTextStyle: GoogleFonts.inter(
        fontSize: 14,
        color: dark ? AppColors.textSecondary : AppColors.textLightSecondary,
        height: 1.5,
      ),
    );
  }

  // ─── Scrollbar Theme ──────────────────────────────────────────────────────
  static ScrollbarThemeData _scrollbarTheme({required bool dark}) {
    return ScrollbarThemeData(
      thumbColor: WidgetStateProperty.all(
        dark ? AppColors.borderDark : AppColors.borderLight,
      ),
      trackColor: WidgetStateProperty.all(Colors.transparent),
      radius: const Radius.circular(4),
      thickness: WidgetStateProperty.all(4),
    );
  }

  // ─── Dark Theme ───────────────────────────────────────────────────────────
  static ThemeData get dark {
    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      colorScheme: const ColorScheme.dark(
        brightness: Brightness.dark,
        primary: AppColors.primary,
        onPrimary: Colors.white,
        primaryContainer: AppColors.primarySubtle,
        onPrimaryContainer: AppColors.primaryLight,
        secondary: AppColors.secondary,
        onSecondary: Colors.black,
        secondaryContainer: AppColors.secondarySubtle,
        onSecondaryContainer: AppColors.secondaryLight,
        tertiary: AppColors.accent,
        onTertiary: Colors.white,
        tertiaryContainer: AppColors.accentSubtle,
        onTertiaryContainer: AppColors.accentLight,
        error: AppColors.error,
        onError: Colors.white,
        errorContainer: AppColors.errorSubtle,
        onErrorContainer: AppColors.error,
        surface: AppColors.surfaceDark,
        onSurface: AppColors.textPrimary,
        onSurfaceVariant: AppColors.textSecondary,
        surfaceContainerHighest: AppColors.cardDark2,
        outline: AppColors.borderDark,
        outlineVariant: AppColors.borderDarkSubtle,
        shadow: AppColors.shadowDark,
        scrim: AppColors.scrimDark,
        inverseSurface: AppColors.textPrimary,
        onInverseSurface: AppColors.backgroundDark,
        inversePrimary: AppColors.primaryDarker,
      ),
      scaffoldBackgroundColor: AppColors.backgroundDark,
      textTheme: _buildTextTheme(dark: true),
      primaryTextTheme: _buildTextTheme(dark: true),
      appBarTheme: _appBarTheme(dark: true),
      cardTheme: _cardTheme(dark: true),
      chipTheme: _chipTheme(dark: true),
      elevatedButtonTheme: _elevatedButtonTheme(dark: true),
      outlinedButtonTheme: _outlinedButtonTheme(dark: true),
      textButtonTheme: _textButtonTheme(dark: true),
      iconButtonTheme: _iconButtonTheme(dark: true),
      inputDecorationTheme: _inputDecorationTheme(dark: true),
      sliderTheme: _sliderTheme(dark: true),
      switchTheme: _switchTheme(dark: true),
      navigationRailTheme: _navRailTheme(dark: true),
      tabBarTheme: _tabBarTheme(dark: true),
      tooltipTheme: _tooltipTheme(dark: true),
      dialogTheme: _dialogTheme(dark: true),
      scrollbarTheme: _scrollbarTheme(dark: true),
      dividerTheme: const DividerThemeData(
        color: AppColors.dividerDark,
        thickness: 1,
        space: 1,
      ),
      iconTheme: const IconThemeData(
        color: AppColors.textSecondary,
        size: 24,
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: AppColors.cardDark2,
        contentTextStyle: GoogleFonts.inter(
          color: AppColors.textPrimary,
          fontSize: 14,
        ),
        behavior: SnackBarBehavior.floating,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: const BorderSide(color: AppColors.borderDark),
        ),
        elevation: 8,
      ),
      listTileTheme: const ListTileThemeData(
        iconColor: AppColors.textSecondary,
        textColor: AppColors.textPrimary,
        contentPadding: EdgeInsets.symmetric(horizontal: 20, vertical: 4),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.all(Radius.circular(12)),
        ),
      ),
      expansionTileTheme: const ExpansionTileThemeData(
        iconColor: AppColors.textSecondary,
        textColor: AppColors.textPrimary,
        collapsedIconColor: AppColors.textSecondary,
        collapsedTextColor: AppColors.textPrimary,
        backgroundColor: Colors.transparent,
        collapsedBackgroundColor: Colors.transparent,
      ),
      progressIndicatorTheme: const ProgressIndicatorThemeData(
        color: AppColors.primary,
        linearTrackColor: AppColors.surfaceDark2,
        circularTrackColor: AppColors.surfaceDark2,
      ),
      floatingActionButtonTheme: FloatingActionButtonThemeData(
        backgroundColor: AppColors.primary,
        foregroundColor: Colors.white,
        elevation: 8,
        focusElevation: 12,
        hoverElevation: 12,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      ),
      popupMenuTheme: PopupMenuThemeData(
        color: AppColors.cardDark,
        elevation: 12,
        shadowColor: AppColors.shadowDark,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: const BorderSide(color: AppColors.borderDark),
        ),
        textStyle: GoogleFonts.inter(
          fontSize: 14,
          color: AppColors.textPrimary,
        ),
      ),
    );
  }

  // ─── Light Theme ──────────────────────────────────────────────────────────
  static ThemeData get light {
    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.light,
      colorScheme: const ColorScheme.light(
        brightness: Brightness.light,
        primary: AppColors.primaryDarker,
        onPrimary: Colors.white,
        primaryContainer: AppColors.primarySubtle,
        onPrimaryContainer: AppColors.primaryDarker,
        secondary: AppColors.secondaryDark,
        onSecondary: Colors.white,
        secondaryContainer: AppColors.secondarySubtle,
        onSecondaryContainer: AppColors.secondaryDark,
        tertiary: AppColors.accent,
        onTertiary: Colors.white,
        tertiaryContainer: AppColors.accentSubtle,
        onTertiaryContainer: AppColors.accentDark,
        error: AppColors.errorDark,
        onError: Colors.white,
        surface: AppColors.surfaceLight,
        onSurface: AppColors.textLight,
        onSurfaceVariant: AppColors.textLightSecondary,
        outline: AppColors.borderLight,
        shadow: AppColors.shadowDark,
      ),
      scaffoldBackgroundColor: AppColors.backgroundLight,
      textTheme: _buildTextTheme(dark: false),
      primaryTextTheme: _buildTextTheme(dark: false),
      appBarTheme: _appBarTheme(dark: false),
      cardTheme: _cardTheme(dark: false),
      chipTheme: _chipTheme(dark: false),
      elevatedButtonTheme: _elevatedButtonTheme(dark: false),
      outlinedButtonTheme: _outlinedButtonTheme(dark: false),
      textButtonTheme: _textButtonTheme(dark: false),
      iconButtonTheme: _iconButtonTheme(dark: false),
      inputDecorationTheme: _inputDecorationTheme(dark: false),
      sliderTheme: _sliderTheme(dark: false),
      switchTheme: _switchTheme(dark: false),
      navigationRailTheme: _navRailTheme(dark: false),
      tabBarTheme: _tabBarTheme(dark: false),
      tooltipTheme: _tooltipTheme(dark: false),
      dialogTheme: _dialogTheme(dark: false),
      scrollbarTheme: _scrollbarTheme(dark: false),
      dividerTheme: const DividerThemeData(
        color: AppColors.borderLight,
        thickness: 1,
        space: 1,
      ),
      iconTheme: const IconThemeData(
        color: AppColors.textLightSecondary,
        size: 24,
      ),
      floatingActionButtonTheme: FloatingActionButtonThemeData(
        backgroundColor: AppColors.primary,
        foregroundColor: Colors.white,
        elevation: 8,
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      ),
    );
  }
}
