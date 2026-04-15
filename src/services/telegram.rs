use reqwest::Client;
use serde::Serialize;
use std::env;

#[allow(dead_code)]
#[derive(Serialize)]
struct TelegramMessage {
    chat_id: String,
    text: String,
}

#[allow(dead_code)]
pub struct TelegramBot {
    client: Client,
    bot_token: String,
    chat_id: String,
}

#[allow(dead_code)]
impl TelegramBot {
    pub fn new() -> Option<Self> {
        let bot_token = env::var("TELEGRAM_BOT_TOKEN").ok()?;
        let chat_id = env::var("TELEGRAM_CHAT_ID").ok()?;
        Some(Self {
            client: Client::new(),
            bot_token,
            chat_id,
        })
    }

    pub async fn send(&self, message: &str) -> Result<(), reqwest::Error> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);

        self.client
            .post(&url)
            .json(&TelegramMessage {
                chat_id: self.chat_id.clone(),
                text: message.to_string(),
            })
            .send()
            .await?;

        Ok(())
    }
}

// Alert types
#[allow(dead_code)]
pub async fn alert_account_banned(account_id: i32, reason: &str) {
    if let Some(bot) = TelegramBot::new() {
        let message = format!("🚨 Account Banned\nID: {}\nReason: {}", account_id, reason);
        let _ = bot.send(&message).await;
    }
}

#[allow(dead_code)]
pub async fn alert_proxy_down(proxy_id: i32, proxy_url: &str) {
    if let Some(bot) = TelegramBot::new() {
        let message = format!("⚠️ Proxy Down\nID: {}\nURL: {}", proxy_id, proxy_url);
        let _ = bot.send(&message).await;
    }
}

#[allow(dead_code)]
pub async fn alert_high_error_rate(error_rate: f64) {
    if let Some(bot) = TelegramBot::new() {
        let message = format!(
            "🔴 High Error Rate\nRate: {:.1}%\n\nPlease check the system immediately.",
            error_rate
        );
        let _ = bot.send(&message).await;
    }
}

#[allow(dead_code)]
pub async fn alert_payment_received(
    payment_id: i32,
    user_email: &str,
    amount: &str,
    currency: &str,
) {
    if let Some(bot) = TelegramBot::new() {
        let message = format!(
            "💰 New Payment Received\nID: {}\nUser: {}\nAmount: {} {}",
            payment_id, user_email, amount, currency
        );
        let _ = bot.send(&message).await;
    }
}

#[allow(dead_code)]
pub async fn alert_backup_failed(reason: &str) {
    if let Some(bot) = TelegramBot::new() {
        let message = format!(
            "❌ Backup Failed\nReason: {}\n\nPlease check the backup system.",
            reason
        );
        let _ = bot.send(&message).await;
    }
}
