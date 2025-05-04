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
    let notice_localized_reason = String(
        data: Data(
            buffer: UnsafeBufferPointer(start: localized_reason, count: Int(localized_reason_len))),
        encoding: .utf8)!
    print(notice_localized_reason)
    return true
}
