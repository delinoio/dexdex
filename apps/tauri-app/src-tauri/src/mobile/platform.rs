//! Mobile platform detection and configuration.
//!
//! This module provides utilities for detecting mobile platforms and enforcing
//! remote-only mode on mobile devices.

use serde::{Deserialize, Serialize};

/// Mobile platform type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MobilePlatform {
    /// iOS (iPhone, iPad)
    Ios,
    /// Android
    Android,
}

/// Platform information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    /// Whether the app is running on a mobile device.
    pub is_mobile: bool,
    /// The mobile platform, if applicable.
    pub mobile_platform: Option<MobilePlatform>,
    /// Whether local mode is supported on this platform.
    pub supports_local_mode: bool,
    /// Whether biometric authentication is available.
    pub biometric_available: bool,
    /// The device model (if available).
    pub device_model: Option<String>,
    /// The OS version.
    pub os_version: Option<String>,
}

impl PlatformInfo {
    /// Creates platform info for the current platform.
    pub fn current() -> Self {
        #[cfg(target_os = "ios")]
        {
            Self {
                is_mobile: true,
                mobile_platform: Some(MobilePlatform::Ios),
                supports_local_mode: false,
                biometric_available: true, // Will be checked at runtime
                device_model: get_ios_device_model(),
                os_version: get_ios_version(),
            }
        }

        #[cfg(target_os = "android")]
        {
            Self {
                is_mobile: true,
                mobile_platform: Some(MobilePlatform::Android),
                supports_local_mode: false,
                biometric_available: true, // Will be checked at runtime
                device_model: get_android_device_model(),
                os_version: get_android_version(),
            }
        }

        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        {
            Self {
                is_mobile: false,
                mobile_platform: None,
                supports_local_mode: true,
                biometric_available: false,
                device_model: None,
                os_version: Some(std::env::consts::OS.to_string()),
            }
        }
    }
}

/// Returns whether the app is running on a mobile device.
#[inline]
pub fn is_mobile() -> bool {
    cfg!(any(target_os = "ios", target_os = "android"))
}

/// Returns whether local mode is supported on the current platform.
///
/// Local mode requires Docker and full file system access, which are not
/// available on mobile devices.
#[inline]
pub fn supports_local_mode() -> bool {
    !is_mobile()
}

/// Returns the mobile platform, if running on mobile.
pub fn get_mobile_platform() -> Option<MobilePlatform> {
    #[cfg(target_os = "ios")]
    {
        Some(MobilePlatform::Ios)
    }

    #[cfg(target_os = "android")]
    {
        Some(MobilePlatform::Android)
    }

    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    {
        None
    }
}

/// Gets the iOS device model (placeholder - would use UIDevice in production).
#[cfg(target_os = "ios")]
fn get_ios_device_model() -> Option<String> {
    // In a real implementation, this would use UIDevice.current.model
    // through Swift/Objective-C bindings
    Some("iPhone".to_string())
}

/// Gets the iOS version (placeholder - would use UIDevice in production).
#[cfg(target_os = "ios")]
fn get_ios_version() -> Option<String> {
    // In a real implementation, this would use UIDevice.current.systemVersion
    Some("iOS".to_string())
}

/// Gets the Android device model (placeholder - would use Build.MODEL in
/// production).
#[cfg(target_os = "android")]
fn get_android_device_model() -> Option<String> {
    // In a real implementation, this would use android.os.Build.MODEL
    Some("Android Device".to_string())
}

/// Gets the Android version (placeholder - would use Build.VERSION.RELEASE in
/// production).
#[cfg(target_os = "android")]
fn get_android_version() -> Option<String> {
    // In a real implementation, this would use android.os.Build.VERSION.RELEASE
    Some("Android".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_info_current() {
        let info = PlatformInfo::current();

        #[cfg(target_os = "ios")]
        {
            assert!(info.is_mobile);
            assert_eq!(info.mobile_platform, Some(MobilePlatform::Ios));
            assert!(!info.supports_local_mode);
        }

        #[cfg(target_os = "android")]
        {
            assert!(info.is_mobile);
            assert_eq!(info.mobile_platform, Some(MobilePlatform::Android));
            assert!(!info.supports_local_mode);
        }

        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        {
            assert!(!info.is_mobile);
            assert!(info.mobile_platform.is_none());
            assert!(info.supports_local_mode);
        }
    }

    #[test]
    fn test_is_mobile() {
        let result = is_mobile();

        #[cfg(any(target_os = "ios", target_os = "android"))]
        assert!(result);

        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        assert!(!result);
    }

    #[test]
    fn test_supports_local_mode() {
        let result = supports_local_mode();

        #[cfg(any(target_os = "ios", target_os = "android"))]
        assert!(!result);

        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        assert!(result);
    }
}
