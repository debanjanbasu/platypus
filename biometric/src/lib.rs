#[cfg(test)]
mod test;

#[swift_bridge::bridge]
mod ffi {

    extern "Swift" {
        fn can_check_biometrics() -> bool;
    }

    extern "Swift" {
        fn authenticate(localized_reason: &str) -> bool;
    }
}
