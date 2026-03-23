// Email - 設備維護相關通知

use crate::services::system_settings::SmtpConfig;

use super::EmailService;

impl EmailService {
    /// 寄送設備校正/確效逾期提醒
    pub async fn send_equipment_overdue_email(
        smtp: &SmtpConfig,
        to_email: &str,
        display_name: &str,
        overdue_items_html: &str,
        overdue_count: usize,
    ) -> anyhow::Result<()> {
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
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="cid:logo" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>⚠️ 設備校正/確效逾期提醒</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>以下 <strong>{overdue_count}</strong> 項設備的校正/確效/查核已逾期，請儘速安排處理。</p>
            {overdue_items_html}
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
        </div>
    </div>
</body>
</html>"#,
        );

        let plain_body = format!(
            "設備校正/確效逾期提醒\n\n共 {} 項設備逾期，請登入系統處理。",
            overdue_count
        );

        Self::send_email_smtp(
            smtp,
            to_email,
            display_name,
            &format!("[iPig] 設備校正/確效逾期提醒 ({} 項)", overdue_count),
            &plain_body,
            &html_body,
        )
        .await
    }

    /// 寄送設備無法維修通知
    pub async fn send_equipment_unrepairable_email(
        smtp: &SmtpConfig,
        to_email: &str,
        display_name: &str,
        equipment_name: &str,
        serial_number: &str,
        problem: &str,
    ) -> anyhow::Result<()> {
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
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="cid:logo" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>🔧 設備無法維修通知</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>以下設備經維修後判定<strong>無法修復</strong>，請安排後續報廢流程。</p>
            <div class="info-box">
                <p><strong>設備名稱：</strong> {equipment_name}</p>
                <p><strong>序號：</strong> {serial_number}</p>
                <p><strong>問題描述：</strong> {problem}</p>
            </div>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
        </div>
    </div>
</body>
</html>"#,
        );

        let plain_body = format!(
            "設備無法維修通知\n\n設備「{}」（序號：{}）經維修後判定無法修復。\n問題描述：{}\n\n請安排後續報廢流程。",
            equipment_name, serial_number, problem
        );

        Self::send_email_smtp(
            smtp,
            to_email,
            display_name,
            &format!("[iPig] 設備無法維修 - {}", equipment_name),
            &plain_body,
            &html_body,
        )
        .await
    }

    /// 寄送設備報廢申請通知
    pub async fn send_equipment_disposal_email(
        smtp: &SmtpConfig,
        to_email: &str,
        display_name: &str,
        equipment_name: &str,
        applicant_name: &str,
        reason: &str,
    ) -> anyhow::Result<()> {
        let html_body = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body {{ font-family: 'Microsoft JhengHei', Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #f59e0b; color: white; padding: 20px; text-align: center; border-radius: 8px 8px 0 0; }}
        .content {{ background: #f8fafc; padding: 30px; border: 1px solid #e2e8f0; }}
        .info-box {{ background: white; padding: 20px; border-radius: 8px; margin: 20px 0; border-left: 4px solid #f59e0b; }}
        .footer {{ text-align: center; padding: 20px; color: #64748b; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <div style="text-align: center; margin-bottom: 15px;">
                <img src="cid:logo" alt="iPig System" style="height: 50px; width: auto; background: white; padding: 5px; border-radius: 5px;">
            </div>
            <h1>📋 設備報廢申請</h1>
        </div>
        <div class="content">
            <p>親愛的 <strong>{display_name}</strong>，您好！</p>
            <p>有一項設備報廢申請待您核准。</p>
            <div class="info-box">
                <p><strong>設備名稱：</strong> {equipment_name}</p>
                <p><strong>申請人：</strong> {applicant_name}</p>
                <p><strong>報廢原因：</strong> {reason}</p>
            </div>
        </div>
        <div class="footer">
            <p>此信件由系統自動發送，請勿直接回覆。</p>
        </div>
    </div>
</body>
</html>"#,
        );

        let plain_body = format!(
            "設備報廢申請\n\n設備「{}」由 {} 申請報廢。\n原因：{}\n\n請登入系統核准或駁回。",
            equipment_name, applicant_name, reason
        );

        Self::send_email_smtp(
            smtp,
            to_email,
            display_name,
            &format!("[iPig] 設備報廢申請 - {}", equipment_name),
            &plain_body,
            &html_body,
        )
        .await
    }
}
