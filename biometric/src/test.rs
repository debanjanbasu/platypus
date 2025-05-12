#[cfg(test)]
pub mod tests {
    use crate::{authenticate, can_check_biometrics};

    #[test]
    fn test_can_check_biometrics() {
        assert!(can_check_biometrics());
    }

    #[tokio::test]
    async fn test_authenticate() {
        assert!(
            authenticate("use your device's biometrics for trust store")
                .await
                .unwrap_or_default()
        );
    }
}
