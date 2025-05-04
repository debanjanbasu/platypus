#[cfg(test)]
pub mod tests {
    use crate::ffi::SwiftLibrary::can_check_biometrics;

    #[test]
    fn test_can_check_biometrics() {
        assert!(can_check_biometrics());
    }
}
