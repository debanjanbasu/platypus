use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

#[cfg(target_vendor = "apple")]
use tokio::sync::oneshot::error::RecvError;

#[cfg(test)]
mod test;

#[derive(Error, Debug)]
pub enum BiometricError {
    /// Indicates that the current platform does not support biometric authentication.
    #[error("Biometric authentication is not supported on this platform")]
    UnsupportedPlatform,

    #[cfg(target_vendor = "apple")]
    #[error("Failed to receive authentication status from native layer: {0}")]
    CallbackReceiveError(#[from] RecvError),

    #[cfg(target_vendor = "apple")]
    #[error("Biometric authentication failed on native side: {0}")]
    NativeAuthFailed(String),
}

#[async_trait]
pub trait BiometricService: Send + Sync {
    fn can_check(&self) -> bool;
    async fn authenticate(&self, localized_reason: &str) -> Result<bool, BiometricError>;
}

#[cfg(target_vendor = "apple")]
#[swift_bridge::bridge]
mod ffi {
    extern "Swift" {
        fn can_check_biometrics() -> bool;
        fn authenticate_with_callback(
            localized_reason: &str,
            callback: Box<dyn FnOnce(Result<String, String>)>,
        );
    }
}

// Native (Apple platforms) implementation
#[cfg(target_vendor = "apple")]
mod native {
    use super::{BiometricError, BiometricService};
    use async_trait::async_trait;
    use tokio::sync::oneshot;

    pub struct NativeBiometricService;

    #[async_trait]
    impl BiometricService for NativeBiometricService {
        fn can_check(&self) -> bool {
            super::ffi::can_check_biometrics()
        }

        async fn authenticate(&self, localized_reason: &str) -> Result<bool, BiometricError> {
            let (tx, rx) = oneshot::channel();

            super::ffi::authenticate_with_callback(
                localized_reason,
                Box::new(move |result: Result<String, String>| {
                    let _ = tx.send(result.map(|s| s == "true"));
                }),
            );

            rx.await
                .map_err(BiometricError::CallbackReceiveError)?
                .map_err(BiometricError::NativeAuthFailed)
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::{BiometricError, BiometricService};
    use async_trait::async_trait;

    pub struct WasmBiometricService;

    #[async_trait]
    impl BiometricService for WasmBiometricService {
        fn can_check(&self) -> bool {
            false
        }

        async fn authenticate(&self, _localized_reason: &str) -> Result<bool, BiometricError> {
            Err(BiometricError::UnsupportedPlatform)
        }
    }
}

#[must_use]
pub fn get_biometric_service() -> Arc<dyn BiometricService> {
    #[cfg(target_vendor = "apple")]
    {
        Arc::new(native::NativeBiometricService)
    }

    #[cfg(target_arch = "wasm32")]
    {
        Arc::new(wasm::WasmBiometricService)
    }
}
