// Email 服務模組
// 拆分自原始 email.rs（1,192 行）

mod auth;
mod protocol;
mod alert;

use lettre::{
    message::{
        header::ContentType,
        Attachment, Body, MultiPart, SinglePart,
    },
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

/// 編譯時嵌入 Logo PNG，供所有郵件以 CID inline attachment 方式引用
/// HTML 中以 <img src="cid:logo"> 引用即可，不依賴外部 URL
pub(crate) const LOGO_PNG: &[u8] = include_bytes!("pigmodel-logo.png");

use crate::config::Config;
use crate::services::system_settings::SmtpConfig;

pub struct EmailService;

impl EmailService {
    /// DB-first SMTP resolution: reads settings from DB, falls back to .env Config
    pub async fn resolve_smtp(pool: &sqlx::PgPool, config: &Config) -> SmtpConfig {
        let svc = crate::services::SystemSettingsService::new(pool.clone());
        svc.resolve_smtp_config(config).await
    }

    /// 通用發送郵件方法（使用已解析的 SmtpConfig）
    pub(crate) async fn send_email_smtp(
        smtp: &SmtpConfig,
        to_email: &str,
        to_name: &str,
        subject: &str,
        plain_body: &str,
        html_body: &str,
    ) -> anyhow::Result<()> {
        if to_email.is_empty() || !to_email.contains('@') {
            tracing::warn!(
                "Skipping email '{}' - invalid recipient address: '{}'",
                subject, to_email
            );
            return Ok(());
        }

        let smtp_host = match &smtp.host {
            Some(h) => h.as_str(),
            None => {
                tracing::info!("Email disabled (no SMTP host), skipping: {}", subject);
                return Ok(());
            }
        };

        let from = format!("{} <{}>", smtp.from_name, smtp.from_email);

        // 建構 MIME：alternative(plain, related(html, inline-logo))
        let logo_attachment = Attachment::new_inline("logo".to_string())
            .body(Body::new(LOGO_PNG.to_vec()), "image/png".parse().unwrap());
        let html_with_logo = MultiPart::related()
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html_body.to_string()),
            )
            .singlepart(logo_attachment);
        let email_body = MultiPart::alternative()
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(plain_body.to_string()),
            )
            .multipart(html_with_logo);

        let email = Message::builder()
            .from(from.parse()?)
            .to(format!("{} <{}>", to_name, to_email).parse()?)
            .subject(subject)
            .multipart(email_body)?;

        let mailer = if let (Some(username), Some(password)) =
            (&smtp.username, &smtp.password)
        {
            let creds = Credentials::new(username.clone(), password.clone());
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)?
                .port(smtp.port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(smtp_host)?
                .port(smtp.port)
                .build()
        };

        use crate::services::retry::{with_retry, RetryConfig};
        with_retry(
            &RetryConfig {
                max_retries: 2,
                ..RetryConfig::default()
            },
            "SMTP send",
            || {
                let mailer_ref = &mailer;
                let email_clone = email.clone();
                async move { mailer_ref.send(email_clone).await.map(|_| ()).map_err(|e| anyhow::anyhow!(e)) }
            },
        ).await?;
        Ok(())
    }

    /// Legacy: 通用發送郵件方法（直接使用 .env Config）
    pub(crate) async fn send_email(
        config: &Config,
        smtp_host: &str,
        to_email: &str,
        to_name: &str,
        subject: &str,
        plain_body: &str,
        html_body: &str,
    ) -> anyhow::Result<()> {
        if to_email.is_empty() || !to_email.contains('@') {
            tracing::warn!(
                "Skipping email '{}' - invalid recipient address: '{}'",
                subject, to_email
            );
            return Ok(());
        }

        let from = format!("{} <{}>", config.smtp_from_name, config.smtp_from_email);

        // 建構 MIME：alternative(plain, related(html, inline-logo))
        let logo_attachment = Attachment::new_inline("logo".to_string())
            .body(Body::new(LOGO_PNG.to_vec()), "image/png".parse().unwrap());
        let html_with_logo = MultiPart::related()
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(html_body.to_string()),
            )
            .singlepart(logo_attachment);
        let email_body = MultiPart::alternative()
            .singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(plain_body.to_string()),
            )
            .multipart(html_with_logo);

        let email = Message::builder()
            .from(from.parse()?)
            .to(format!("{} <{}>", to_name, to_email).parse()?)
            .subject(subject)
            .multipart(email_body)?;

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
