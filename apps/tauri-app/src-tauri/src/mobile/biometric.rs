//! Biometric authentication for mobile platforms.
//!
//! This module provides biometric authentication support for iOS (Face ID/Touch
//! ID) and Android (Fingerprint/Face).

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// Biometric authentication type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiometricType {
    /// Face recognition (Face ID on iOS, Face Unlock on Android)
    Face,
    /// Fingerprint recognition (Touch ID on iOS, Fingerprint on Android)
    Fingerprint,
    /// Iris scanning (Android only)
    Iris,
    /// No biometric available
    None,
}

/// Biometric authentication status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiometricStatus {
    /// Whether biometric authentication is available.
    pub available: bool,
    /// The type of biometric available.
    pub biometric_type: BiometricType,
    /// Whether the user has enrolled in biometric authentication.
    pub enrolled: bool,
    /// Human-readable description of the biometric type.
    pub description: String,
}

impl Default for BiometricStatus {
    fn default() -> Self {
        Self {
            available: false,
            biometric_type: BiometricType::None,
            enrolled: false,
            description: "Biometric authentication not available".to_string(),
        }
    }
}

/// Biometric authentication manager.
pub struct BiometricAuth;

impl BiometricAuth {
    /// Checks if biometric authentication is available.
    pub fn check_availability() -> BiometricStatus {
        #[cfg(target_os = "ios")]
        {
            Self::check_ios_availability()
        }

        #[cfg(target_os = "android")]
        {
            Self::check_android_availability()
        }

        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        {
            BiometricStatus::default()
        }
    }

    /// Authenticates the user using biometrics.
    ///
    /// # Arguments
    ///
    /// * `reason` - The reason for requesting authentication (shown to user)
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if authentication succeeded, `Ok(false)` if the user
    /// cancelled, or an error if authentication failed.
    pub async fn authenticate(reason: &str) -> AppResult<bool> {
        #[cfg(target_os = "ios")]
        {
            Self::authenticate_ios(reason).await
        }

        #[cfg(target_os = "android")]
        {
            Self::authenticate_android(reason).await
        }

        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        {
            let _ = reason;
            Err(AppError::PlatformError(
                "Biometric authentication is only available on mobile devices".to_string(),
            ))
        }
    }

    /// iOS-specific availability check.
    #[cfg(target_os = "ios")]
    fn check_ios_availability() -> BiometricStatus {
        // In a real implementation, this would use LocalAuthentication framework:
        // LAContext().canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics)
        //
        // This is a placeholder that would be replaced with actual Swift/ObjC bindings
        // using something like swift-bridge or objc crate.
        BiometricStatus {
            available: true,
            biometric_type: BiometricType::Face, // Would check LABiometryType
            enrolled: true,
            description: "Face ID".to_string(),
        }
    }

    /// Android-specific availability check.
    #[cfg(target_os = "android")]
    fn check_android_availability() -> BiometricStatus {
        // In a real implementation, this would use BiometricManager:
        // BiometricManager.from(context).canAuthenticate(BIOMETRIC_STRONG)
        //
        // This is a placeholder that would be replaced with actual JNI bindings
        // or Tauri's Android plugin system.
        BiometricStatus {
            available: true,
            biometric_type: BiometricType::Fingerprint, // Would check actual type
            enrolled: true,
            description: "Fingerprint".to_string(),
        }
    }

    /// iOS-specific authentication.
    #[cfg(target_os = "ios")]
    async fn authenticate_ios(reason: &str) -> AppResult<bool> {
        // In a real implementation, this would use LocalAuthentication framework:
        // let context = LAContext()
        // context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics,
        //                        localizedReason: reason)
        //
        // This would be implemented using Swift/ObjC bindings.
        let _ = reason;
        tracing::warn!(
            "iOS biometric authentication not yet implemented - reason: {}",
            reason
        );

        // Return error instead of placeholder success for security
        Err(AppError::PlatformError(
            "iOS biometric authentication not yet implemented. Native integration required."
                .to_string(),
        ))
    }

    /// Android-specific authentication.
    #[cfg(target_os = "android")]
    async fn authenticate_android(reason: &str) -> AppResult<bool> {
        // In a real implementation, this would use BiometricPrompt:
        // BiometricPrompt.Builder(activity)
        //     .setTitle("DexDex Authentication")
        //     .setDescription(reason)
        //     .build()
        //     .authenticate()
        //
        // This would be implemented using JNI bindings or Tauri's Android plugin
        // system.
        let _ = reason;
        tracing::warn!(
            "Android biometric authentication not yet implemented - reason: {}",
            reason
        );

        // Return error instead of placeholder success for security
        Err(AppError::PlatformError(
            "Android biometric authentication not yet implemented. Native integration required."
                .to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biometric_status_default() {
        let status = BiometricStatus::default();
        assert!(!status.available);
        assert_eq!(status.biometric_type, BiometricType::None);
        assert!(!status.enrolled);
    }

    #[test]
    fn test_check_availability() {
        let status = BiometricAuth::check_availability();

        #[cfg(not(any(target_os = "ios", target_os = "android")))]
        {
            assert!(!status.available);
            assert_eq!(status.biometric_type, BiometricType::None);
        }
    }

    #[test]
    fn test_biometric_type_serialization() {
        let face = BiometricType::Face;
        let json = serde_json::to_string(&face).unwrap();
        assert_eq!(json, "\"face\"");

        let fingerprint = BiometricType::Fingerprint;
        let json = serde_json::to_string(&fingerprint).unwrap();
        assert_eq!(json, "\"fingerprint\"");
    }
}
