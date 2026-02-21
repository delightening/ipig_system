// Email - 警示與動物相關（獸醫建議、低庫存、效期、安樂死）

use crate::config::Config;
use super::EmailService;

impl EmailService {
    /// 寄送獸醫師建議通知
    pub async fn send_vet_recommendation_email(
        config: &Config, to_email: &str, display_name: &str,
        ear_tag: &str, iacuc_no: Option<&str>, record_type: &str,
        recommendation_content: &str, is_urgent: bool,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping vet recommendation email to {}", to_email);
            return Ok(());
        }
        let smtp_host = config.smtp_host.as_ref().expect("smtp_host 應由 is_email_enabled() 保證存在");
        let animals_url = format!("{}/animals", config.app_url);
        let logo_url = format!("{}/pigmodel-logo.png", config.app_url);

        let (header_bg, header_icon, urgency_note) = if is_urgent {
            ("#dc2626", "🚨",
             "<p style=\"color: #dc2626; font-weight: bold;\">⚠️ 此為緊急建議，請立即處理！</p>")
        } else {
            ("#059669", "🩺", "")
        };

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
        .info-box {{ background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border-left: 4px solid {header_bg}; }}
        .recommendation {{ background: #ecfdf5; padding: 15px; border-radius: 8px; margin: 15px 0; }}
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
            <h1>{header_icon} 獸醫師建議通知</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            {urgency_note}
            <p>獸醫師已對以下動物新增照護建議，請查閱並執行。</p>
            <div class="info-box">
                <p><strong>耳號：</strong> {ear_tag}</p>
                <p><strong>IACUC No.：</strong> {iacuc_no}</p>
                <p><strong>紀錄類型：</strong> {record_type}</p>
            </div>
            <div class="recommendation">
                <p><strong>建議內容：</strong></p>
                <p>{recommendation_content}</p>
            </div>
            <center><a href="{animals_url}" class="button">登入系統查看</a></center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            header_bg = header_bg, header_icon = header_icon,
            urgency_note = urgency_note, display_name = display_name,
            ear_tag = ear_tag, iacuc_no = iacuc_no.unwrap_or("-"),
            record_type = record_type, recommendation_content = recommendation_content,
            animals_url = animals_url, logo_url = logo_url,
        );

        let urgency_prefix = if is_urgent { "[緊急] " } else { "" };
        let plain_urgency = if is_urgent { "⚠️ 此為緊急建議，請立即處理！\n\n" } else { "" };

        let plain_body = format!(
            r#"獸醫師建議通知

親愛的 {display_name}，您好！

{plain_urgency}獸醫師已對以下動物新增照護建議，請查閱並執行。

【動物資訊】
耳號：{ear_tag}
IACUC No.：{iacuc_no}
紀錄類型：{record_type}

【建議內容】
{recommendation_content}

請登入系統查看：{animals_url}

此信件由系統自動發送，請勿直接回覆。
© 2026 豬博士動物科技有限公司"#,
            display_name = display_name, plain_urgency = plain_urgency,
            ear_tag = ear_tag, iacuc_no = iacuc_no.unwrap_or("-"),
            record_type = record_type, recommendation_content = recommendation_content,
            animals_url = animals_url,
        );

        Self::send_email(config, smtp_host, to_email, display_name,
            &format!("{}[iPig] 獸醫師建議 - 耳號 {}", urgency_prefix, ear_tag),
            &plain_body, &html_body).await?;

        tracing::info!("Vet recommendation email sent to {} (urgent: {})", to_email, is_urgent);
        Ok(())
    }

    /// 寄送低庫存提醒
    pub async fn send_low_stock_alert_email(
        config: &Config, to_email: &str, display_name: &str,
        alerts_html: &str, alert_count: usize,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping low stock alert email to {}", to_email);
            return Ok(());
        }
        let smtp_host = config.smtp_host.as_ref().expect("smtp_host 應由 is_email_enabled() 保證存在");
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
            <center><a href="{inventory_url}" class="button">登入系統處理</a></center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name, alert_count = alert_count,
            alerts_html = alerts_html, inventory_url = inventory_url, logo_url = logo_url,
        );

        let plain_body = format!(
            "低庫存提醒\n\n共 {} 項品項需要補貨，請登入系統查看。\n\n{}",
            alert_count, inventory_url
        );

        Self::send_email(config, smtp_host, to_email, display_name,
            &format!("[iPig] 低庫存提醒 - {}", today),
            &plain_body, &html_body).await?;

        tracing::info!("Low stock alert email sent to {}", to_email);
        Ok(())
    }

    /// 寄送效期提醒
    pub async fn send_expiry_alert_email(
        config: &Config, to_email: &str, display_name: &str,
        alerts_html: &str, expired_count: usize, expiring_count: usize,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping expiry alert email to {}", to_email);
            return Ok(());
        }
        let smtp_host = config.smtp_host.as_ref().expect("smtp_host 應由 is_email_enabled() 保證存在");
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
            <center><a href="{inventory_url}" class="button">登入系統處理</a></center>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            header_bg = header_bg, display_name = display_name,
            expired_count = expired_count, expiring_count = expiring_count,
            alerts_html = alerts_html, inventory_url = inventory_url, logo_url = logo_url,
        );

        let plain_body = format!(
            "效期提醒\n\n已過期：{} 項\n即將到期：{} 項\n\n請登入系統處理：{}",
            expired_count, expiring_count, inventory_url
        );

        Self::send_email(config, smtp_host, to_email, display_name,
            &format!("{}[iPig] 效期提醒 - {}", subject_prefix, today),
            &plain_body, &html_body).await?;

        tracing::info!("Expiry alert email sent to {}", to_email);
        Ok(())
    }

    /// 寄送安樂死執行通知給 PI
    pub async fn send_euthanasia_order_email(
        config: &Config, to_email: &str, display_name: &str,
        ear_tag: &str, iacuc_no: Option<&str>, vet_name: &str,
        reason: &str, deadline: &str,
    ) -> anyhow::Result<()> {
        if !config.is_email_enabled() {
            tracing::info!("Email disabled, skipping euthanasia order email to {}", to_email);
            return Ok(());
        }
        let smtp_host = config.smtp_host.as_ref().expect("smtp_host 應由 is_email_enabled() 保證存在");
        let animals_url = format!("{}/animals", config.app_url);
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
        .warning-box {{ background: #fef2f2; padding: 15px; border-radius: 8px; margin: 15px 0; border: 2px solid #dc2626; }}
        .button {{ display: inline-block; background: #dc2626; color: white; padding: 12px 30px; text-decoration: none; border-radius: 6px; margin: 10px 5px; }}
        .button-secondary {{ display: inline-block; background: #f59e0b; color: white; padding: 12px 30px; text-decoration: none; border-radius: 6px; margin: 10px 5px; }}
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
        .urgent {{ color: #dc2626; font-weight: bold; font-size: 18px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="{logo_url}" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>🚨 安樂死執行通知</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p class="urgent">您的計畫下的動物已被獸醫師開立安樂死單，請儘速處理。</p>
            <div class="info-box">
                <p><strong>耳號：</strong> {ear_tag}</p>
                <p><strong>IACUC No.：</strong> {iacuc_no}</p>
                <p><strong>開單獸醫：</strong> {vet_name}</p>
                <p><strong>安樂死原因：</strong> {reason}</p>
            </div>
            <div class="warning-box">
                <p class="urgent">⏰ 執行期限：{deadline}</p>
                <p>系統將於 <strong>24 小時</strong>後自動解鎖執行權限。若您未在期限內回應，獸醫師將可直接執行安樂死。</p>
            </div>
            <center>
                <a href="{animals_url}" class="button">同意執行</a>
                <a href="{animals_url}" class="button-secondary">申請暫緩</a>
            </center>
            <p style="color: #64748b; font-size: 12px; margin-top: 20px;">
                請登入系統選擇「同意執行」或「申請暫緩」。如有疑問，請聯繫獸醫師或 IACUC 辦公室。
            </p>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
            <p>© 2026 豬博士動物科技有限公司</p>
        </div>
    </div>
</body>
</html>"#,
            display_name = display_name, ear_tag = ear_tag,
            iacuc_no = iacuc_no.unwrap_or("-"), vet_name = vet_name,
            reason = reason, deadline = deadline,
            animals_url = animals_url, logo_url = logo_url,
        );

        let plain_body = format!(
            r#"🚨 安樂死執行通知

親愛的 {display_name}，您好！

您的計畫下的動物已被獸醫師開立安樂死單，請儘速處理。

【動物資訊】
耳號：{ear_tag}
IACUC No.：{iacuc_no}
開單獸醫：{vet_name}
安樂死原因：{reason}

⏰ 執行期限：{deadline}
系統將於 24 小時後自動解鎖執行權限。

請登入系統處理：{animals_url}

此信件由系統自動發送，請勿直接回覆。
© 2026 豬博士動物科技有限公司"#,
            display_name = display_name, ear_tag = ear_tag,
            iacuc_no = iacuc_no.unwrap_or("-"), vet_name = vet_name,
            reason = reason, deadline = deadline, animals_url = animals_url,
        );

        Self::send_email(config, smtp_host, to_email, display_name,
            &format!("[緊急] 動物 #{} 安樂死執行通知", ear_tag),
            &plain_body, &html_body).await?;

        tracing::info!("Euthanasia order email sent to {}", to_email);
        Ok(())
    }
}
