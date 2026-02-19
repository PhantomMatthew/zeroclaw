//! Feishu (Lark Chinese version) channel implementation.
//!
//! This is a dedicated channel for Feishu (飞书), the Chinese version of Lark.
//! It wraps `LarkChannel` with `use_feishu: true` fixed and provides a cleaner
//! configuration interface without the `use_feishu` field.

use super::lark::LarkChannel;
use super::traits::{Channel, ChannelMessage, SendMessage};
use crate::config::schema::FeishuConfig;
use async_trait::async_trait;

/// Feishu channel (飞书) - Chinese version of Lark.
///
/// This channel always uses Feishu endpoints (`open.feishu.cn`).
/// For the international version, use `LarkChannel` instead.
pub struct FeishuChannel {
    inner: LarkChannel,
}

impl FeishuChannel {
    /// Create a new Feishu channel.
    ///
    /// # Arguments
    /// * `app_id` - Feishu application ID
    /// * `app_secret` - Feishu application secret
    /// * `verification_token` - Token for webhook verification (optional for WebSocket mode)
    /// * `port` - HTTP port for webhook mode (ignored for WebSocket mode)
    /// * `allowed_users` - List of allowed user open_ids, or `["*"]` for all users
    pub fn new(
        app_id: String,
        app_secret: String,
        verification_token: String,
        port: Option<u16>,
        allowed_users: Vec<String>,
    ) -> Self {
        let inner = LarkChannel::new(app_id, app_secret, verification_token, port, allowed_users)
            .with_feishu(true);
        Self { inner }
    }

    /// Create a Feishu channel from configuration.
    pub fn from_config(config: &FeishuConfig) -> Self {
        let inner = LarkChannel::new(
            config.app_id.clone(),
            config.app_secret.clone(),
            config.verification_token.clone().unwrap_or_default(),
            config.port,
            config.allowed_users.clone(),
        )
        .with_feishu(true)
        .with_receive_mode(config.receive_mode.clone());
        Self { inner }
    }

    /// Check if a user open_id is allowed.
    fn is_user_allowed(&self, open_id: &str) -> bool {
        self.inner.is_user_allowed(open_id)
    }
}

#[async_trait]
impl Channel for FeishuChannel {
    fn name(&self) -> &str {
        "feishu"
    }

    async fn send(&self, message: &SendMessage) -> anyhow::Result<()> {
        self.inner.send(message).await
    }

    async fn listen(&self, tx: tokio::sync::mpsc::Sender<ChannelMessage>) -> anyhow::Result<()> {
        self.inner.listen(tx).await
    }

    async fn health_check(&self) -> bool {
        self.inner.health_check().await
    }

    async fn start_typing(&self, recipient: &str) -> anyhow::Result<()> {
        self.inner.start_typing(recipient).await
    }

    async fn stop_typing(&self, recipient: &str) -> anyhow::Result<()> {
        self.inner.stop_typing(recipient).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_channel() -> FeishuChannel {
        FeishuChannel::new(
            "cli_test_app_id".into(),
            "test_app_secret".into(),
            "test_verification_token".into(),
            None,
            vec!["ou_testuser123".into()],
        )
    }

    #[test]
    fn feishu_channel_name() {
        let ch = make_channel();
        assert_eq!(ch.name(), "feishu");
    }

    #[test]
    fn feishu_user_allowed_exact() {
        let ch = make_channel();
        assert!(ch.is_user_allowed("ou_testuser123"));
        assert!(!ch.is_user_allowed("ou_other"));
    }

    #[test]
    fn feishu_user_allowed_wildcard() {
        let ch = FeishuChannel::new(
            "id".into(),
            "secret".into(),
            "token".into(),
            None,
            vec!["*".into()],
        );
        assert!(ch.is_user_allowed("ou_anyone"));
    }

    #[test]
    fn feishu_from_config() {
        use crate::config::schema::LarkReceiveMode;
        let config = FeishuConfig {
            app_id: "cli_app123".into(),
            app_secret: "secret456".into(),
            encrypt_key: None,
            verification_token: Some("vtoken789".into()),
            allowed_users: vec!["ou_user1".into(), "ou_user2".into()],
            receive_mode: LarkReceiveMode::Websocket,
            port: None,
        };
        let ch = FeishuChannel::from_config(&config);
        assert_eq!(ch.name(), "feishu");
        assert!(ch.is_user_allowed("ou_user1"));
        assert!(!ch.is_user_allowed("ou_stranger"));
    }

    #[tokio::test]
    async fn feishu_health_check_needs_network() {
        let ch = make_channel();
        // Without valid credentials, health check will fail
        // This is expected behavior - we're just verifying it compiles and runs
        let _ = ch.health_check().await;
    }
}
