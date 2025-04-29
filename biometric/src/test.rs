#[cfg(test)]
pub mod tests {
    use crate::ffi::{authenticate, can_check_biometrics};

    #[test]
    fn test_can_check_biometrics() {
        assert!(can_check_biometrics());
    }

    #[test]
    fn test_authenticate() {
        assert!(authenticate("test"));
    }
}
