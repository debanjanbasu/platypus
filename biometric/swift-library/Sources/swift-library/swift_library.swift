// The Swift Programming Language
// https://docs.swift.org/swift-book
import LocalAuthentication

public func can_check_biometrics() -> Bool {
    let context = LAContext()
    var error: NSError?
    return context.canEvaluatePolicy(
        .deviceOwnerAuthenticationWithBiometrics, error: &error)
}

public func authenticate(localized_reason: UnsafePointer<UInt8>, localized_reason_len: UInt8)
    -> Bool
{
    return true
}
