// Email - 計畫相關（提交、狀態變更、審查指派）

use crate::config::Config;
use super::EmailService;

impl EmailService {
    /// 寄送計畫提交通知（給 IACUC_STAFF）
    pub async fn send_protocol_submitted_email(
        config: &Config, to_email: &str, display_name: &str,
        protocol_no: &str, protocol_title: &str, pi_name: &str, submitted_at: &str,
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
            <center><a href="{protocol_url}" class="button">登入系統處理</a></center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name, protocol_no = protocol_no,
            protocol_title = protocol_title, pi_name = pi_name,
            submitted_at = submitted_at, protocol_url = protocol_url, logo_url = logo_url,
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
            display_name = display_name, protocol_no = protocol_no,
            protocol_title = protocol_title, pi_name = pi_name,
            submitted_at = submitted_at, protocol_url = protocol_url,
        );

        Self::send_email(config, smtp_host, to_email, display_name,
            &format!("[iPig] 新計畫提交 - {}", protocol_no),
            &plain_body, &html_body).await?;

        tracing::info!("Protocol submitted email sent to {}", to_email);
        Ok(())
    }

    /// 寄送計畫狀態變更通知
    pub async fn send_protocol_status_change_email(
        config: &Config, to_email: &str, display_name: &str,
        protocol_no: &str, protocol_title: &str,
        new_status: &str, changed_at: &str, reason: Option<&str>,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping protocol status change email to {}", to_email);
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
            <center><a href="{protocol_url}" class="button">登入系統查看</a></center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name, protocol_no = protocol_no,
            protocol_title = protocol_title, new_status = new_status,
            changed_at = changed_at, reason_section = reason_section,
            protocol_url = protocol_url, logo_url = logo_url,
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
            display_name = display_name, protocol_no = protocol_no,
            protocol_title = protocol_title, new_status = new_status,
            changed_at = changed_at, reason_plain = reason_plain,
            protocol_url = protocol_url,
        );

        Self::send_email(config, smtp_host, to_email, display_name,
            &format!("[iPig] 計畫狀態更新 - {}", protocol_no),
            &plain_body, &html_body).await?;

        tracing::info!("Protocol status change email sent to {}", to_email);
        Ok(())
    }

    /// 寄送審查指派通知
    pub async fn send_review_assignment_email(
        config: &Config, to_email: &str, display_name: &str,
        protocol_no: &str, protocol_title: &str, pi_name: &str, due_date: Option<&str>,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping review assignment email to {}", to_email);
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
            <center><a href="{protocol_url}" class="button">登入系統審查</a></center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name, protocol_no = protocol_no,
            protocol_title = protocol_title, pi_name = pi_name,
            due_date = due_date.unwrap_or("待定"),
            protocol_url = protocol_url, logo_url = logo_url,
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
            display_name = display_name, protocol_no = protocol_no,
            protocol_title = protocol_title, pi_name = pi_name,
            due_date = due_date.unwrap_or("待定"), protocol_url = protocol_url,
        );

        Self::send_email(config, smtp_host, to_email, display_name,
            &format!("[iPig] 審查指派 - {}", protocol_no),
            &plain_body, &html_body).await?;

        tracing::info!("Review assignment email sent to {}", to_email);
        Ok(())
    }
}
