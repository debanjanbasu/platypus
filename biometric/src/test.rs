#[cfg(test)]
pub mod tests {
    use crate::get_biometric_service;

    #[test]
    fn test_can_check_biometrics() {
        let service = get_biometric_service();
        assert!(service.can_check());
    }

    #[tokio::test]
    async fn test_authenticate() {
        let service = get_biometric_service();
        assert!(
            service
                .authenticate("use your device's biometrics for trust store")
                .await
                .unwrap_or_default()
        );
    }
}