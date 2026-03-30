// Email - 邀請相關

use crate::config::Config;

use super::EmailService;

impl EmailService {
    /// 寄送邀請 Email 給客戶
    pub async fn send_invitation_email(
        config: &Config,
        to_email: &str,
        invite_link: &str,
        expires_at_formatted: &str,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!(
                "Email disabled, skipping invitation email to {}",
                to_email
            );
            return Ok(());
        }
        let smtp_host = config
            .smtp_host
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("SMTP_HOST not configured"))?;

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>邀請您加入實驗動物管理平台</title>
</head>
<body style="margin: 0; padding: 0; background-color: #f1f5f9; font-family: 'Microsoft JhengHei', sans-serif;">
    <div style="padding: 40px 20px;">
        <div style="max-width: 600px; margin: 0 auto; background: white; border-radius: 16px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
            <div style="background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%); color: white; padding: 32px 24px; text-align: center; border-radius: 16px 16px 0 0;">
                <div style="margin-bottom: 12px;">
                    <img src="cid:logo" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 8px;">
                </div>
                <h1 style="margin: 0; font-size: 22px;">邀請您加入實驗動物管理平台</h1>
            </div>
            <div style="padding: 32px 24px;">
                <p style="color: #334155; font-size: 15px; line-height: 1.8;">您好，</p>
                <p style="color: #334155; font-size: 15px; line-height: 1.8;">
                    豬博士動物科技邀請您加入實驗動物管理平台。
                </p>

                <div style="background: #f0f9ff; border-radius: 12px; padding: 20px; margin: 24px 0; border-left: 4px solid #3b82f6;">
                    <p style="margin: 0 0 8px 0; color: #1e40af; font-weight: 600;">透過此平台，您可以：</p>
                    <ul style="margin: 0; padding-left: 20px; color: #334155; line-height: 2;">
                        <li>線上提交動物實驗計劃書（AUP）</li>
                        <li>即時追蹤審查進度</li>
                        <li>與審查委員線上溝通</li>
                    </ul>
                </div>

                <p style="color: #334155; font-size: 15px; line-height: 1.8;">
                    請點擊以下按鈕完成註冊（連結僅限一次使用）：
                </p>

                <div style="text-align: center; margin: 32px 0;">
                    <a href="{invite_link}" style="display: inline-block; background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%); color: #ffffff; padding: 14px 32px; text-decoration: none; border-radius: 8px; font-weight: 600; font-size: 16px;">完成註冊</a>
                </div>

                <div style="background: #fffbeb; border-radius: 8px; padding: 16px; margin: 24px 0; border-left: 4px solid #f59e0b;">
                    <p style="margin: 0; color: #92400e; font-size: 14px;">⏰ 此連結將於 <strong>{expires_at}</strong> 到期。</p>
                </div>

                <div style="background: #f8fafc; border-radius: 8px; padding: 16px; margin-top: 24px; text-align: center; font-size: 14px; color: #64748b;">
                    如有任何問題，請聯繫我們<br>
                    📞 電話：037-433789<br>
                    📧 Email：{from_email}
                </div>
            </div>
            <div style="background: #f8fafc; padding: 24px; text-align: center; border-top: 1px solid #e2e8f0; border-radius: 0 0 16px 16px;">
                <p style="margin: 4px 0; font-size: 12px; color: #94a3b8;">此信件由系統自動發送，請勿直接回覆</p>
                <p style="margin: 4px 0; font-size: 12px; color: #64748b;">© 2026 豬博士動物科技有限公司</p>
            </div>
        </div>
    </div>
</body>
</html>"#,
            invite_link = invite_link,
            expires_at = expires_at_formatted,
            from_email = config.smtp_from_email,
        );

        let plain_body = format!(
            r#"[豬博士] 邀請您加入實驗動物管理平台

您好，

豬博士動物科技邀請您加入實驗動物管理平台。

透過此平台，您可以：
• 線上提交動物實驗計劃書（AUP）
• 即時追蹤審查進度
• 與審查委員線上溝通

請點擊以下連結完成註冊（連結僅限一次使用）：
{invite_link}

⏰ 此連結將於 {expires_at} 到期。

如有任何問題，請聯繫我們：
電話：037-433789
Email：{from_email}

豬博士動物科技有限公司"#,
            invite_link = invite_link,
            expires_at = expires_at_formatted,
            from_email = config.smtp_from_email,
        );

        Self::send_email(
            config,
            smtp_host,
            to_email,
            to_email,
            "[豬博士] 邀請您加入實驗動物管理平台",
            &plain_body,
            &html_body,
        )
        .await?;

        tracing::info!("Invitation email sent to {}", to_email);
        Ok(())
    }
}
