// The Swift Programming Language
// https://docs.swift.org/swift-book
import LocalAuthentication

func can_check_biometrics() -> Bool {
    let context = LAContext()
    var error: NSError?
    return context.canEvaluatePolicy(
        .deviceOwnerAuthenticationWithBiometrics, error: &error)
}

func authenticate(
    localized_reason: String,
    callback: @escaping @Sendable (RustResult<String, String>) -> Void
) {
    // Fix 1: Ensure localized_reason is a Swift String *before* capturing in the Task.
    let reason = localized_reason

    // Fix 3: Mark the 'callback' parameter as @escaping and @Sendable.
    // @escaping is required because the callback is captured by the Task and called asynchronously.
    // @Sendable is required because the callback is captured by the @Sendable Task closure.
    // The caller must guarantee that the provided callback is safe to call from a
    // concurrent context (i.e., it's @Sendable or correctly handles thread safety).

    Task {  // This Task closure is @Sendable
        let context = LAContext()

        do {
            // Use the async/await version of evaluatePolicy, available from iOS 16.
            // This method throws errors like LAError.userCancel, LAError.biometryNotAvailable, etc.
            // If authentication fails (user enters wrong passcode, fails fingerprint scan),
            // it returns 'false' without throwing.
            let success = try await context.evaluatePolicy(
                .deviceOwnerAuthenticationWithBiometrics, localizedReason: reason)

            // Call the callback with the result.
            // Since callback is marked @Sendable, it is safe to call here within the Task.
            if success {
                callback(.Ok("true"))
            } else {
                // Authentication failed (e.g., user denied or failed to match)
                // The async version returns false but doesn't provide an error object for simple failure=false.
                // We provide a generic message consistent with the original structure.
                callback(.Err("Authentication failed"))
            }
        } catch {
            // Handle errors thrown by evaluatePolicy (e.g., user cancelled, policy not available, etc.)
            let nsError = error as NSError  // LAError is an NSError subclass
            let errorMessage = nsError.localizedDescription
            callback(.Err(errorMessage))
        }
    }
}
