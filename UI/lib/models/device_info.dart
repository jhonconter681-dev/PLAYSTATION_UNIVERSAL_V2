// lib/models/device_info.dart
// PUCE - PlayStation Universal Controller Emulator
// Complete device info model with JSON serialization

/// Represents a connected input device detected by PUCE core.
class DeviceInfo {
  final String id;
  final int vendorId;
  final int productId;
  final String name;
  final String manufacturer;
  final String deviceType;
  final int buttonCount;
  final int axisCount;
  final List<String> sensors;
  final List<String> capabilities;
  final String? firmware;
  final String? bluetoothVersion;
  final String? usbVersion;
  final double batteryLevel; // 0.0 – 1.0
  final double latencyMs;
  final bool isConnected;
  final bool isWireless;
  final String connectionType; // 'usb' | 'bluetooth' | 'unknown'
  final DateTime? lastSeen;

  const DeviceInfo({
    required this.id,
    required this.vendorId,
    required this.productId,
    required this.name,
    required this.manufacturer,
    required this.deviceType,
    required this.buttonCount,
    required this.axisCount,
    required this.sensors,
    required this.capabilities,
    this.firmware,
    this.bluetoothVersion,
    this.usbVersion,
    this.batteryLevel = 1.0,
    this.latencyMs = 0.0,
    this.isConnected = false,
    this.isWireless = false,
    this.connectionType = 'unknown',
    this.lastSeen,
  });

  // ─── Factory: from JSON ────────────────────────────────────────────────
  factory DeviceInfo.fromJson(Map<String, dynamic> json) {
    return DeviceInfo(
      id: json['id'] as String? ?? '',
      vendorId: json['vendor_id'] as int? ?? 0,
      productId: json['product_id'] as int? ?? 0,
      name: json['name'] as String? ?? 'Unknown Device',
      manufacturer: json['manufacturer'] as String? ?? 'Unknown',
      deviceType: json['device_type'] as String? ?? 'generic',
      buttonCount: json['button_count'] as int? ?? 0,
      axisCount: json['axis_count'] as int? ?? 0,
      sensors: (json['sensors'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          [],
      capabilities: (json['capabilities'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          [],
      firmware: json['firmware'] as String?,
      bluetoothVersion: json['bluetooth_version'] as String?,
      usbVersion: json['usb_version'] as String?,
      batteryLevel: (json['battery_level'] as num?)?.toDouble() ?? 1.0,
      latencyMs: (json['latency_ms'] as num?)?.toDouble() ?? 0.0,
      isConnected: json['is_connected'] as bool? ?? false,
      isWireless: json['is_wireless'] as bool? ?? false,
      connectionType: json['connection_type'] as String? ?? 'unknown',
      lastSeen: json['last_seen'] != null
          ? DateTime.tryParse(json['last_seen'] as String)
          : null,
    );
  }

  // ─── To JSON ───────────────────────────────────────────────────────────
  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'vendor_id': vendorId,
      'product_id': productId,
      'name': name,
      'manufacturer': manufacturer,
      'device_type': deviceType,
      'button_count': buttonCount,
      'axis_count': axisCount,
      'sensors': sensors,
      'capabilities': capabilities,
      if (firmware != null) 'firmware': firmware,
      if (bluetoothVersion != null) 'bluetooth_version': bluetoothVersion,
      if (usbVersion != null) 'usb_version': usbVersion,
      'battery_level': batteryLevel,
      'latency_ms': latencyMs,
      'is_connected': isConnected,
      'is_wireless': isWireless,
      'connection_type': connectionType,
      if (lastSeen != null) 'last_seen': lastSeen!.toIso8601String(),
    };
  }

  // ─── copyWith ──────────────────────────────────────────────────────────
  DeviceInfo copyWith({
    String? id,
    int? vendorId,
    int? productId,
    String? name,
    String? manufacturer,
    String? deviceType,
    int? buttonCount,
    int? axisCount,
    List<String>? sensors,
    List<String>? capabilities,
    String? firmware,
    String? bluetoothVersion,
    String? usbVersion,
    double? batteryLevel,
    double? latencyMs,
    bool? isConnected,
    bool? isWireless,
    String? connectionType,
    DateTime? lastSeen,
  }) {
    return DeviceInfo(
      id: id ?? this.id,
      vendorId: vendorId ?? this.vendorId,
      productId: productId ?? this.productId,
      name: name ?? this.name,
      manufacturer: manufacturer ?? this.manufacturer,
      deviceType: deviceType ?? this.deviceType,
      buttonCount: buttonCount ?? this.buttonCount,
      axisCount: axisCount ?? this.axisCount,
      sensors: sensors ?? List.from(this.sensors),
      capabilities: capabilities ?? List.from(this.capabilities),
      firmware: firmware ?? this.firmware,
      bluetoothVersion: bluetoothVersion ?? this.bluetoothVersion,
      usbVersion: usbVersion ?? this.usbVersion,
      batteryLevel: batteryLevel ?? this.batteryLevel,
      latencyMs: latencyMs ?? this.latencyMs,
      isConnected: isConnected ?? this.isConnected,
      isWireless: isWireless ?? this.isWireless,
      connectionType: connectionType ?? this.connectionType,
      lastSeen: lastSeen ?? this.lastSeen,
    );
  }

  // ─── Convenience ──────────────────────────────────────────────────────
  String get vendorIdHex =>
      '0x${vendorId.toRadixString(16).toUpperCase().padLeft(4, '0')}';
  String get productIdHex =>
      '0x${productId.toRadixString(16).toUpperCase().padLeft(4, '0')}';
  bool get hasMotion => sensors.any((s) => s.toLowerCase().contains('gyro') ||
      s.toLowerCase().contains('accel'));
  bool get hasTouchpad =>
      capabilities.any((c) => c.toLowerCase().contains('touch'));
  bool get hasRumble =>
      capabilities.any((c) => c.toLowerCase().contains('rumble') ||
          c.toLowerCase().contains('haptic'));

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is DeviceInfo &&
          runtimeType == other.runtimeType &&
          id == other.id;

  @override
  int get hashCode => id.hashCode;

  @override
  String toString() =>
      'DeviceInfo(id: $id, name: $name, connected: $isConnected)';
}

/// Represents a momentary button/axis state snapshot from a device.
class DeviceState {
  final String deviceId;
  final Map<String, bool> buttons;   // button name → pressed
  final Map<String, double> axes;    // axis name → value (-1.0 – 1.0)
  final Map<String, double> sensors; // sensor name → value
  final DateTime timestamp;

  const DeviceState({
    required this.deviceId,
    required this.buttons,
    required this.axes,
    required this.sensors,
    required this.timestamp,
  });

  factory DeviceState.empty(String deviceId) => DeviceState(
        deviceId: deviceId,
        buttons: {},
        axes: {},
        sensors: {},
        timestamp: DateTime.now(),
      );

  factory DeviceState.fromJson(Map<String, dynamic> json) {
    return DeviceState(
      deviceId: json['device_id'] as String? ?? '',
      buttons: (json['buttons'] as Map<String, dynamic>?)
              ?.map((k, v) => MapEntry(k, v as bool)) ??
          {},
      axes: (json['axes'] as Map<String, dynamic>?)
              ?.map((k, v) => MapEntry(k, (v as num).toDouble())) ??
          {},
      sensors: (json['sensors'] as Map<String, dynamic>?)
              ?.map((k, v) => MapEntry(k, (v as num).toDouble())) ??
          {},
      timestamp: json['timestamp'] != null
          ? DateTime.tryParse(json['timestamp'] as String) ?? DateTime.now()
          : DateTime.now(),
    );
  }

  bool get isAnyButtonPressed => buttons.values.any((v) => v);
}
