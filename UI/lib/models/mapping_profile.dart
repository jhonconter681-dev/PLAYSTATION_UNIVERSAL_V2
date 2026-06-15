// lib/models/mapping_profile.dart
// PUCE - PlayStation Universal Controller Emulator
// Models for mapping profiles, button mappings, and axis mappings

import 'dart:convert';

/// Maps a single source button to a PlayStation target button.
class ButtonMapping {
  final String sourceButton;   // e.g. 'BTN_A', 'BTN_0'
  final String targetButton;   // e.g. 'CROSS', 'CIRCLE', 'SQUARE', 'TRIANGLE'
  final bool isEnabled;
  final double? turboHz;       // null = no turbo, else fire rate in Hz
  final bool isMacro;          // Is this part of a macro sequence?
  final List<String>? macroSequence;

  const ButtonMapping({
    required this.sourceButton,
    required this.targetButton,
    this.isEnabled = true,
    this.turboHz,
    this.isMacro = false,
    this.macroSequence,
  });

  factory ButtonMapping.fromJson(Map<String, dynamic> json) => ButtonMapping(
        sourceButton: json['source_button'] as String,
        targetButton: json['target_button'] as String,
        isEnabled: json['is_enabled'] as bool? ?? true,
        turboHz: (json['turbo_hz'] as num?)?.toDouble(),
        isMacro: json['is_macro'] as bool? ?? false,
        macroSequence: (json['macro_sequence'] as List<dynamic>?)
            ?.map((e) => e.toString())
            .toList(),
      );

  Map<String, dynamic> toJson() => {
        'source_button': sourceButton,
        'target_button': targetButton,
        'is_enabled': isEnabled,
        if (turboHz != null) 'turbo_hz': turboHz,
        'is_macro': isMacro,
        if (macroSequence != null) 'macro_sequence': macroSequence,
      };

  ButtonMapping copyWith({
    String? sourceButton,
    String? targetButton,
    bool? isEnabled,
    double? turboHz,
    bool? isMacro,
    List<String>? macroSequence,
  }) =>
      ButtonMapping(
        sourceButton: sourceButton ?? this.sourceButton,
        targetButton: targetButton ?? this.targetButton,
        isEnabled: isEnabled ?? this.isEnabled,
        turboHz: turboHz ?? this.turboHz,
        isMacro: isMacro ?? this.isMacro,
        macroSequence: macroSequence ?? this.macroSequence,
      );
}

/// Describes how an analog axis should be mapped and processed.
class AxisMapping {
  final String sourceAxis;     // e.g. 'ABS_X', 'ABS_RZ'
  final String targetAxis;     // e.g. 'LX', 'LY', 'RX', 'RY', 'L2', 'R2'
  final double deadzone;       // 0.0 – 1.0, default 0.1
  final double sensitivity;    // multiplier, default 1.0
  final bool isInverted;
  final AxisCurve curve;
  final double minValue;       // mapped output min
  final double maxValue;       // mapped output max
  final bool isEnabled;

  const AxisMapping({
    required this.sourceAxis,
    required this.targetAxis,
    this.deadzone = 0.1,
    this.sensitivity = 1.0,
    this.isInverted = false,
    this.curve = AxisCurve.linear,
    this.minValue = -1.0,
    this.maxValue = 1.0,
    this.isEnabled = true,
  });

  factory AxisMapping.fromJson(Map<String, dynamic> json) => AxisMapping(
        sourceAxis: json['source_axis'] as String,
        targetAxis: json['target_axis'] as String,
        deadzone: (json['deadzone'] as num?)?.toDouble() ?? 0.1,
        sensitivity: (json['sensitivity'] as num?)?.toDouble() ?? 1.0,
        isInverted: json['is_inverted'] as bool? ?? false,
        curve: AxisCurve.values.firstWhere(
          (c) => c.name == (json['curve'] as String?),
          orElse: () => AxisCurve.linear,
        ),
        minValue: (json['min_value'] as num?)?.toDouble() ?? -1.0,
        maxValue: (json['max_value'] as num?)?.toDouble() ?? 1.0,
        isEnabled: json['is_enabled'] as bool? ?? true,
      );

  Map<String, dynamic> toJson() => {
        'source_axis': sourceAxis,
        'target_axis': targetAxis,
        'deadzone': deadzone,
        'sensitivity': sensitivity,
        'is_inverted': isInverted,
        'curve': curve.name,
        'min_value': minValue,
        'max_value': maxValue,
        'is_enabled': isEnabled,
      };

  AxisMapping copyWith({
    String? sourceAxis,
    String? targetAxis,
    double? deadzone,
    double? sensitivity,
    bool? isInverted,
    AxisCurve? curve,
    double? minValue,
    double? maxValue,
    bool? isEnabled,
  }) =>
      AxisMapping(
        sourceAxis: sourceAxis ?? this.sourceAxis,
        targetAxis: targetAxis ?? this.targetAxis,
        deadzone: deadzone ?? this.deadzone,
        sensitivity: sensitivity ?? this.sensitivity,
        isInverted: isInverted ?? this.isInverted,
        curve: curve ?? this.curve,
        minValue: minValue ?? this.minValue,
        maxValue: maxValue ?? this.maxValue,
        isEnabled: isEnabled ?? this.isEnabled,
      );
}

/// Axis response curve shapes
enum AxisCurve {
  linear,
  quadratic,
  cubic,
  exponential,
  sCurve,
}

extension AxisCurveExtension on AxisCurve {
  String get displayName {
    switch (this) {
      case AxisCurve.linear:
        return 'Linear';
      case AxisCurve.quadratic:
        return 'Quadratic';
      case AxisCurve.cubic:
        return 'Cubic';
      case AxisCurve.exponential:
        return 'Exponential';
      case AxisCurve.sCurve:
        return 'S-Curve';
    }
  }

  String get description {
    switch (this) {
      case AxisCurve.linear:
        return 'Direct 1:1 mapping';
      case AxisCurve.quadratic:
        return 'Finer control near center';
      case AxisCurve.cubic:
        return 'Very fine control near center';
      case AxisCurve.exponential:
        return 'Aggressive response at extremes';
      case AxisCurve.sCurve:
        return 'Smooth S-shape response';
    }
  }

  /// Apply the curve to an input value in range [-1, 1]
  double apply(double x) {
    switch (this) {
      case AxisCurve.linear:
        return x;
      case AxisCurve.quadratic:
        return x.sign * x * x;
      case AxisCurve.cubic:
        return x * x * x;
      case AxisCurve.exponential:
        return x.sign * (x.abs() > 0 ? (x.abs() * 1.5).clamp(0.0, 1.0) : 0);
      case AxisCurve.sCurve:
        // Smooth step
        final t = x.abs();
        final smooth = t * t * (3 - 2 * t);
        return x.sign * smooth;
    }
  }
}

/// Complete mapping profile for a device + PS mode combination.
class MappingProfile {
  final String id;
  final String name;
  final String description;
  final String? gameName;
  final String? gameIconPath;
  final String deviceType;       // 'xbox', 'nintendo', 'generic', etc.
  final String emulationMode;    // EmulationMode.apiId
  final List<ButtonMapping> buttonMappings;
  final List<AxisMapping> axisMappings;
  final DateTime createdAt;
  final DateTime updatedAt;
  final bool isDefault;
  final String? author;
  final String version;

  const MappingProfile({
    required this.id,
    required this.name,
    this.description = '',
    this.gameName,
    this.gameIconPath,
    required this.deviceType,
    required this.emulationMode,
    required this.buttonMappings,
    required this.axisMappings,
    required this.createdAt,
    required this.updatedAt,
    this.isDefault = false,
    this.author,
    this.version = '1.0.0',
  });

  factory MappingProfile.fromJson(Map<String, dynamic> json) {
    return MappingProfile(
      id: json['id'] as String,
      name: json['name'] as String,
      description: json['description'] as String? ?? '',
      gameName: json['game_name'] as String?,
      gameIconPath: json['game_icon_path'] as String?,
      deviceType: json['device_type'] as String? ?? 'generic',
      emulationMode: json['emulation_mode'] as String? ?? 'ps4',
      buttonMappings: (json['button_mappings'] as List<dynamic>?)
              ?.map((e) =>
                  ButtonMapping.fromJson(e as Map<String, dynamic>))
              .toList() ??
          [],
      axisMappings: (json['axis_mappings'] as List<dynamic>?)
              ?.map((e) =>
                  AxisMapping.fromJson(e as Map<String, dynamic>))
              .toList() ??
          [],
      createdAt: DateTime.tryParse(json['created_at'] as String? ?? '') ??
          DateTime.now(),
      updatedAt: DateTime.tryParse(json['updated_at'] as String? ?? '') ??
          DateTime.now(),
      isDefault: json['is_default'] as bool? ?? false,
      author: json['author'] as String?,
      version: json['version'] as String? ?? '1.0.0',
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'name': name,
        'description': description,
        if (gameName != null) 'game_name': gameName,
        if (gameIconPath != null) 'game_icon_path': gameIconPath,
        'device_type': deviceType,
        'emulation_mode': emulationMode,
        'button_mappings': buttonMappings.map((b) => b.toJson()).toList(),
        'axis_mappings': axisMappings.map((a) => a.toJson()).toList(),
        'created_at': createdAt.toIso8601String(),
        'updated_at': updatedAt.toIso8601String(),
        'is_default': isDefault,
        if (author != null) 'author': author,
        'version': version,
      };

  /// Encode to JSON string
  String toJsonString() => jsonEncode(toJson());

  /// Decode from JSON string
  static MappingProfile fromJsonString(String s) =>
      MappingProfile.fromJson(jsonDecode(s) as Map<String, dynamic>);

  MappingProfile copyWith({
    String? id,
    String? name,
    String? description,
    String? gameName,
    String? gameIconPath,
    String? deviceType,
    String? emulationMode,
    List<ButtonMapping>? buttonMappings,
    List<AxisMapping>? axisMappings,
    DateTime? createdAt,
    DateTime? updatedAt,
    bool? isDefault,
    String? author,
    String? version,
  }) =>
      MappingProfile(
        id: id ?? this.id,
        name: name ?? this.name,
        description: description ?? this.description,
        gameName: gameName ?? this.gameName,
        gameIconPath: gameIconPath ?? this.gameIconPath,
        deviceType: deviceType ?? this.deviceType,
        emulationMode: emulationMode ?? this.emulationMode,
        buttonMappings: buttonMappings ?? this.buttonMappings,
        axisMappings: axisMappings ?? this.axisMappings,
        createdAt: createdAt ?? this.createdAt,
        updatedAt: updatedAt ?? DateTime.now(),
        isDefault: isDefault ?? this.isDefault,
        author: author ?? this.author,
        version: version ?? this.version,
      );

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is MappingProfile &&
          runtimeType == other.runtimeType &&
          id == other.id;

  @override
  int get hashCode => id.hashCode;
}

// ─── Default PlayStation button targets ─────────────────────────────────────
class PsButtons {
  PsButtons._();
  static const String cross = 'CROSS';
  static const String circle = 'CIRCLE';
  static const String square = 'SQUARE';
  static const String triangle = 'TRIANGLE';
  static const String l1 = 'L1';
  static const String l2 = 'L2';
  static const String l3 = 'L3';
  static const String r1 = 'R1';
  static const String r2 = 'R2';
  static const String r3 = 'R3';
  static const String dpadUp = 'DPAD_UP';
  static const String dpadDown = 'DPAD_DOWN';
  static const String dpadLeft = 'DPAD_LEFT';
  static const String dpadRight = 'DPAD_RIGHT';
  static const String options = 'OPTIONS';
  static const String share = 'SHARE';
  static const String create = 'CREATE';
  static const String ps = 'PS';
  static const String touchpad = 'TOUCHPAD';
  static const String mute = 'MUTE';
  static const String fn1 = 'FN1';
  static const String fn2 = 'FN2';
  static const String back1 = 'BACK1';
  static const String back2 = 'BACK2';
  static const String back3 = 'BACK3';
  static const String back4 = 'BACK4';

  static const List<String> all = [
    cross, circle, square, triangle,
    l1, l2, l3, r1, r2, r3,
    dpadUp, dpadDown, dpadLeft, dpadRight,
    options, share, create, ps, touchpad, mute,
    fn1, fn2, back1, back2, back3, back4,
  ];
}

// ─── Default PlayStation axis targets ────────────────────────────────────────
class PsAxes {
  PsAxes._();
  static const String lx = 'LX';
  static const String ly = 'LY';
  static const String rx = 'RX';
  static const String ry = 'RY';
  static const String l2 = 'L2_AXIS';
  static const String r2 = 'R2_AXIS';

  static const List<String> all = [lx, ly, rx, ry, l2, r2];
}
