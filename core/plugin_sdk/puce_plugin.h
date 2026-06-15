/**
 * @file puce_plugin.h
 * @brief Official C-ABI SDK for PUCE (PlayStation Universal Controller Emulator) Plugins.
 * 
 * Allows third-party developers to add support for new controllers, 
 * custom mappings, or hardware extensions without modifying the PUCE core.
 * 
 * Target ABI Version: 1
 */

#ifndef PUCE_PLUGIN_H
#define PUCE_PLUGIN_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

#define PUCE_PLUGIN_ABI_VERSION 1

/**
 * @brief Information about the plugin.
 */
typedef struct {
    const char* name;
    const char* version;
    const char* author;
    const char* description;
    uint32_t abi_version; // MUST be set to PUCE_PLUGIN_ABI_VERSION
} PucePluginInfo;

/**
 * @brief Basic device identifiers passed by the PUCE core during discovery.
 */
typedef struct {
    uint16_t vendor_id;
    uint16_t product_id;
    const char* name;
    const char* manufacturer;
    uint8_t button_count;
    uint8_t axis_count;
} DeviceInfoC;

/**
 * @brief Maps a source hardware button (0-255) to a target PlayStation button.
 */
typedef struct {
    uint8_t source_button;
    uint8_t target_button; // Matches PUCE PSButton enum
    uint8_t modifier;      // 0 = none
} ButtonMappingC;

/**
 * @brief Maps a source hardware axis (0-255) to a target PlayStation axis.
 */
typedef struct {
    uint8_t source_axis;
    uint8_t target_axis; // Matches PUCE PSAxis enum
    float scale;
    float dead_zone;
    uint8_t invert; // 0 = normal, 1 = inverted
} AxisMappingC;

/**
 * @brief A full input mapping profile returned by the plugin.
 */
typedef struct {
    const char* name;
    uint8_t ps_mode; // Matches PUCE PSMode enum
    
    const ButtonMappingC* button_mappings;
    uint32_t button_count;
    
    const AxisMappingC* axis_mappings;
    uint32_t axis_count;
} MappingProfileC;

/* ========================================================================= */
/* REQUIRED EXPORTS                                                          */
/* Every PUCE plugin MUST implement and export these functions.              */
/* ========================================================================= */

#ifdef _WIN32
  #define PUCE_EXPORT __declspec(dllexport)
#else
  #define PUCE_EXPORT __attribute__((visibility("default")))
#endif

/**
 * @brief Initialize the plugin. Called once when loaded.
 * @return true if initialization successful, false otherwise.
 */
PUCE_EXPORT bool puce_plugin_init(void);

/**
 * @brief Shutdown the plugin and free resources. Called before unloading.
 */
PUCE_EXPORT void puce_plugin_shutdown(void);

/**
 * @brief Returns a pointer to the static PluginInfo struct.
 */
PUCE_EXPORT const PucePluginInfo* puce_plugin_get_info(void);

/**
 * @brief Checks if this plugin knows how to handle the given device.
 * @param device The device detected by the PUCE core.
 * @return true if the plugin provides mappings/logic for this device.
 */
PUCE_EXPORT bool puce_plugin_handles_device(const DeviceInfoC* device);

/**
 * @brief Gets the custom mapping profile for the handled device.
 * @param device The device to map.
 * @return Pointer to a static/heap MappingProfileC, or NULL if error.
 */
PUCE_EXPORT const MappingProfileC* puce_plugin_get_mapping(const DeviceInfoC* device);

#ifdef __cplusplus
}
#endif

#endif // PUCE_PLUGIN_H
