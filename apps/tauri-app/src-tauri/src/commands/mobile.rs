//! Mobile-specific Tauri commands.
//!
//! This module provides commands for mobile-specific functionality including
//! platform detection, biometric authentication, and push notifications.

use crate::mobile::{
    biometric::{BiometricAuth, BiometricStatus},
    notifications::{MobilePushNotifications, PushRegistrationStatus},
    platform::PlatformInfo,
};

/// Gets information about the current platform.
#[tauri::command]
pub fn get_platform_info() -> PlatformInfo {
    PlatformInfo::current()
}

/// Checks if the app is running on a mobile device.
#[tauri::command]
pub fn is_mobile() -> bool {
    crate::mobile::platform::is_mobile()
}

/// Checks if local mode is supported on the current platform.
#[tauri::command]
pub fn supports_local_mode() -> bool {
    crate::mobile::platform::supports_local_mode()
}

/// Checks biometric authentication availability.
#[tauri::command]
pub fn check_biometric_availability() -> BiometricStatus {
    BiometricAuth::check_availability()
}

/// Authenticates using biometrics.
#[tauri::command]
pub async fn authenticate_biometric(reason: String) -> Result<bool, String> {
    BiometricAuth::authenticate(&reason)
        .await
        .map_err(|e| e.to_string())
}

/// Requests push notification permission.
#[tauri::command]
pub async fn request_push_permission() -> Result<bool, String> {
    MobilePushNotifications::request_permission()
        .await
        .map_err(|e| e.to_string())
}

/// Registers for push notifications.
#[tauri::command]
pub async fn register_push_notifications() -> Result<PushRegistrationStatus, String> {
    MobilePushNotifications::register()
        .await
        .map_err(|e| e.to_string())
}

/// Unregisters from push notifications.
#[tauri::command]
pub async fn unregister_push_notifications() -> Result<(), String> {
    MobilePushNotifications::unregister()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_info() {
        let info = get_platform_info();
        // On desktop, these should be the expected values
        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        {
            assert!(!info.is_mobile);
            assert!(info.supports_local_mode);
        }
    }

    #[test]
    fn test_is_mobile_command() {
        let result = is_mobile();
        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        assert!(!result);
    }

    #[test]
    fn test_supports_local_mode_command() {
        let result = supports_local_mode();
        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        assert!(result);
    }

    #[test]
    fn test_check_biometric_availability_command() {
        let status = check_biometric_availability();
        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        {
            assert!(!status.available);
        }
    }
}
