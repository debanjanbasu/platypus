// The Swift Programming Language
// https://docs.swift.org/swift-book
import LocalAuthentication

func can_check_biometrics() -> Bool {
    let context = LAContext()
    var error: NSError?
    return context.canEvaluatePolicy(
        .deviceOwnerAuthenticationWithBiometrics, error: &error)
}

func authenticate_with_callback(
    localized_reason: RustStr,
    callback: @escaping @Sendable (RustResult<String, String>) -> Void  // Mark callback as escaping and Sendable
) {
    // Convert the RustString to a Swift String outside the Task.
    // String is Sendable, so it can be safely captured by the Task.
    let reasonText = localized_reason.toString()

    Task {
        let context = LAContext()

        do {
            let success = try await context.evaluatePolicy(
                .deviceOwnerAuthenticationWithBiometrics, localizedReason: reasonText)  // Use the captured Sendable String

            if success {
                callback(.Ok("true"))
            } else {
                // Even if authentication fails (e.g., user cancels), it's not an *error* in policy evaluation.
                // The API returns `false` for `success`. We report this as Ok(false).
                callback(.Ok("false"))
            }
        } catch {
            // Handle actual errors during policy evaluation (e.g., biometrics not available/enrolled).
            let nsError = error as NSError  // LAError conforms to NSError
            // Provide a more specific error description if possible
            let laError = error as? LAError
            let errorDescription = laError?.localizedDescription ?? nsError.localizedDescription
            callback(.Err(errorDescription))
        }
    }
}
