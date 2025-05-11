#[cfg(test)]
pub mod tests {
    use crate::{authenticate, can_check_biometrics};

    #[test]
    fn test_can_check_biometrics() {
        assert!(can_check_biometrics());
    }

    #[tokio::test]
    async fn test_authenticate() {
        assert!(authenticate("Test").await.unwrap_or_default());
    }
}
