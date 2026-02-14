// Email 服務模組
// 拆分自原始 email.rs（1,192 行）

mod auth;
mod protocol;
mod alert;

use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::config::Config;

pub struct EmailService;

impl EmailService {
    /// 通用發送郵件方法
    pub(crate) async fn send_email(
        config: &Config,
        smtp_host: &str,
        to_email: &str,
        to_name: &str,
        subject: &str,
        plain_body: &str,
        html_body: &str,
    ) -> anyhow::Result<()> {
        // 檢查收件人 email 是否為空或無效，若無效則跳過寄信
        if to_email.is_empty() || !to_email.contains('@') {
            tracing::warn!(
                "Skipping email '{}' - invalid recipient address: '{}'",
                subject, to_email
            );
            return Ok(());
        }

        let from = format!("{} <{}>", config.smtp_from_name, config.smtp_from_email);

        let email = Message::builder()
            .from(from.parse()?)
            .to(format!("{} <{}>", to_name, to_email).parse()?)
            .subject(subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(plain_body.to_string()),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_body.to_string()),
                    ),
            )?;

        // Gmail port 587 需要 STARTTLS
        let mailer = if let (Some(username), Some(password)) =
            (&config.smtp_username, &config.smtp_password)
        {
            let creds = Credentials::new(username.clone(), password.clone());
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)?
                .port(config.smtp_port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)?
                .port(config.smtp_port)
                .build()
        };

        mailer.send(email).await?;
        Ok(())
    }
}
