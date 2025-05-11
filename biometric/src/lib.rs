use std::sync::LazyLock;
use thiserror::Error; // Import thiserror
use tokio::sync::{Mutex as AsyncMutex, oneshot, oneshot::Sender};

#[cfg(test)]
mod test;

// Define the custom error type
#[derive(Error, Debug)]
pub enum AuthenticationError {
    #[error("Authentication already in progress")]
    InProgress,
    #[error("Authentication channel closed prematurely")]
    ChannelClosed,
    #[error("Authentication callback invoked without a pending authentication request")]
    CallbackWithoutRequest,
    #[error("Receiver for authentication result was dropped before callback completion")]
    ReceiverDroppedInCallback,
    #[error("Biometric authentication failed: {0}")]
    SwiftReportedError(String),
    #[error("Internal callback error: {0}. Swift error: {1}")]
    CallbackOperationFailedWithSwiftError(String, String),
    #[error("Internal callback error: {0}")]
    CallbackOperationFailed(String),
}

#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        // The signature remains Result<bool, String> for FFI
        async fn authentication_done_callback(
            authenticated: bool,
            error: String,
        ) -> Result<bool, String>;
    }
    extern "Swift" {
        fn can_check_biometrics() -> bool;
        fn authenticate(localized_reason: &str);
    }
}

#[must_use = "Need to ensure that biometric capabilities are present before doing anything else."]
pub fn can_check_biometrics() -> bool {
    ffi::can_check_biometrics()
}

static PENDING_AUTH_SENDER: LazyLock<AsyncMutex<Option<Sender<bool>>>> =
    LazyLock::new(|| AsyncMutex::new(None));

// Updated to return Result<bool, AuthenticationError>
/// Initiates the biometric authentication flow.
///
/// This function triggers the native Swift `authenticate` function and waits
/// for the result via a oneshot channel, which is signalled by the
/// `authentication_done_callback` FFI function.
///
/// Only one authentication request can be pending at a time.
///
/// # Arguments
/// * `localized_reason` - The localized string to display to the user explaining
///   why authentication is needed.
///
/// # Errors
///
/// Returns an `AuthenticationError` if:
///
/// * `AuthenticationError::InProgress`: Another authentication request is already
///   pending (i.e., `authenticate` was called again before the previous call
///   completed or failed).
/// * `AuthenticationError::ChannelClosed`: The internal channel used to signal
///   completion from the native callback was closed before the callback
///   function could send the result. This typically means the `authentication_done_callback`
///   function was never invoked, or an unexpected panic occurred between
///   `ffi::authenticate` and `authentication_done_callback`.
pub async fn authenticate(localized_reason: &str) -> Result<bool, AuthenticationError> {
    let (tx, rx) = oneshot::channel::<bool>();

    {
        let mut sender_opt_guard = PENDING_AUTH_SENDER.lock().await;
        if sender_opt_guard.is_some() {
            return Err(AuthenticationError::InProgress);
        }
        *sender_opt_guard = Some(tx);
    }

    // Call into the native Swift function to start the authentication UI
    ffi::authenticate(localized_reason);

    // Wait for the authentication_done_callback to be called via the oneshot channel
    rx.await.map_err(|_| AuthenticationError::ChannelClosed)
}

// FFI signature remains Result<bool, String>
/// Callback invoked by the Swift layer when the authentication process completes.
///
/// This function receives the authentication result (`authenticated`) and any error
/// message from Swift (`error_str`). It signals the waiting `authenticate` future
/// via a oneshot channel and returns a `Result` formatted for the FFI layer.
///
/// # Arguments
/// * `authenticated` - `true` if authentication was successful, `false` otherwise.
/// * `error_str` - An error message from Swift if authentication failed, otherwise an empty string.
///
/// # Errors
///
/// Returns an `Err(String)` in the following cases:
///
/// *   If the callback is invoked without a pending authentication request. The returned string
///     describes the `AuthenticationError::CallbackWithoutRequest` condition, potentially
///     including the Swift error if one was provided.
/// *   If the receiver for the authentication result was dropped before the callback
///     completed. The returned string describes the `AuthenticationError::ReceiverDroppedInCallback`
///     condition, potentially including the Swift error if one was provided.
/// *   If the Swift layer reported an error during authentication (i.e., `error_str` is not empty).
///     The returned string wraps the Swift error message, indicating an `AuthenticationError::SwiftReportedError`.
///     Note that in this case, `authenticated` might still be true or false depending on how
///     Swift signals error vs. simple cancellation/failure without an error string.
pub async fn authentication_done_callback(
    authenticated: bool,
    error_str: String,
) -> Result<bool, String> {
    let sender_to_use: Option<Sender<bool>>;
    {
        let mut sender_opt_guard = PENDING_AUTH_SENDER.lock().await;
        sender_to_use = sender_opt_guard.take();
    }

    if let Some(tx) = sender_to_use {
        if tx.send(authenticated).is_err() {
            // Receiver was dropped. This means the `authenticate()` future
            // is no longer waiting.
            let rust_err_msg = AuthenticationError::ReceiverDroppedInCallback.to_string();
            return if error_str.is_empty() {
                Err(AuthenticationError::CallbackOperationFailed(rust_err_msg).to_string())
            } else {
                Err(AuthenticationError::CallbackOperationFailedWithSwiftError(
                    rust_err_msg,
                    error_str,
                )
                .to_string())
            };
        }

        // Successfully sent status. Now handle Swift error if any.
        if error_str.is_empty() {
            Ok(authenticated)
        } else {
            Err(AuthenticationError::SwiftReportedError(error_str).to_string())
        }
    } else {
        // No sender found.
        let rust_err_msg = AuthenticationError::CallbackWithoutRequest.to_string();
        if error_str.is_empty() {
            Err(AuthenticationError::CallbackOperationFailed(rust_err_msg).to_string())
        } else {
            Err(
                AuthenticationError::CallbackOperationFailedWithSwiftError(rust_err_msg, error_str)
                    .to_string(),
            )
        }
    }
}
