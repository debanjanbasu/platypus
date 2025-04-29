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
    print(localized_reason)
    return true
}
