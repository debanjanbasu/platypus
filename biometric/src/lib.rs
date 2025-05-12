use thiserror::Error;
use tokio::sync::oneshot::{self, error::RecvError};

#[cfg(test)]
mod test;

#[derive(Error, Debug)]
pub enum BiometricError {
    #[error("Failed to receive authentication status from native layer: {0}")]
    CallbackReceiveError(#[from] RecvError),

    #[error("Biometric authentication failed on native side: {0}")]
    NativeAuthFailed(String),
}

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

#[must_use = "Need to ensure that biometric capabilities are present before doing anything else."]
pub fn can_check_biometrics() -> bool {
    ffi::can_check_biometrics()
}

/// Asynchronously performs biometric authentication using the native platform's capabilities.
///
/// Presents the user with the native biometric authentication UI (e.g., Face ID, Touch ID).
/// Waits for the user's interaction and returns the authentication result.
///
/// # Arguments
///
/// * `localized_reason` - A string explaining why the authentication is needed. This is typically
///   displayed to the user in the authentication prompt.
///
/// # Returns
///
/// A `Result` indicating the outcome:
/// - `Ok(true)` if the authentication was successful.
/// - `Ok(false)` if the authentication failed (e.g., user cancelled, incorrect biometric).
/// - `Err(BiometricError)` if an error occurred during the process.
///
/// # Errors
///
/// This function can return the following errors:
///
/// * `BiometricError::CallbackReceiveError`: If there was an issue receiving the authentication
///   result back from the native layer (e.g., the callback mechanism failed).
/// * `BiometricError::NativeAuthFailed`: If the native authentication process itself reported
///   an error (e.g., system error, configuration issue). The contained `String` provides
///   details from the native side.
pub async fn authenticate(localized_reason: &str) -> Result<bool, BiometricError> {
    // The receiver needs to handle Result<bool, String> where String is the potential error from Swift
    let (tx, rx) = oneshot::channel::<Result<bool, String>>();

    // Call into the native Swift function to start the authentication UI
    ffi::authenticate_with_callback(
        localized_reason,
        Box::new(move |result: Result<String, String>| {
            // Simplify the mapping from Swift's Result<String, String> to Result<bool, String>
            // If Ok(status_str), map it to Ok(status_str == "true").
            // If Err(error_string), it remains Err(error_string).
            let message_to_send = result.map(|status_str| status_str == "true");

            // Send the processed result. Ignore send errors as they are handled by rx.await below.
            let _ = tx.send(message_to_send);
        }),
    );

    // Wait for the callback result.
    // rx.await returns Result<Result<bool, String>, RecvError>
    // The outer `?` handles the RecvError, converting it via #[from] in BiometricError::CallbackReceiveError
    let callback_payload: Result<bool, String> = rx.await?;

    // At this point, callback_payload is Result<bool, String>.
    // The String is the error message from the Swift side, if one occurred there.
    // Map the potential error string from Swift into our NativeAuthFailed variant.
    // If callback_payload is Ok(bool), it remains Ok(bool).
    // If callback_payload is Err(String), it becomes Err(BiometricError::NativeAuthFailed(String)).
    callback_payload.map_err(BiometricError::NativeAuthFailed)
}
