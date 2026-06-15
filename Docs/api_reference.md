# PUCE API Reference

This document outlines the Foreign Function Interface (FFI) exposed by the `libpuce_core` native Rust library. This API is used by the Flutter frontend and any other third-party interfaces.

## Initialization & Lifecycle

### `int32_t puce_init()`
Initializes the PUCE core engines (Detection, Mapping, Emulation, AI, Database).
**Returns**: `0` on success, or a negative error code.

### `void puce_shutdown()`
Stops all engines safely, unloads plugins, and releases handles.

### `const char* puce_get_version()`
Returns a pointer to a null-terminated string containing the core version.
**Note**: The returned string pointer must be freed using `puce_free_string()`.

## Device Management

### `int32_t puce_start_detection()`
Starts the background scanning threads for USB and Bluetooth HID devices.

### `void puce_stop_detection()`
Stops the device detection threads.

### `const char* puce_get_devices()`
Returns a JSON-formatted string of all currently connected and recognized devices.
**Returns**: JSON string pointer. Must be freed with `puce_free_string()`.

### `const char* puce_get_status()`
Returns a JSON-formatted string detailing the real-time status of the engine (running state, device count, latency averages).

## Emulation Control

### `int32_t puce_set_emulation_mode(const char* device_id, int32_t mode)`
Assigns a specific PlayStation emulation mode to a connected device.
- `mode = 0` : PS1
- `mode = 6` : DualShock 4
- `mode = 7` : DualSense
**Returns**: `0` on success.

### `int32_t puce_apply_profile(const char* device_id, const char* profile_id)`
Applies a specific mapping profile from the SQLite database to the selected device.

## Memory Management

### `void puce_free_string(const char* ptr)`
Frees memory allocated by the Rust core for strings returned via FFI. MUST be called to prevent memory leaks.
