// Email - 認證相關（歡迎信、密碼重設、密碼變更）

use crate::config::Config;
use super::EmailService;

impl EmailService {
    /// 寄送歡迎信給新用戶（含密碼重設連結，不含明文密碼）
    pub async fn send_welcome_email(
        config: &Config,
        to_email: &str,
        display_name: &str,
        reset_token: &str,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping welcome email to {}", to_email);
            return Ok(());
        }
        let smtp_host = config.smtp_host.as_ref()
            .ok_or_else(|| anyhow::anyhow!("SMTP_HOST not configured"))?;
        let reset_url = format!("{}/reset-password?token={}", config.app_url, reset_token);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>歡迎加入豬博士 iPig 系統</title>
</head>
<body style="margin: 0; padding: 0; background-color: #f1f5f9; font-family: 'Microsoft JhengHei', sans-serif;">
    <div style="padding: 40px 20px;">
        <div style="max-width: 600px; margin: 0 auto; background: white; border-radius: 16px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
            <div style="background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%); color: white; padding: 32px 24px; text-align: center; border-radius: 16px 16px 0 0;">
                <h1 style="margin: 0; font-size: 24px;">🐷 歡迎加入豬博士 iPig 系統</h1>
                <p style="margin-top: 8px; font-size: 14px; opacity: 0.9;">您的帳號已成功開通</p>
            </div>
            <div style="padding: 32px 24px;">
                <p>親愛的 <strong>{display_name}</strong>，您好！</p>
                <p>您的豬博士 iPig 系統帳號已開通。</p>

                <div style="background: #ffffff; padding: 24px; border-radius: 12px; margin: 24px 0; border: 1px solid #cbd5e1; border-left: 4px solid #3b82f6;">
                    <p style="margin: 8px 0;"><span style="color: #64748b;">📧 帳號 (Email)</span><br><span style="font-weight: 700; color: #0f172a;">{to_email}</span></p>
                    <p style="margin: 16px 0; color: #64748b; font-size: 14px;">✨ 請點擊下方按鈕設定您的密碼</p>
                </div>

                <div style="background: #fffbeb; border-radius: 8px; padding: 16px; margin: 24px 0; border-left: 4px solid #f59e0b;">
                    <p style="margin: 0; color: #92400e; font-size: 14px;">⏰ 此連結將在 <strong>24 小時</strong>後失效，請儘速設定密碼。</p>
                </div>

                <div style="text-align: center; margin: 32px 0;">
                    <a href="{reset_url}" style="display: inline-block; background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%); color: #ffffff; padding: 14px 32px; text-decoration: none; border-radius: 8px; font-weight: 600;">設定我的密碼</a>
                </div>

                <div style="background: #f8fafc; border-radius: 8px; padding: 16px; margin-top: 24px; text-align: center; font-size: 14px; color: #64748b;">
                    如有任何問題，請聯繫工作人員<br>📞 電話：037-433789
                </div>
            </div>
            <div style="background: #f8fafc; padding: 24px; text-align: center; border-top: 1px solid #e2e8f0;">
                <p style="margin: 4px 0; font-size: 12px; color: #94a3b8;">此信件由系統自動發送，請勿直接回覆</p>
                <p style="margin: 4px 0; font-size: 12px; color: #64748b;">© 2026 豬博士動物科技有限公司</p>
            </div>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name,
            to_email = to_email,
            reset_url = reset_url,
        );

        let plain_body = format!(
            r#"歡迎加入豬博士 iPig 系統

親愛的 {display_name}，您好！

您的豬博士 iPig 系統帳號已開通。

📧 帳號（Email）：{to_email}

請點擊以下連結設定您的密碼：
{reset_url}

⏰ 此連結將在 24 小時後失效，請儘速設定密碼。

如有任何問題，請聯繫工作人員（電話：037-433789）。

此信件由系統自動發送，請勿直接回覆。
© 2026 豬博士動物科技有限公司"#,
            display_name = display_name,
            to_email = to_email,
            reset_url = reset_url,
        );

        Self::send_email(
            config,
            smtp_host,
            to_email,
            display_name,
            "🐷 歡迎加入豬博士 iPig 系統 - 請設定您的密碼",
            &plain_body,
            &html_body,
        )
        .await?;

        tracing::info!("Welcome email sent to {}", to_email);
        Ok(())
    }

    /// 寄送密碼重設信
    pub async fn send_password_reset_email(
        config: &Config, to_email: &str, display_name: &str, reset_token: &str,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping password reset email to {}", to_email);
            return Ok(());
        }
        let smtp_host = config.smtp_host.as_ref()
            .ok_or_else(|| anyhow::anyhow!("SMTP_HOST not configured"))?;
        let reset_url = format!("{}/reset-password?token={}", config.app_url, reset_token);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Microsoft JhengHei', Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #dc2626; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ background: #f8fafc; padding: 30px; border: 1px solid #e2e8f0; }}
        .info-box {{ background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border-left: 4px solid #dc2626; }}
        .button {{ display: inline-block; background: #dc2626; color: white; padding: 12px 30px; text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
        .warning {{ color: #dc2626; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="cid:logo" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1 style="color: #ffffff;">🔑 密碼重設通知</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>我們收到您重設密碼的請求。請點擊下方按鈕重設您的密碼：</p>
            <center><a href="{reset_url}" class="button">重設密碼</a></center>
            <div class="info-box">
                <p class="warning">⚠️ 此連結將於 1 小時後失效。</p>
                <p>如果您沒有發起此請求，請忽略此信件，您的帳號密碼不會被變更。</p>
            </div>
            <p>如有任何問題，請聯繫工作人員（電話：037-433789）。</p>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name, reset_url = reset_url,
        );

        let plain_body = format!(
            r#"密碼重設通知

親愛的 {display_name}，您好！

我們收到您重設密碼的請求。請點擊以下連結重設您的密碼：

{reset_url}

⚠️ 此連結將於 1 小時後失效。
如果您沒有發起此請求，請忽略此信件，您的帳號密碼不會被變更。

如有任何問題，請聯繫工作人員（電話：037-433789）。

此信件由系統自動發送，請勿直接回覆。
© 2026 豬博士動物科技有限公司"#,
            display_name = display_name, reset_url = reset_url,
        );

        Self::send_email(config, smtp_host, to_email, display_name,
            "🔑 豬博士 iPig 系統 - 密碼重設通知",
            &plain_body, &html_body).await?;

        tracing::info!("Password reset email sent to {}", to_email);
        Ok(())
    }

    /// 寄送密碼變更成功通知
    pub async fn send_password_changed_email(
        config: &Config, to_email: &str, display_name: &str,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping password changed email to {}", to_email);
            return Ok(());
        }
        let smtp_host = config.smtp_host.as_ref()
            .ok_or_else(|| anyhow::anyhow!("SMTP_HOST not configured"))?;
        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Microsoft JhengHei', Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #16a34a; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ background: #f8fafc; padding: 30px; border: 1px solid #e2e8f0; }}
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
        .warning {{ color: #dc2626; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="cid:logo" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1 style="color: #ffffff;">✅ 密碼變更成功</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>您的豬博士 iPig 系統密碼已成功變更。</p>
            <p class="warning">⚠️ 如果這不是您本人的操作，請立即聯繫系統管理員。</p>
            <p>如有任何問題，請聯繫工作人員（電話：037-433789）。</p>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name,
        );

        let plain_body = format!(
            r#"密碼變更成功

親愛的 {display_name}，您好！

您的豬博士 iPig 系統密碼已成功變更。

⚠️ 如果這不是您本人的操作，請立即聯繫系統管理員。

如有任何問題，請聯繫工作人員（電話：037-433789）。

此信件由系統自動發送，請勿直接回覆。
© 2026 豬博士動物科技有限公司"#,
            display_name = display_name,
        );

        Self::send_email(config, smtp_host, to_email, display_name,
            "✅ 豬博士 iPig 系統 - 密碼變更成功",
            &plain_body, &html_body).await?;

        tracing::info!("Password changed email sent to {}", to_email);
        Ok(())
    }
}
