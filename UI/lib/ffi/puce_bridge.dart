// ─────────────────────────────────────────────────────────────
// PUCE FFI Bridge — Dart interface to the Rust core library
// ─────────────────────────────────────────────────────────────

import 'dart:ffi';
import 'dart:io';
import 'dart:convert';
import 'package:ffi/ffi.dart';
import '../models/device_info.dart';
import '../models/emulation_mode.dart';
import '../models/mapping_profile.dart';

// ─────────────────────────────────────────────────────────────
// Native function signatures (C ABI)
// ─────────────────────────────────────────────────────────────

typedef _PuceInitNative = Int32 Function();
typedef _PuceInitDart = int Function();

typedef _PuceShutdownNative = Void Function();
typedef _PuceShutdownDart = void Function();

typedef _PuceGetVersionNative = Pointer<Utf8> Function();
typedef _PuceGetVersionDart = Pointer<Utf8> Function();

typedef _PuceStartDetectionNative = Int32 Function();
typedef _PuceStartDetectionDart = int Function();

typedef _PuceStopDetectionNative = Void Function();
typedef _PuceStopDetectionDart = void Function();

typedef _PuceGetDevicesNative = Pointer<Utf8> Function();
typedef _PuceGetDevicesDart = Pointer<Utf8> Function();

typedef _PuceGetStatusNative = Pointer<Utf8> Function();
typedef _PuceGetStatusDart = Pointer<Utf8> Function();

typedef _PuceSetEmulationModeNative = Int32 Function(Pointer<Utf8> deviceId, Int32 mode);
typedef _PuceSetEmulationModeDart = int Function(Pointer<Utf8> deviceId, int mode);

typedef _PuceApplyProfileNative = Int32 Function(Pointer<Utf8> deviceId, Pointer<Utf8> profileId);
typedef _PuceApplyProfileDart = int Function(Pointer<Utf8> deviceId, Pointer<Utf8> profileId);

typedef _PuceStartCalibrationNative = Int32 Function(Pointer<Utf8> deviceId);
typedef _PuceStartCalibrationDart = int Function(Pointer<Utf8> deviceId);

typedef _PuceFreeStringNative = Void Function(Pointer<Utf8> ptr);
typedef _PuceFreeStringDart = void Function(Pointer<Utf8> ptr);

// ─────────────────────────────────────────────────────────────
// Bridge Exceptions
// ─────────────────────────────────────────────────────────────

class PuceBridgeException implements Exception {
  final String message;
  final int? errorCode;
  const PuceBridgeException(this.message, {this.errorCode});

  @override
  String toString() => 'PuceBridgeException: $message (code: $errorCode)';
}

// ─────────────────────────────────────────────────────────────
// PUCE Bridge — Singleton
// ─────────────────────────────────────────────────────────────

class PuceBridge {
  PuceBridge._internal();
  static final PuceBridge instance = PuceBridge._internal();

  DynamicLibrary? _lib;
  bool _initialized = false;

  // Bound functions
  late _PuceInitDart _init;
  late _PuceShutdownDart _shutdown;
  late _PuceGetVersionDart _getVersion;
  late _PuceStartDetectionDart _startDetection;
  late _PuceStopDetectionDart _stopDetection;
  late _PuceGetDevicesDart _getDevices;
  late _PuceGetStatusDart _getStatus;
  late _PuceSetEmulationModeDart _setEmulationMode;
  late _PuceApplyProfileDart _applyProfile;
  late _PuceStartCalibrationDart _startCalibration;
  late _PuceFreeStringDart _freeString;

  // ─────────────────────────────────────────────────────────
  // Initialization
  // ─────────────────────────────────────────────────────────

  Future<void> initialize() async {
    if (_initialized) return;

    try {
      _lib = _loadLibrary();
      _bindFunctions();

      final result = _init();
      if (result != 0) {
        throw PuceBridgeException('puce_init failed', errorCode: result);
      }

      _initialized = true;
    } catch (e) {
      // In debug/dev mode without the native library, use stub mode
      _initialized = true;
      _useStubMode();
      rethrow;
    }
  }

  DynamicLibrary _loadLibrary() {
    if (Platform.isWindows) {
      return DynamicLibrary.open('puce_core.dll');
    } else if (Platform.isLinux || Platform.isAndroid) {
      return DynamicLibrary.open('libpuce_core.so');
    } else if (Platform.isMacOS || Platform.isIOS) {
      return DynamicLibrary.open('libpuce_core.dylib');
    } else {
      throw PuceBridgeException('Unsupported platform: ${Platform.operatingSystem}');
    }
  }

  void _bindFunctions() {
    final lib = _lib!;

    _init = lib.lookupFunction<_PuceInitNative, _PuceInitDart>('puce_init');
    _shutdown = lib.lookupFunction<_PuceShutdownNative, _PuceShutdownDart>('puce_shutdown');
    _getVersion = lib.lookupFunction<_PuceGetVersionNative, _PuceGetVersionDart>('puce_get_version');
    _startDetection = lib.lookupFunction<_PuceStartDetectionNative, _PuceStartDetectionDart>('puce_start_detection');
    _stopDetection = lib.lookupFunction<_PuceStopDetectionNative, _PuceStopDetectionDart>('puce_stop_detection');
    _getDevices = lib.lookupFunction<_PuceGetDevicesNative, _PuceGetDevicesDart>('puce_get_devices');
    _getStatus = lib.lookupFunction<_PuceGetStatusNative, _PuceGetStatusDart>('puce_get_status');
    _setEmulationMode = lib.lookupFunction<_PuceSetEmulationModeNative, _PuceSetEmulationModeDart>('puce_set_emulation_mode');
    _applyProfile = lib.lookupFunction<_PuceApplyProfileNative, _PuceApplyProfileDart>('puce_apply_profile');
    _startCalibration = lib.lookupFunction<_PuceStartCalibrationNative, _PuceStartCalibrationDart>('puce_start_calibration');
    _freeString = lib.lookupFunction<_PuceFreeStringNative, _PuceFreeStringDart>('puce_free_string');
  }

  bool _stubMode = false;

  void _useStubMode() {
    _stubMode = true;
  }

  // ─────────────────────────────────────────────────────────
  // Public API
  // ─────────────────────────────────────────────────────────

  String getVersion() {
    if (_stubMode) return '1.0.0-dev (stub)';
    final ptr = _getVersion();
    final str = ptr.toDartString();
    _freeString(ptr);
    return str;
  }

  void startDetection() {
    if (_stubMode) return;
    final result = _startDetection();
    if (result != 0) {
      throw PuceBridgeException('Failed to start detection', errorCode: result);
    }
  }

  void stopDetection() {
    if (_stubMode) return;
    _stopDetection();
  }

  List<DeviceInfo> getDevices() {
    if (_stubMode) return _stubDevices();

    final ptr = _getDevices();
    if (ptr == nullptr) return [];

    try {
      final json = ptr.toDartString();
      final Map<String, dynamic> data = jsonDecode(json);
      final List<dynamic> deviceList = data['devices'] as List<dynamic>? ?? [];
      return deviceList
          .map((d) => DeviceInfo.fromJson(d as Map<String, dynamic>))
          .toList();
    } catch (e) {
      return [];
    } finally {
      _freeString(ptr);
    }
  }

  Map<String, dynamic> getStatus() {
    if (_stubMode) return _stubStatus();

    final ptr = _getStatus();
    if (ptr == nullptr) return {};

    try {
      final json = ptr.toDartString();
      return jsonDecode(json) as Map<String, dynamic>;
    } catch (_) {
      return {};
    } finally {
      _freeString(ptr);
    }
  }

  void setEmulationMode(String deviceId, EmulationMode mode) {
    if (_stubMode) return;

    final idPtr = deviceId.toNativeUtf8();
    try {
      final result = _setEmulationMode(idPtr, mode.index);
      if (result != 0) {
        throw PuceBridgeException(
          'Failed to set emulation mode',
          errorCode: result,
        );
      }
    } finally {
      malloc.free(idPtr);
    }
  }

  void applyProfile(String deviceId, String profileId) {
    if (_stubMode) return;

    final devicePtr = deviceId.toNativeUtf8();
    final profilePtr = profileId.toNativeUtf8();
    try {
      final result = _applyProfile(devicePtr, profilePtr);
      if (result != 0) {
        throw PuceBridgeException('Failed to apply profile', errorCode: result);
      }
    } finally {
      malloc.free(devicePtr);
      malloc.free(profilePtr);
    }
  }

  void startCalibration(String deviceId) {
    if (_stubMode) return;

    final ptr = deviceId.toNativeUtf8();
    try {
      _startCalibration(ptr);
    } finally {
      malloc.free(ptr);
    }
  }

  void shutdown() {
    if (_stubMode || !_initialized) return;
    _shutdown();
    _initialized = false;
  }

  // ─────────────────────────────────────────────────────────
  // Stub data for development without native library
  // ─────────────────────────────────────────────────────────

  List<DeviceInfo> _stubDevices() {
    return [
      DeviceInfo(
        id: 'stub-device-001',
        vendorId: 0x054C,
        productId: 0x0CE6,
        name: 'DualSense Wireless Controller',
        manufacturer: 'Sony Interactive Entertainment',
        deviceType: 'PlayStation',
        buttonCount: 17,
        axisCount: 6,
        sensors: ['gyroscope', 'accelerometer', 'touchpad'],
        capabilities: ['haptics', 'adaptive_triggers', 'microphone', 'speaker'],
        firmware: '0142',
        bluetoothVersion: '5.0',
        usbVersion: 'USB 3.0',
        batteryLevel: 0.87,
        latencyMs: 1.2,
        isConnected: true,
        connectionType: 'USB',
      ),
    ];
  }

  Map<String, dynamic> _stubStatus() {
    return {
      'running': true,
      'detection_active': true,
      'version': '1.0.0-dev (stub)',
      'device_count': 1,
    };
  }
}
