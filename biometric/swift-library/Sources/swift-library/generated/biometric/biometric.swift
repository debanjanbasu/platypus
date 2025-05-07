@_cdecl("__swift_bridge__$can_check_biometrics")
func __swift_bridge__can_check_biometrics () -> Bool {
    can_check_biometrics()
}

@_cdecl("__swift_bridge__$authenticate")
func __swift_bridge__authenticate (_ localized_reason: UnsafeMutableRawPointer, _ callback: UnsafeMutableRawPointer) {
    { let cb1 = __private__RustFnOnceCallback$authenticate$param1(ptr: callback); let _ = authenticate(localized_reason: RustString(ptr: localized_reason), callback: { arg0 in cb1.call(arg0) }) }()
}
class __private__RustFnOnceCallback$authenticate$param1 {
    var ptr: UnsafeMutableRawPointer
    var called = false

    init(ptr: UnsafeMutableRawPointer) {
        self.ptr = ptr
    }

    deinit {
        if !called {
            __swift_bridge__$authenticate$_free$param1(ptr)
        }
    }

    func call<GenericIntoRustString: IntoRustString>(_ arg0: RustResult<GenericIntoRustString, GenericIntoRustString>) {
        if called {
            fatalError("Cannot call a Rust FnOnce function twice")
        }
        called = true
        return __swift_bridge__$authenticate$param1(ptr, { switch arg0 { case .Ok(let ok): return __private__ResultPtrAndPtr(is_ok: true, ok_or_err: { let rustString = ok.intoRustString(); rustString.isOwned = false; return rustString.ptr }()) case .Err(let err): return __private__ResultPtrAndPtr(is_ok: false, ok_or_err: { let rustString = err.intoRustString(); rustString.isOwned = false; return rustString.ptr }()) } }())
    }
}



