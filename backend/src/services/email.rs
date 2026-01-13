use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::config::Config;

pub struct EmailService;

impl EmailService {
    /// 寄送歡迎信給新用戶
    pub async fn send_welcome_email(
        config: &Config,
        to_email: &str,
        display_name: &str,
        password: &str,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping welcome email to {}", to_email);
            return Ok(());
        }

        let smtp_host = config.smtp_host.as_ref().unwrap();
        let login_url = format!("{}/login", config.app_url);
        let logo_url = format!("{}/pigmodel-logo.png", config.app_url);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {{ margin: 0; padding: 0; background-color: #f1f5f9; font-family: 'Microsoft JhengHei', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif; line-height: 1.6; color: #334155; }}
        .wrapper {{ padding: 40px 20px; }}
        .container {{ max-width: 600px; margin: 0 auto; background: white; border-radius: 16px; overflow: hidden; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06); }}
        .header {{ background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%); color: white; padding: 32px 24px; text-align: center; }}
        .header h1 {{ margin: 0; font-size: 24px; font-weight: 600; }}
        .header .subtitle {{ margin-top: 8px; font-size: 14px; opacity: 0.9; }}
        .logo {{ font-size: 48px; margin-bottom: 12px; }}
        .content {{ padding: 32px 24px; }}
        .greeting {{ font-size: 16px; margin-bottom: 16px; }}
        .info-box {{ background: linear-gradient(135deg, #f8fafc 0%, #f1f5f9 100%); padding: 24px; border-radius: 12px; margin: 24px 0; border-left: 4px solid #3b82f6; }}
        .info-box p {{ margin: 8px 0; font-size: 15px; }}
        .info-box .label {{ color: #64748b; font-size: 13px; }}
        .info-box .value {{ font-weight: 600; color: #1e293b; }}
        .button-container {{ text-align: center; margin: 32px 0; }}
        .button {{ display: inline-block; background: linear-gradient(135deg, #1e40af 0%, #3b82f6 100%); color: #ffffff !important; padding: 14px 32px; text-decoration: none; border-radius: 8px; font-weight: 600; font-size: 15px; transition: transform 0.2s; }}
        .warning {{ background: #fef2f2; border-radius: 8px; padding: 16px; margin: 24px 0; border-left: 4px solid #ef4444; }}
        .warning p {{ margin: 0; color: #991b1b; font-size: 14px; }}
        .contact {{ background: #f8fafc; border-radius: 8px; padding: 16px; margin-top: 24px; text-align: center; font-size: 14px; color: #64748b; }}
        .footer {{ background: #f8fafc; padding: 24px; text-align: center; border-top: 1px solid #e2e8f0; }}
        .footer p {{ margin: 4px 0; font-size: 12px; color: #94a3b8; }}
        .footer .company {{ font-weight: 500; color: #64748b; }}
    </style>
</head>
<body>
    <div class="wrapper">
        <div class="container">
            <div class="header">
                <div class="logo">
                    <img src="{logo_url}" alt="iPig System" style="height: 64px; width: auto; margin-bottom: 12px;">
                </div>
                <h1>歡迎加入豬博士 iPig 系統</h1>
                <p class="subtitle">您的帳號已成功開通</p>
            </div>
            <div class="content">
                <p class="greeting">親愛的 <strong>{display_name}</strong>，您好！</p>
                <p>您的豬博士 iPig 系統帳號已開通。以下是您的登入資訊：</p>
                
                <div class="info-box">
                    <p><span class="label">📧 帳號（Email）</span><br><span class="value">{to_email}</span></p>
                    <p><span class="label">🔑 初始密碼</span><br><span class="value">{password}</span></p>
                </div>
                
                <div class="warning">
                    <p>⚠️ 為確保帳號安全，請於首次登入後立即變更密碼。</p>
                </div>
                
                <div class="button-container">
                    <a href="{login_url}" class="button">立即登入系統</a>
                </div>
                
                <div class="contact">
                    如有任何問題，請聯繫工作人員<br>
                    📞 電話：037-433789
                </div>
            </div>
            <div class="footer">
                <p>此信件由系統自動發送，請勿直接回覆。</p>
                <p class="company">© 2026 豬博士動物科技有限公司</p>
            </div>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name,
            to_email = to_email,
            password = password,
            login_url = login_url,
            logo_url = logo_url,
        );

        let plain_body = format!(
            r#"歡迎加入豬博士 iPig 系統

親愛的 {display_name}，您好！

您的豬博士 iPig 系統帳號已開通。以下是您的登入資訊：

📧 帳號（Email）：{to_email}
🔑 初始密碼：{password}

⚠️ 為確保帳號安全，請於首次登入後立即變更密碼。

登入網址：{login_url}

如有任何問題，請聯繫工作人員（電話：037-433789）。

此信件由系統自動發送，請勿直接回覆。
© 2026 豬博士動物科技有限公司"#,
            display_name = display_name,
            to_email = to_email,
            password = password,
            login_url = login_url,
        );

        Self::send_email(config, smtp_host, to_email, display_name, 
            "🐷 歡迎加入豬博士 iPig 系統 - 您的帳號已開通",
            &plain_body, &html_body).await?;

        tracing::info!("Welcome email sent to {}", to_email);
        Ok(())
    }

    /// 寄送密碼重設信
    pub async fn send_password_reset_email(
        config: &Config,
        to_email: &str,
        display_name: &str,
        reset_token: &str,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping password reset email to {}", to_email);
            return Ok(());
        }

        let smtp_host = config.smtp_host.as_ref().unwrap();
        let reset_url = format!("{}/reset-password?token={}", config.app_url, reset_token);
        let logo_url = format!("{}/pigmodel-logo.png", config.app_url);

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
                <img src="{logo_url}" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>🔑 密碼重設通知</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>我們收到您重設密碼的請求。請點擊下方按鈕重設您的密碼：</p>
            
            <center>
                <a href="{reset_url}" class="button">重設密碼</a>
            </center>
            
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
            display_name = display_name,
            reset_url = reset_url,
            logo_url = logo_url,
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
            display_name = display_name,
            reset_url = reset_url,
        );

        Self::send_email(config, smtp_host, to_email, display_name,
            "🔑 豬博士 iPig 系統 - 密碼重設通知",
            &plain_body, &html_body).await?;

        tracing::info!("Password reset email sent to {}", to_email);
        Ok(())
    }

    /// 寄送密碼變更成功通知
    pub async fn send_password_changed_email(
        config: &Config,
        to_email: &str,
        display_name: &str,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping password changed email to {}", to_email);
            return Ok(());
        }

        let smtp_host = config.smtp_host.as_ref().unwrap();
        let logo_url = format!("{}/pigmodel-logo.png", config.app_url);

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
                <img src="{logo_url}" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>✅ 密碼變更成功</h1>
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
            logo_url = logo_url,
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

    /// 寄送計畫提交通知（給 IACUC_STAFF）
    pub async fn send_protocol_submitted_email(
        config: &Config,
        to_email: &str,
        display_name: &str,
        protocol_no: &str,
        protocol_title: &str,
        pi_name: &str,
        submitted_at: &str,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping protocol submitted email to {}", to_email);
            return Ok(());
        }

        let smtp_host = config.smtp_host.as_ref().unwrap();
        let protocol_url = format!("{}/protocols", config.app_url);
        let logo_url = format!("{}/pigmodel-logo.png", config.app_url);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Microsoft JhengHei', Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #2563eb; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ background: #f8fafc; padding: 30px; border: 1px solid #e2e8f0; }}
        .info-box {{ background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border-left: 4px solid #2563eb; }}
        .button {{ display: inline-block; background: #2563eb; color: white; padding: 12px 30px; text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="{logo_url}" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>📋 新計畫提交通知</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>有新計畫已提交，請進行行政預審。</p>
            
            <div class="info-box">
                <p><strong>計畫編號：</strong> {protocol_no}</p>
                <p><strong>計畫名稱：</strong> {protocol_title}</p>
                <p><strong>計畫主持人：</strong> {pi_name}</p>
                <p><strong>提交時間：</strong> {submitted_at}</p>
            </div>
            
            <center>
                <a href="{protocol_url}" class="button">登入系統處理</a>
            </center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name,
            protocol_no = protocol_no,
            protocol_title = protocol_title,
            pi_name = pi_name,
            submitted_at = submitted_at,
            protocol_url = protocol_url,
            logo_url = logo_url,
        );

        let plain_body = format!(
            r#"新計畫提交通知

親愛的 {display_name}，您好！

有新計畫已提交，請進行行政預審。

【計畫資訊】
計畫編號：{protocol_no}
計畫名稱：{protocol_title}
計畫主持人：{pi_name}
提交時間：{submitted_at}

請登入系統處理：{protocol_url}

此信件由系統自動發送，請勿直接回覆。
© 2026 豬博士動物科技有限公司"#,
            display_name = display_name,
            protocol_no = protocol_no,
            protocol_title = protocol_title,
            pi_name = pi_name,
            submitted_at = submitted_at,
            protocol_url = protocol_url,
        );

        Self::send_email(
            config,
            smtp_host,
            to_email,
            display_name,
            &format!("[iPig] 新計畫提交 - {}", protocol_no),
            &plain_body,
            &html_body,
        )
        .await?;

        tracing::info!("Protocol submitted email sent to {}", to_email);
        Ok(())
    }

    /// 寄送計畫狀態變更通知
    pub async fn send_protocol_status_change_email(
        config: &Config,
        to_email: &str,
        display_name: &str,
        protocol_no: &str,
        protocol_title: &str,
        new_status: &str,
        changed_at: &str,
        reason: Option<&str>,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!(
                "Email disabled, skipping protocol status change email to {}",
                to_email
            );
            return Ok(());
        }

        let smtp_host = config.smtp_host.as_ref().unwrap();
        let protocol_url = format!("{}/my-projects", config.app_url);
        let logo_url = format!("{}/pigmodel-logo.png", config.app_url);

        let reason_section = reason
            .map(|r| format!("<p><strong>變更原因：</strong> {}</p>", r))
            .unwrap_or_default();

        let reason_plain = reason
            .map(|r| format!("變更原因：{}", r))
            .unwrap_or_default();

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Microsoft JhengHei', Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #7c3aed; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ background: #f8fafc; padding: 30px; border: 1px solid #e2e8f0; }}
        .info-box {{ background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border-left: 4px solid #7c3aed; }}
        .button {{ display: inline-block; background: #7c3aed; color: white; padding: 12px 30px; text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
        .status {{ font-size: 18px; font-weight: bold; color: #7c3aed; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="{logo_url}" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>📝 計畫狀態更新通知</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>您的計畫狀態已更新。</p>
            
            <div class="info-box">
                <p><strong>計畫編號：</strong> {protocol_no}</p>
                <p><strong>計畫名稱：</strong> {protocol_title}</p>
                <p><strong>新狀態：</strong> <span class="status">{new_status}</span></p>
                <p><strong>變更時間：</strong> {changed_at}</p>
                {reason_section}
            </div>
            
            <center>
                <a href="{protocol_url}" class="button">登入系統查看</a>
            </center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name,
            protocol_no = protocol_no,
            protocol_title = protocol_title,
            new_status = new_status,
            changed_at = changed_at,
            reason_section = reason_section,
            protocol_url = protocol_url,
            logo_url = logo_url,
        );

        let plain_body = format!(
            r#"計畫狀態更新通知

親愛的 {display_name}，您好！

您的計畫狀態已更新。

【計畫資訊】
計畫編號：{protocol_no}
計畫名稱：{protocol_title}
新狀態：{new_status}
變更時間：{changed_at}
{reason_plain}

請登入系統查看：{protocol_url}

此信件由系統自動發送，請勿直接回覆。
© 2026 豬博士動物科技有限公司"#,
            display_name = display_name,
            protocol_no = protocol_no,
            protocol_title = protocol_title,
            new_status = new_status,
            changed_at = changed_at,
            reason_plain = reason_plain,
            protocol_url = protocol_url,
        );

        Self::send_email(
            config,
            smtp_host,
            to_email,
            display_name,
            &format!("[iPig] 計畫狀態更新 - {}", protocol_no),
            &plain_body,
            &html_body,
        )
        .await?;

        tracing::info!("Protocol status change email sent to {}", to_email);
        Ok(())
    }

    /// 寄送審查指派通知
    pub async fn send_review_assignment_email(
        config: &Config,
        to_email: &str,
        display_name: &str,
        protocol_no: &str,
        protocol_title: &str,
        pi_name: &str,
        due_date: Option<&str>,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!(
                "Email disabled, skipping review assignment email to {}",
                to_email
            );
            return Ok(());
        }

        let smtp_host = config.smtp_host.as_ref().unwrap();
        let protocol_url = format!("{}/protocols", config.app_url);
        let logo_url = format!("{}/pigmodel-logo.png", config.app_url);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Microsoft JhengHei', Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #ea580c; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ background: #f8fafc; padding: 30px; border: 1px solid #e2e8f0; }}
        .info-box {{ background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border-left: 4px solid #ea580c; }}
        .button {{ display: inline-block; background: #ea580c; color: white; padding: 12px 30px; text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
        .warning {{ color: #ea580c; font-weight: bold; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="{logo_url}" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>👁️ 審查指派通知</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>您已被指派審查以下計畫，請於期限內完成審查。</p>
            
            <div class="info-box">
                <p><strong>計畫編號：</strong> {protocol_no}</p>
                <p><strong>計畫名稱：</strong> {protocol_title}</p>
                <p><strong>計畫主持人：</strong> {pi_name}</p>
                <p><strong>審查期限：</strong> <span class="warning">{due_date}</span></p>
            </div>
            
            <center>
                <a href="{protocol_url}" class="button">登入系統審查</a>
            </center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name,
            protocol_no = protocol_no,
            protocol_title = protocol_title,
            pi_name = pi_name,
            due_date = due_date.unwrap_or("待定"),
            protocol_url = protocol_url,
            logo_url = logo_url,
        );

        let plain_body = format!(
            r#"審查指派通知

親愛的 {display_name}，您好！

您已被指派審查以下計畫，請於期限內完成審查。

【計畫資訊】
計畫編號：{protocol_no}
計畫名稱：{protocol_title}
計畫主持人：{pi_name}
審查期限：{due_date}

請登入系統審查：{protocol_url}

此信件由系統自動發送，請勿直接回覆。
© 2026 豬博士動物科技有限公司"#,
            display_name = display_name,
            protocol_no = protocol_no,
            protocol_title = protocol_title,
            pi_name = pi_name,
            due_date = due_date.unwrap_or("待定"),
            protocol_url = protocol_url,
        );

        Self::send_email(
            config,
            smtp_host,
            to_email,
            display_name,
            &format!("[iPig] 審查指派 - {}", protocol_no),
            &plain_body,
            &html_body,
        )
        .await?;

        tracing::info!("Review assignment email sent to {}", to_email);
        Ok(())
    }

    /// 寄送獸醫師建議通知
    pub async fn send_vet_recommendation_email(
        config: &Config,
        to_email: &str,
        display_name: &str,
        ear_tag: &str,
        iacuc_no: Option<&str>,
        record_type: &str,
        recommendation_content: &str,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!(
                "Email disabled, skipping vet recommendation email to {}",
                to_email
            );
            return Ok(());
        }

        let smtp_host = config.smtp_host.as_ref().unwrap();
        let pigs_url = format!("{}/pigs", config.app_url);
        let logo_url = format!("{}/pigmodel-logo.png", config.app_url);

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Microsoft JhengHei', Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #059669; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ background: #f8fafc; padding: 30px; border: 1px solid #e2e8f0; }}
        .info-box {{ background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border-left: 4px solid #059669; }}
        .recommendation {{ background: #ecfdf5; padding: 15px; border-radius: 8px; margin: 15px 0; }}
        .button {{ display: inline-block; background: #059669; color: white; padding: 12px 30px; text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="{logo_url}" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>🩺 獸醫師建議通知</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>獸醫師已對以下豬隻新增照護建議，請查閱並執行。</p>
            
            <div class="info-box">
                <p><strong>耳號：</strong> {ear_tag}</p>
                <p><strong>IACUC NO.：</strong> {iacuc_no}</p>
                <p><strong>紀錄類型：</strong> {record_type}</p>
            </div>
            
            <div class="recommendation">
                <p><strong>建議內容：</strong></p>
                <p>{recommendation_content}</p>
            </div>
            
            <center>
                <a href="{pigs_url}" class="button">登入系統查看</a>
            </center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name,
            ear_tag = ear_tag,
            iacuc_no = iacuc_no.unwrap_or("-"),
            record_type = record_type,
            recommendation_content = recommendation_content,
            pigs_url = pigs_url,
            logo_url = logo_url,
        );

        let plain_body = format!(
            r#"獸醫師建議通知

親愛的 {display_name}，您好！

獸醫師已對以下豬隻新增照護建議，請查閱並執行。

【豬隻資訊】
耳號：{ear_tag}
IACUC NO.：{iacuc_no}
紀錄類型：{record_type}

【建議內容】
{recommendation_content}

請登入系統查看：{pigs_url}

此信件由系統自動發送，請勿直接回覆。
© 2026 豬博士動物科技有限公司"#,
            display_name = display_name,
            ear_tag = ear_tag,
            iacuc_no = iacuc_no.unwrap_or("-"),
            record_type = record_type,
            recommendation_content = recommendation_content,
            pigs_url = pigs_url,
        );

        Self::send_email(
            config,
            smtp_host,
            to_email,
            display_name,
            &format!("[iPig] 獸醫師建議 - 耳號 {}", ear_tag),
            &plain_body,
            &html_body,
        )
        .await?;

        tracing::info!("Vet recommendation email sent to {}", to_email);
        Ok(())
    }

    /// 寄送低庫存提醒
    pub async fn send_low_stock_alert_email(
        config: &Config,
        to_email: &str,
        display_name: &str,
        alerts_html: &str,
        alert_count: usize,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping low stock alert email to {}", to_email);
            return Ok(());
        }

        let smtp_host = config.smtp_host.as_ref().unwrap();
        let inventory_url = format!("{}/inventory", config.app_url);
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let logo_url = format!("{}/pigmodel-logo.png", config.app_url);

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
        .alert-table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        .alert-table th, .alert-table td {{ border: 1px solid #e2e8f0; padding: 10px; text-align: left; }}
        .alert-table th {{ background: #fef2f2; }}
        .button {{ display: inline-block; background: #dc2626; color: white; padding: 12px 30px; text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="{logo_url}" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>⚠️ 低庫存提醒</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>以下 <strong>{alert_count}</strong> 項品項庫存已低於安全庫存，請安排補貨。</p>
            
            {alerts_html}
            
            <center>
                <a href="{inventory_url}" class="button">登入系統處理</a>
            </center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name,
            alert_count = alert_count,
            alerts_html = alerts_html,
            inventory_url = inventory_url,
            logo_url = logo_url,
        );

        let plain_body = format!(
            "低庫存提醒\n\n共 {} 項品項需要補貨，請登入系統查看。\n\n{}",
            alert_count, inventory_url
        );

        Self::send_email(
            config,
            smtp_host,
            to_email,
            display_name,
            &format!("[iPig] 低庫存提醒 - {}", today),
            &plain_body,
            &html_body,
        )
        .await?;

        tracing::info!("Low stock alert email sent to {}", to_email);
        Ok(())
    }

    /// 寄送效期提醒
    pub async fn send_expiry_alert_email(
        config: &Config,
        to_email: &str,
        display_name: &str,
        alerts_html: &str,
        expired_count: usize,
        expiring_count: usize,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping expiry alert email to {}", to_email);
            return Ok(());
        }

        let smtp_host = config.smtp_host.as_ref().unwrap();
        let inventory_url = format!("{}/inventory", config.app_url);
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let logo_url = format!("{}/pigmodel-logo.png", config.app_url);

        let is_urgent = expired_count > 0;
        let header_bg = if is_urgent { "#dc2626" } else { "#f59e0b" };
        let subject_prefix = if is_urgent { "[緊急]" } else { "" };

        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Microsoft JhengHei', Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: {header_bg}; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ background: #f8fafc; padding: 30px; border: 1px solid #e2e8f0; }}
        .alert-table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        .alert-table th, .alert-table td {{ border: 1px solid #e2e8f0; padding: 10px; text-align: left; }}
        .alert-table th {{ background: #fef3c7; }}
        .expired {{ color: #dc2626; font-weight: bold; }}
        .expiring {{ color: #f59e0b; }}
        .button {{ display: inline-block; background: {header_bg}; color: white; padding: 12px 30px; text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="{logo_url}" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>⏰ 效期提醒</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>以下品項即將到期或已過期，請注意處理。</p>
            <ul>
                <li><span class="expired">已過期：{expired_count} 項</span></li>
                <li><span class="expiring">即將到期（30天內）：{expiring_count} 項</span></li>
            </ul>
            
            {alerts_html}
            
            <center>
                <a href="{inventory_url}" class="button">登入系統處理</a>
            </center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            header_bg = header_bg,
            display_name = display_name,
            expired_count = expired_count,
            expiring_count = expiring_count,
            alerts_html = alerts_html,
            inventory_url = inventory_url,
            logo_url = logo_url,
        );

        let plain_body = format!(
            "效期提醒\n\n已過期：{} 項\n即將到期：{} 項\n\n請登入系統處理：{}",
            expired_count, expiring_count, inventory_url
        );

        Self::send_email(
            config,
            smtp_host,
            to_email,
            display_name,
            &format!("{}[iPig] 效期提醒 - {}", subject_prefix, today),
            &plain_body,
            &html_body,
        )
        .await?;

        tracing::info!("Expiry alert email sent to {}", to_email);
        Ok(())
    }

    /// 通用發送郵件方法
    async fn send_email(
        config: &Config,
        smtp_host: &str,
        to_email: &str,
        to_name: &str,
        subject: &str,
        plain_body: &str,
        html_body: &str,
    ) -> anyhow::Result<()> {
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
