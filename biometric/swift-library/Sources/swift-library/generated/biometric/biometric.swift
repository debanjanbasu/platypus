public func authentication_done_callback<GenericIntoRustString: IntoRustString>(_ authenticated: Bool, _ error: GenericIntoRustString) async throws -> Bool {
    func onComplete(cbWrapperPtr: UnsafeMutableRawPointer?, rustFnRetVal: __swift_bridge__$ResultBoolAndString) {
        let wrapper = Unmanaged<CbWrapper$authentication_done_callback>.fromOpaque(cbWrapperPtr!).takeRetainedValue()
        switch rustFnRetVal.tag { case __swift_bridge__$ResultBoolAndString$ResultOk: wrapper.cb(.success(rustFnRetVal.payload.ok)) case __swift_bridge__$ResultBoolAndString$ResultErr: wrapper.cb(.failure(RustString(ptr: rustFnRetVal.payload.err))) default: fatalError() }
    }

    return try await withCheckedThrowingContinuation({ (continuation: CheckedContinuation<Bool, Error>) in
        let callback = { rustFnRetVal in
            continuation.resume(with: rustFnRetVal)
        }

        let wrapper = CbWrapper$authentication_done_callback(cb: callback)
        let wrapperPtr = Unmanaged.passRetained(wrapper).toOpaque()

        __swift_bridge__$authentication_done_callback(wrapperPtr, onComplete, authenticated, { let rustString = error.intoRustString(); rustString.isOwned = false; return rustString.ptr }())
    })
}
class CbWrapper$authentication_done_callback {
    var cb: (Result<Bool, Error>) -> ()

    public init(cb: @escaping (Result<Bool, Error>) -> ()) {
        self.cb = cb
    }
}
@_cdecl("__swift_bridge__$can_check_biometrics")
func __swift_bridge__can_check_biometrics () -> Bool {
    can_check_biometrics()
}

@_cdecl("__swift_bridge__$authenticate")
func __swift_bridge__authenticate (_ localized_reason: RustStr) {
    authenticate(localized_reason: localized_reason)
}



