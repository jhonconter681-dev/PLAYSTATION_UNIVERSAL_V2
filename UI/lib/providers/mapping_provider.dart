// ─────────────────────────────────────────────────────────────
// PUCE Mapping Provider
// ─────────────────────────────────────────────────────────────

import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../models/mapping_profile.dart';
import '../ffi/puce_bridge.dart';

// Provides a list of saved profiles (stubbed for now, normally loaded from DB)
final savedProfilesProvider = StateNotifierProvider<ProfilesNotifier, List<MappingProfile>>((ref) {
  return ProfilesNotifier();
});

class ProfilesNotifier extends StateNotifier<List<MappingProfile>> {
  ProfilesNotifier() : super([
    // Default universal profile
    MappingProfile(
      id: 'default-profile-id',
      name: 'Universal Default',
      psMode: 'DualShock4',
      buttonMappings: [],
      axisMappings: [],
      virtualButtons: [],
      isDefault: true,
    )
  ]);

  void addProfile(MappingProfile profile) {
    state = [...state, profile];
  }

  void updateProfile(MappingProfile profile) {
    state = [
      for (final p in state)
        if (p.id == profile.id) profile else p
    ];
  }

  void removeProfile(String id) {
    state = state.where((p) => p.id != id).toList();
  }
}

// The currently active profile being edited or applied
final activeProfileProvider = StateProvider<MappingProfile?>((ref) {
  return ref.watch(savedProfilesProvider).firstOrNull;
});

// A provider that applies the selected profile to the selected device
final profileApplierProvider = Provider<void>((ref) {
  final activeProfile = ref.watch(activeProfileProvider);
  // We can't watch selectedDeviceIdProvider directly here without causing UI rebuilds,
  // but in a full implementation we'd trigger the FFI call here or via a dedicated button.
  if (activeProfile != null) {
    // PuceBridge.instance.applyProfile(deviceId, activeProfile.id);
  }
});
