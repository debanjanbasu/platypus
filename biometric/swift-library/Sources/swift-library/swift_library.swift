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
    localized_reason: RustString,
    result: @escaping (RustResult<String, String>) -> Void
) {
    let context = LAContext()
    let semaphore = DispatchSemaphore(value: 0)

    // Ensure evaluatePolicy runs on the main thread if UI interaction is involved
    // or if the callback needs to interact with main-thread-only state.
    // However, for simplicity and based on the original code structure,
    // we'll keep the execution as is, assuming the callback is thread-safe.
    context.evaluatePolicy(
        .deviceOwnerAuthentication,
        localizedReason: localized_reason.toString()
    ) { success, error in
        // Call the result callback exactly once
        if success {
            // Assuming RustResult<Bool, String> uses Bool for the Ok case's associated value
            result(.Ok("true"))
        } else {
            // Assuming RustResult<Bool, String> uses String (meaning RustString here based on original usage)
            // for the Err case's associated value.
            let errorDescription = error?.localizedDescription ?? "Authentication failed"
            // Assuming String has an extension method intoRustString()
            // Also assuming the generic parameter String in RustResult<Bool, String> maps to RustString
            // and that RustString is Sendable.
            result(.Err(errorDescription))
        }

        // Signal the semaphore after the callback is invoked
        semaphore.signal()
    }

    // Wait for the asynchronous operation to complete before returning
    // This makes the Swift function synchronous from the caller's perspective.
    // Consider if this blocking behavior is desirable, especially if called from the main thread.
    semaphore.wait()

    // Function is void, so no return statement. Removed: return resultSuccess
}
