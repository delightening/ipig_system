// 效期月度快照與比較通知

type SnapshotRow = (
    String,
    String,
    Option<String>,
    chrono::NaiveDate,
    i16,
    rust_decimal::Decimal,
    String,
    String,
);

use crate::{
    error::AppError,
    models::{CreateNotificationRequest, NotificationType},
};

use super::NotificationService;

/// 月度快照比較結果
pub struct MonthlyDiff {
    /// 本月新出現（上月沒有）
    pub added: Vec<SnapshotItem>,
    /// 上月有但本月消失（已處理或清空）
    pub removed: Vec<SnapshotItem>,
    /// 本月仍持續（上月也有）
    pub continued: Vec<SnapshotItem>,
}

#[derive(Debug, Clone)]
pub struct SnapshotItem {
    pub sku: String,
    pub product_name: String,
    pub batch_no: Option<String>,
    pub expiry_date: chrono::NaiveDate,
    pub days_past: i16,
    pub on_hand_qty: rust_decimal::Decimal,
    pub base_uom: String,
    pub warehouse_code: String,
}

impl NotificationService {
    /// 拍攝本月效期月度快照（過期超過 threshold_days 的品項）
    pub async fn take_expiry_monthly_snapshot(
        &self,
        snapshot_ym: &str,
        threshold_days: i16,
    ) -> Result<usize, AppError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            WITH alerts AS (
                SELECT * FROM fn_expiry_alerts(60, 730)
                WHERE days_until_expiry < -($1::INT)
            )
            INSERT INTO expiry_monthly_snapshots
                (snapshot_ym, product_id, sku, product_name, warehouse_id, batch_no,
                 expiry_date, days_past, on_hand_qty, base_uom)
            SELECT
                $2, a.product_id, a.sku, a.product_name, a.warehouse_id, a.batch_no,
                a.expiry_date, (-a.days_until_expiry)::SMALLINT, a.on_hand_qty, a.base_uom
            FROM alerts a
            ON CONFLICT (snapshot_ym, product_id, warehouse_id, COALESCE(batch_no, ''))
            DO UPDATE SET
                days_past   = EXCLUDED.days_past,
                on_hand_qty = EXCLUDED.on_hand_qty
            RETURNING 1
            "#,
        )
        .bind(threshold_days)
        .bind(snapshot_ym)
        .fetch_all(&self.db)
        .await?
        .len() as i64;

        tracing::info!("[ExpiryMonthly] 快照 {} 完成：{} 筆", snapshot_ym, count);
        Ok(count as usize)
    }

    /// 比較兩個月份的快照，回傳 diff
    pub async fn compare_expiry_snapshots(
        &self,
        current_ym: &str,
        previous_ym: &str,
    ) -> Result<MonthlyDiff, AppError> {
        let current = self.fetch_snapshot_items(current_ym).await?;
        let previous = self.fetch_snapshot_items(previous_ym).await?;

        let prev_keys: std::collections::HashSet<_> = previous
            .iter()
            .map(Self::snapshot_key)
            .collect();
        let curr_keys: std::collections::HashSet<_> = current
            .iter()
            .map(Self::snapshot_key)
            .collect();

        let added: Vec<_> = current
            .iter()
            .filter(|i| !prev_keys.contains(&Self::snapshot_key(i)))
            .cloned()
            .collect();
        let removed: Vec<_> = previous
            .iter()
            .filter(|i| !curr_keys.contains(&Self::snapshot_key(i)))
            .cloned()
            .collect();
        let continued: Vec<_> = current
            .iter()
            .filter(|i| prev_keys.contains(&Self::snapshot_key(i)))
            .cloned()
            .collect();

        Ok(MonthlyDiff { added, removed, continued })
    }

    /// 發送月度效期比較通知給所有指定角色的使用者
    pub async fn send_monthly_expiry_comparison(
        &self,
        diff: &MonthlyDiff,
        current_ym: &str,
        threshold_days: i16,
    ) -> Result<usize, AppError> {
        let title = format!("【月度彙整】{} 效期逾期品項通知", current_ym);
        let content = Self::build_monthly_comparison_content(diff, current_ym, threshold_days);

        let user_ids: Vec<(uuid::Uuid,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT ur.user_id
            FROM user_roles ur
            JOIN roles r ON ur.role_id = r.id
            WHERE r.code IN (
                SELECT DISTINCT role_code
                FROM notification_routing
                WHERE event_type = 'expiry_alert' AND is_active = true
                  AND frequency = 'monthly'
            )
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        let mut count = 0;
        for (user_id,) in user_ids {
            let request = CreateNotificationRequest {
                user_id,
                notification_type: NotificationType::ExpiryWarning,
                title: title.clone(),
                content: Some(content.clone()),
                related_entity_type: Some("expiry_monthly".to_string()),
                related_entity_id: None,
            };
            self.create_notification(request).await?;
            count += 1;
        }

        Ok(count)
    }

    // ── 內部 helpers ──

    async fn fetch_snapshot_items(&self, ym: &str) -> Result<Vec<SnapshotItem>, AppError> {
        let rows: Vec<SnapshotRow> =
            sqlx::query_as(
                r#"
                SELECT ems.sku, ems.product_name, ems.batch_no, ems.expiry_date,
                       ems.days_past, ems.on_hand_qty, ems.base_uom, w.code
                FROM expiry_monthly_snapshots ems
                JOIN warehouses w ON ems.warehouse_id = w.id
                WHERE ems.snapshot_ym = $1
                ORDER BY ems.days_past DESC
                "#,
            )
            .bind(ym)
            .fetch_all(&self.db)
            .await?;

        Ok(rows.into_iter().map(|(sku, product_name, batch_no, expiry_date, days_past, on_hand_qty, base_uom, warehouse_code)| {
            SnapshotItem { sku, product_name, batch_no, expiry_date, days_past, on_hand_qty, base_uom, warehouse_code }
        }).collect())
    }

    fn snapshot_key(item: &SnapshotItem) -> String {
        format!(
            "{}|{}|{}",
            item.sku,
            item.batch_no.as_deref().unwrap_or(""),
            item.warehouse_code
        )
    }

    fn build_monthly_comparison_content(
        diff: &MonthlyDiff,
        current_ym: &str,
        threshold_days: i16,
    ) -> String {
        let mut s = format!(
            "【{} 效期逾期品項月度彙整】\n過期超過 {} 天的品項：\n",
            current_ym, threshold_days
        );

        if !diff.added.is_empty() {
            s.push_str(&format!("\n▲ 本月新增 {} 項：\n", diff.added.len()));
            for item in diff.added.iter().take(10) {
                s.push_str(&format!(
                    "  - {} [{}] 已過期 {} 天，庫存 {} {}\n",
                    item.product_name,
                    item.batch_no.as_deref().unwrap_or("-"),
                    item.days_past,
                    item.on_hand_qty,
                    item.base_uom
                ));
            }
        }
        if !diff.removed.is_empty() {
            s.push_str(&format!("\n▼ 本月已處理 {} 項：\n", diff.removed.len()));
            for item in diff.removed.iter().take(10) {
                s.push_str(&format!(
                    "  - {} [{}]\n",
                    item.product_name,
                    item.batch_no.as_deref().unwrap_or("-")
                ));
            }
        }
        s.push_str(&format!("\n持續逾期：{} 項", diff.continued.len()));
        s
    }
}

/// 計算上月 YYYY-MM 字串
pub fn previous_ym(current_ym: &str) -> Option<String> {
    let parts: Vec<&str> = current_ym.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    let year: i32 = parts[0].parse().ok()?;
    let month: u32 = parts[1].parse().ok()?;
    let (prev_year, prev_month) = if month == 1 {
        (year - 1, 12u32)
    } else {
        (year, month - 1)
    };
    Some(format!("{:04}-{:02}", prev_year, prev_month))
}

/// 取得本月 YYYY-MM 字串
pub fn current_ym() -> String {
    let now = chrono::Utc::now();
    format!("{:04}-{:02}", now.year(), now.month())
}

// for build_monthly_comparison_content
use chrono::Datelike;
