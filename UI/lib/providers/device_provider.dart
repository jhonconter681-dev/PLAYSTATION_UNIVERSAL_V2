// ─────────────────────────────────────────────────────────────
// PUCE Device Provider
// ─────────────────────────────────────────────────────────────

import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../ffi/puce_bridge.dart';
import '../models/device_info.dart';
import '../models/emulation_mode.dart';

// Stream of devices from the core backend
final deviceListProvider = StreamProvider<List<DeviceInfo>>((ref) async* {
  final bridge = PuceBridge.instance;
  
  // Start detection when listening begins
  bridge.startDetection();

  // Poll for devices every second (in a real app, this might be event-driven via a callback)
  while (true) {
    await Future.delayed(const Duration(seconds: 1));
    yield bridge.getDevices();
  }
});

// Currently selected device ID
final selectedDeviceIdProvider = StateProvider<String?>((ref) => null);

// Derived provider for the full selected DeviceInfo object
final selectedDeviceProvider = Provider<DeviceInfo?>((ref) {
  final devicesList = ref.watch(deviceListProvider).valueOrNull ?? [];
  final selectedId = ref.watch(selectedDeviceIdProvider);
  
  if (selectedId == null) return null;
  
  try {
    return devicesList.firstWhere((d) => d.id == selectedId);
  } catch (_) {
    return null; // Not found
  }
});

// Global app status (running, version, device count)
final systemStatusProvider = StreamProvider<Map<String, dynamic>>((ref) async* {
  final bridge = PuceBridge.instance;
  while (true) {
    await Future.delayed(const Duration(milliseconds: 500));
    yield bridge.getStatus();
  }
});

// Active emulation mode state notifier
class EmulationModeNotifier extends StateNotifier<Map<String, EmulationMode>> {
  EmulationModeNotifier() : super({});

  void setMode(String deviceId, EmulationMode mode) {
    final newState = Map<String, EmulationMode>.from(state);
    newState[deviceId] = mode;
    state = newState;
    
    // Also notify the native backend
    try {
      PuceBridge.instance.setEmulationMode(deviceId, mode);
    } catch (e) {
      // Handle error natively
      print('Error setting emulation mode: $e');
    }
  }

  EmulationMode getMode(String deviceId) {
    return state[deviceId] ?? EmulationMode.dualShock4;
  }
}

final emulationModeProvider = StateNotifierProvider<EmulationModeNotifier, Map<String, EmulationMode>>((ref) {
  return EmulationModeNotifier();
});
