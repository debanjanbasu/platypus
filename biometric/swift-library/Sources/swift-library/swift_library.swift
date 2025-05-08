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
    localized_reason: RustStr,
    callback: @escaping @Sendable (RustResult<String, String>) -> Void
) {
    // Convert the RustString to a Swift String outside the Task.
    // String is Sendable, so it can be safely captured by the Task.
    let reasonText = localized_reason.toString()

    Task {
        // The Task closure now captures 'reasonText' (a String) and 'callback' (marked @Sendable).
        // Both are Sendable, so the closure itself becomes Sendable, satisfying Task's requirements.
        let context = LAContext()

        do {
            // Use the async/await version of evaluatePolicy, available from iOS 16.
            // This method throws errors like LAError.userCancel, LAError.biometryNotAvailable, etc.
            // If authentication fails (user enters wrong passcode, fails fingerprint scan),
            // it returns 'false' without throwing.
            let success = try await context.evaluatePolicy(
                .deviceOwnerAuthenticationWithBiometrics, localizedReason: reasonText)  // Use the captured Sendable String

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
