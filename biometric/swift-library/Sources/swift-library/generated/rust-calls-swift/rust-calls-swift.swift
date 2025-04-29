@_cdecl("__swift_bridge__$can_check_biometrics")
func __swift_bridge__can_check_biometrics () -> Bool {
    can_check_biometrics()
}

@_cdecl("__swift_bridge__$authenticate")
func __swift_bridge__authenticate (_ localized_reason: RustStr) -> Bool {
    authenticate(localized_reason: localized_reason)
}



