@_cdecl("__swift_bridge__$can_check_biometrics")
func __swift_bridge__can_check_biometrics () -> Bool {
    can_check_biometrics()
}

@_cdecl("__swift_bridge__$authenticate")
func __swift_bridge__authenticate (_ localized_reason: RustStr, _ callback: UnsafeMutableRawPointer) {
    authenticate(localized_reason: localized_reason, callback: Unmanaged<CallBackType>.fromOpaque(callback).takeRetainedValue())
}


@_cdecl("__swift_bridge__$CallBackType$_free")
func __swift_bridge__CallBackType__free (ptr: UnsafeMutableRawPointer) {
    let _ = Unmanaged<CallBackType>.fromOpaque(ptr).takeRetainedValue()
}
protocol __swift_bridge__IsSendable: Sendable {}
extension CallBackType: __swift_bridge__IsSendable {}

@_cdecl("__swift_bridge__$CallBackType$_free")
func __swift_bridge__CallBackType__free (ptr: UnsafeMutableRawPointer) {
    let _ = Unmanaged<CallBackType>.fromOpaque(ptr).takeRetainedValue()
}
extension CallBackType: __swift_bridge__IsSendable {}


