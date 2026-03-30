// Email 服務模組
// 拆分自原始 email.rs（1,192 行）

mod alert;
mod auth;
mod equipment;
mod invitation;
mod protocol;

use lettre::{
    message::{header::ContentType, Attachment, Body, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

/// 編譯時嵌入 Logo PNG，供所有郵件以 CID inline attachment 方式引用
/// HTML 中以 <img src="cid:logo"> 引用即可，不依賴外部 URL
pub(crate) const LOGO_PNG: &[u8] = include_bytes!("pigmodel-logo.png");

use crate::config::Config;
use crate::services::system_settings::SmtpConfig;

pub struct EmailService;

/// RFC 5322 quoted-string: 跳脫反斜線與雙引號
fn sanitize_display_name(name: &str) -> String {
    name.replace('\\', "\\\\").replace('"', "\\\"")
}

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
                subject,
                to_email
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

        let from = format!("\"{}\" <{}>", sanitize_display_name(&smtp.from_name), smtp.from_email);

        // 建構 MIME：alternative(plain, related(html, inline-logo))
        let logo_attachment = Attachment::new_inline("logo".to_string()).body(
            Body::new(LOGO_PNG.to_vec()),
            "image/png"
                .parse()
                .expect("failed to parse image/png content type"),
        );
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
            .to(format!("\"{}\" <{}>", sanitize_display_name(to_name), to_email).parse()?)
            .subject(subject)
            .multipart(email_body)?;

        let mailer = if let (Some(username), Some(password)) = (&smtp.username, &smtp.password) {
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
                async move {
                    mailer_ref
                        .send(email_clone)
                        .await
                        .map(|_| ())
                        .map_err(|e| anyhow::anyhow!(e))
                }
            },
        )
        .await?;
        Ok(())
    }

    /// 發送 SMTP 設定測試信件
    pub(crate) async fn send_test_email(smtp: &SmtpConfig, to_email: &str) -> anyhow::Result<()> {
        let to_name = to_email.split('@').next().unwrap_or("User");
        let host = smtp.host.as_deref().unwrap_or("-");

        let plain_body = format!(
            "iPig System 測試信件\n\n\
             這是一封測試信件，用於驗證 SMTP 郵件設定是否正確。\n\n\
             SMTP 伺服器：{host}\n\
             寄件人：{} <{}>\n\n\
             如果您收到這封信，表示郵件設定正常運作。",
            smtp.from_name, smtp.from_email,
        );

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head><meta charset="UTF-8"><title>iPig System 測試信件</title></head>
<body style="margin:0;padding:0;background-color:#f1f5f9;font-family:'Microsoft JhengHei',sans-serif;">
<div style="padding:40px 20px;">
<div style="max-width:520px;margin:0 auto;background:#fff;border-radius:12px;overflow:hidden;box-shadow:0 2px 8px rgba(0,0,0,.08);">
  <div style="background:linear-gradient(135deg,#1e293b,#334155);padding:32px;text-align:center;">
    <img src="cid:logo" alt="iPig" style="height:48px;margin-bottom:12px;" />
    <h1 style="margin:0;color:#fff;font-size:20px;">郵件設定測試</h1>
  </div>
  <div style="padding:32px;">
    <p style="color:#334155;font-size:15px;line-height:1.7;">
      這是一封 <strong>測試信件</strong>，用於驗證 SMTP 郵件設定是否正確。
    </p>
    <table style="width:100%;border-collapse:collapse;margin:20px 0;font-size:14px;">
      <tr><td style="padding:8px 12px;color:#64748b;border-bottom:1px solid #e2e8f0;">SMTP 伺服器</td>
          <td style="padding:8px 12px;color:#1e293b;border-bottom:1px solid #e2e8f0;font-weight:600;">{host}</td></tr>
      <tr><td style="padding:8px 12px;color:#64748b;border-bottom:1px solid #e2e8f0;">連接埠</td>
          <td style="padding:8px 12px;color:#1e293b;border-bottom:1px solid #e2e8f0;font-weight:600;">{}</td></tr>
      <tr><td style="padding:8px 12px;color:#64748b;">寄件人</td>
          <td style="padding:8px 12px;color:#1e293b;font-weight:600;">{} &lt;{}&gt;</td></tr>
    </table>
    <div style="background:#f0fdf4;border:1px solid #bbf7d0;border-radius:8px;padding:16px;text-align:center;">
      <p style="margin:0;color:#166534;font-size:15px;font-weight:600;">✓ 郵件設定正常運作</p>
    </div>
  </div>
  <div style="padding:16px 32px;background:#f8fafc;border-top:1px solid #e2e8f0;text-align:center;">
    <p style="margin:0;color:#94a3b8;font-size:12px;">此信件由 iPig System 自動發送</p>
  </div>
</div>
</div>
</body>
</html>"#,
            smtp.port, smtp.from_name, smtp.from_email,
        );

        Self::send_email_smtp(smtp, to_email, to_name, "iPig System 測試信件", &plain_body, &html_body).await
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
                subject,
                to_email
            );
            return Ok(());
        }

        let from = format!("\"{}\" <{}>", sanitize_display_name(&config.smtp_from_name), config.smtp_from_email);

        // 建構 MIME：alternative(plain, related(html, inline-logo))
        let logo_attachment = Attachment::new_inline("logo".to_string()).body(
            Body::new(LOGO_PNG.to_vec()),
            "image/png"
                .parse()
                .expect("failed to parse image/png content type"),
        );
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
            .to(format!("\"{}\" <{}>", sanitize_display_name(to_name), to_email).parse()?)
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
