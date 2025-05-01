// The Swift Programming Language
// https://docs.swift.org/swift-book
import LocalAuthentication

func can_check_biometrics() -> Bool {
    let context = LAContext()
    var error: NSError?
    return context.canEvaluatePolicy(
        .deviceOwnerAuthenticationWithBiometrics, error: &error)
}

func authenticate(localized_reason: RustStr) -> Bool {
    let context = LAContext()
    var resultSuccess = false
    var resultError: Error?
    let semaphore = DispatchSemaphore(value: 0)

    context.evaluatePolicy(
        .deviceOwnerAuthentication,
        localizedReason: localized_reason.toString()
    ) { success, error in
        if success {
            resultSuccess = true
        } else {
            resultSuccess = false
            resultError = error
        }
        semaphore.signal()
    }

    semaphore.wait()

    return resultSuccess
}
