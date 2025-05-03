#[cfg(test)]
mod test;

#[swift_bridge::bridge]
mod ffi {

    extern "Swift" {
        fn can_check_biometrics() -> bool;
    }

    extern "Swift" {
        fn authenticate(localized_reason: String, result: Box<dyn FnOnce(Result<String, String>)>);
    }
}

#[must_use = "This has to be used beforehand to check if the device supports biometric authentication."]
pub fn can_check_biometrics() -> bool {
    ffi::can_check_biometrics()
}

// pub async fn authenticate()
