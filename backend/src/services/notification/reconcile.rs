// 置頂待辦對帳（安全網）
//
// 「待處理」清單的正確性完全取決於各業務流程有沒有在終態呼叫
// [`NotificationService::resolve_pinned_notifications`]。而終態路徑是手寫的——
// 每新增一條（撤回 / 作廢 / 刪除 / 轉單…）就多一次漏接機會，漏接的後果是
// 使用者的待辦永久卡死且**無法自行清除**（待辦依設計不可手動已讀）。
//
// 2026-08-07 事故即為此：巡場報告的 retract / soft delete 兩條路徑沒接，
// 兩則置頂通知綁在已軟刪的報告上卡了一個月。詳見
// `docs/design/features/notification-vs-action-required-2026-08-07.md`。
//
// 本模組是那層安全網：把「置頂中、但關聯實體已不存在或已在終態」的通知降級。
// 它**不改變**「使用者不可手動略過待辦」這個規則——降級的判斷來自業務實體的真實狀態，
// 不是使用者的意願。
//
// 同一份邏輯有兩個呼叫端：
// - 一次性修補：`cargo run --bin reconcile_pinned_notifications`
// - 定期對帳：scheduler（見 `services/scheduler.rs`）

use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::{error::AppError, models::PRIORITY_NORMAL};

use super::NotificationService;

/// 本模組有能力判定終態的 `related_entity_type`。
///
/// 與 [`NotificationService::find_orphan_pinned`] 的 UNION 分支必須一致 ——
/// 新增待辦類型時，這裡與那邊要同時改。`count_unknown_entity_types` 綁定此常數，
/// 因此漏改會表現為「該類型被算成 unknown」而非靜默不一致。
const KNOWN_ENTITY_TYPES: &[&str] = &["vet_patrol_reports", "document", "leave_request"];

/// 一筆待降級的置頂通知（供 dry-run 列印與 log）。
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OrphanPinnedRow {
    pub id: Uuid,
    /// ⚠️ **含個資，禁止寫進日誌。** 僅供操作者在終端機看的 CLI 輸出使用。
    ///
    /// 這已經不是預防性警語：`leave_request` 這一類的 title 是
    /// `[iPig] {申請人姓名} 指定您為職務代理人，待您確認`，**帶員工真實姓名**
    /// （見 `services/hr/leave.rs` 的 `submit_leave`）。另兩類目前不帶個資
    /// （巡場＝`[巡場報告] {日期} 需您填寫追蹤改善`、
    /// 單據＝`[iPig] 採購單未入庫提醒 - {單號}`）。
    ///
    /// 只要有人為了日誌好讀把 title 加進 scheduler 的 `warn!`（結構裡就有，
    /// 是很自然的一步），姓名就進 Loki——那是留存久、存取範圍比 DB 廣的地方。
    pub title: String,
    pub related_entity_type: String,
    pub related_entity_id: Option<Uuid>,
    /// 收件人的 **user_id**，刻意不帶 email。
    ///
    /// 本結構會被排程的 `warn!` 寫進營運日誌（Loki 留存久、存取範圍比 DB 廣），
    /// email 屬直接識別個人的資料，不應為了維運方便而長期落在日誌裡。
    /// 真的需要對應到人時，維運者用這個 id 查 `users` 表即可 —— 那是有存取控管的路徑。
    pub user_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 人類可讀的降級理由（實體已刪 / 已在終態 / 實體不存在）。
    pub reason: String,
}

/// 對帳結果。
#[derive(Debug, Default)]
pub struct ReconcileReport {
    /// 已降級（dry-run 時為「將被降級」）的列。
    pub resolved: Vec<OrphanPinnedRow>,
    /// 置頂中、**已判定為仍需使用者動作**的筆數。
    ///
    /// 已扣掉 [`Self::resolved`] 與 [`Self::unknown_entity_types`] ——
    /// 後者是「未做判斷」而非「判定為合法」，混進來會讓這個數字被高估，
    /// 使維運者誤以為所有剩餘置頂列都已驗證過。
    pub still_pending: i64,
    /// 本模組不認得其 entity_type、因此未做判斷的筆數（保守不動）。
    pub unknown_entity_types: Vec<(String, i64)>,
    /// `related_entity_id IS NULL` 的置頂列筆數（保守不動）。
    ///
    /// 這類列沒有可供追溯的業務實體，本模組**無從判斷**它是否還需要使用者動作 ——
    /// 不是「實體不存在」。必須單獨列出，否則它們會永遠靜靜地留在待處理清單裡
    /// 而沒有人知道原因。已知來源：2026-03 的舊聚合式採購提醒。
    pub null_entity_id: i64,
    /// UPDATE 實際影響的列數。dry-run 時為 0。
    ///
    /// 與 `resolved.len()` 可能不同：UPDATE 帶 `priority > 0` 條件，
    /// 併發下可能有列已被業務路徑先解除。回報時用這個數字，別用候選數。
    pub resolved_rows_affected: u64,
}

/// 單次自動對帳允許降級的上限。超過即中止、不寫入，改由人工確認。
///
/// 依據：2026-08-07 prod 實查全系統置頂待辦僅 7 筆。正常運作下命中數是個位數；
/// 一次要動數百列必然是判定條件壞了或資料異常，那種情況下「照做」比「停下」危險得多 ——
/// 這支作業會靜默改使用者的待辦狀態，而誤降的方向同樣讓使用者無法自救。
const MAX_AUTO_RESOLVE_BATCH: usize = 200;

impl NotificationService {
    /// 對帳所有置頂待辦，降級「關聯實體已不存在或已在終態」者。
    ///
    /// `dry_run = true` 時只查不寫，供上 prod 前核對筆數。
    ///
    /// **保守原則**：只降級能明確證明「不再需要使用者動作」的列。
    /// 認不得的 `related_entity_type` 一律不動，只在報告中列出——
    /// 寧可留下一筆多餘待辦，也不要誤清掉真正待處理的事項。
    pub async fn reconcile_pinned_notifications(
        &self,
        dry_run: bool,
    ) -> Result<ReconcileReport, AppError> {
        // 四個統計必須看同一個快照，否則期間有列被業務路徑解除時，
        // `count_pinned` 變小而候選數不變 → still_pending 算出來不對應任何真實時點。
        //
        // ⚠️ 單純 `BEGIN` **不夠**：PostgreSQL 預設 READ COMMITTED，
        // 同一個 tx 內每個語句各自取新的 snapshot。必須顯式提升到 REPEATABLE READ，
        // 該 tx 的第一個查詢才會固定 snapshot 供後續語句共用。
        let mut tx = self.db.begin().await?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
            .execute(&mut *tx)
            .await?;
        let candidates = Self::find_orphan_pinned(&mut tx).await?;
        let unknown = Self::count_unknown_entity_types(&mut tx).await?;
        let null_entity_id = Self::count_null_entity_id(&mut tx).await?;
        let total_pinned = Self::count_pinned(&mut tx).await?;
        tx.commit().await?;

        let unknown_total: i64 = unknown.iter().map(|(_, n)| *n).sum();

        let mut report = ReconcileReport {
            // 未知 entity_type 與 NULL entity_id 的列都計在 count_pinned 內，
            // 但它們是「未做判斷」，不能算進「已判定仍需動作」。
            // 三者互斥：unknown 依定義不在候選類型內；NULL entity 已被候選查詢排除，
            // 而 count_unknown 只看 entity_type、只會數到「已知類型」以外的列。
            still_pending: (total_pinned
                - candidates.len() as i64
                - unknown_total
                - null_entity_id)
                .max(0),
            unknown_entity_types: unknown,
            null_entity_id,
            resolved: candidates,
            resolved_rows_affected: 0,
        };

        if !dry_run && !report.resolved.is_empty() {
            // 安全閘：這支作業每天自動改資料，判定條件一旦有誤就是靜默清掉真實待辦。
            // 正常情況命中數是個位數（2026-08-07 prod 全系統置頂僅 7 筆）；
            // 一次要動這麼多列必然是判定邏輯壞了或資料異常 —— 中止並讓它在日誌顯眼，
            // 由人決定，不要讓排程自己把事情做完。
            if report.resolved.len() > MAX_AUTO_RESOLVE_BATCH {
                tracing::error!(
                    candidates = report.resolved.len(),
                    limit = MAX_AUTO_RESOLVE_BATCH,
                    "置頂待辦對帳：命中數超過安全上限，已中止未寫入 —— 請人工確認判定條件是否正確"
                );
                return Err(AppError::BusinessRule(format!(
                    "對帳命中 {} 筆超過安全上限 {}，已中止未寫入。請以 --dry-run 檢視後人工確認。",
                    report.resolved.len(),
                    MAX_AUTO_RESOLVE_BATCH
                )));
            }

            let ids: Vec<Uuid> = report.resolved.iter().map(|r| r.id).collect();
            // **只降 priority，不碰 is_read / read_at。**
            //
            // 原本一併設 `is_read = true, read_at = COALESCE(read_at, NOW())`，
            // 那會踩到 cleanup_old_notifications（crud.rs）的 GC 條件
            // `is_read = true AND read_at < NOW() - INTERVAL '90 days'`，每週日執行。
            // COALESCE 會保留**舊的** read_at，而 prod 上的置頂列多半早已 is_read=true
            // （使用者按過「全部已讀」，read_at 是很久以前）—— 於是被對帳降級的列
            // 下一次 GC 就會被硬刪，誤降時連還原與追查的機會都沒有。
            // 只降 priority：列從「待處理」清單消失，但仍以一般通知留在鈴鐺與 DB 裡。
            let result = sqlx::query(
                r#"
                UPDATE notifications
                SET priority = $2
                WHERE id = ANY($1)
                  AND priority > $2
                "#,
            )
            .bind(&ids)
            .bind(PRIORITY_NORMAL)
            .execute(&self.db)
            .await?;

            // 回報實際更新列數而非候選數：UPDATE 帶 `priority > $2`，
            // 併發下可能有列已被業務路徑解除，候選數會高估。
            report.resolved_rows_affected = result.rows_affected();

            for row in &report.resolved {
                tracing::info!(
                    notification_id = %row.id,
                    entity_type = %row.related_entity_type,
                    reason = %row.reason,
                    "置頂待辦對帳：已降級孤兒待辦"
                );
            }
        }

        Ok(report)
    }

    async fn count_pinned(tx: &mut Transaction<'_, Postgres>) -> Result<i64, AppError> {
        Ok(
            sqlx::query_scalar("SELECT count(*) FROM notifications WHERE priority > 0")
                .fetch_one(&mut **tx)
                .await?,
        )
    }

    /// 列出本模組認不得的 entity_type（保守不動，但要讓維運者看見）。
    ///
    /// 已知類型清單以 [`KNOWN_ENTITY_TYPES`] 綁參數帶入 —— 與 `find_orphan_pinned`
    /// 的 UNION 分支共用同一份定義。若兩邊各寫一份，日後新增類型只改一邊時，
    /// 該類型會同時被算成候選與 unknown，`still_pending` 被重複扣而無人察覺。
    async fn count_unknown_entity_types(
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<(String, i64)>, AppError> {
        Ok(sqlx::query_as(
            r#"
            SELECT COALESCE(related_entity_type, '(null)'), count(*)
            FROM notifications
            WHERE priority > 0
              AND (related_entity_type IS NULL
                   OR related_entity_type <> ALL($1))
            GROUP BY 1
            ORDER BY 2 DESC
            "#,
        )
        .bind(KNOWN_ENTITY_TYPES)
        .fetch_all(&mut **tx)
        .await?)
    }

    /// 數「entity_type 已知、但 entity_id 為 NULL」的置頂列 —— 無從判斷，保守不動。
    ///
    /// 刻意排除未知 entity_type，讓本桶與 [`Self::count_unknown_entity_types`] 互斥；
    /// 否則同時符合兩者的列會被重複扣，`still_pending` 失真。
    async fn count_null_entity_id(tx: &mut Transaction<'_, Postgres>) -> Result<i64, AppError> {
        Ok(sqlx::query_scalar(
            r#"
            SELECT count(*)
            FROM notifications
            WHERE priority > 0
              AND related_entity_id IS NULL
              AND related_entity_type = ANY($1)
            "#,
        )
        .bind(KNOWN_ENTITY_TYPES)
        .fetch_one(&mut **tx)
        .await?)
    }

    /// 找出所有「置頂中但已不需要使用者動作」的通知。
    ///
    /// 每個 entity_type 的終態判定各自定義，集中在此一處，新增待辦類型時同步補一段。
    async fn find_orphan_pinned(
        tx: &mut Transaction<'_, Postgres>,
    ) -> Result<Vec<OrphanPinnedRow>, AppError> {
        Ok(sqlx::query_as::<_, OrphanPinnedRow>(
            r#"
            -- 巡場報告：報告已軟刪 / 已完成 / 已撤回 / row 根本不存在 → 追蹤者已無事可做。
            --
            -- 「已撤回」＝ status='draft' 且 follow_up_user_id IS NULL：置頂待辦只在
            -- submit_for_followup 建立（該處必定同時設 status='awaiting_acknowledgement'
            -- 與 follow_up_user_id），所以這個組合唯一的來源就是撤回。必須納入 ——
            -- 撤回正是 2026-08-07 事故的觸發路徑，若該處的解除將來回歸，
            -- 這道安全網要接得住。
            SELECT n.id, n.title, n.related_entity_type, n.related_entity_id,
                   n.user_id, n.created_at,
                   CASE
                     WHEN r.id IS NULL              THEN '關聯的巡場報告已不存在'
                     WHEN r.deleted_at IS NOT NULL  THEN '關聯的巡場報告已刪除'
                     WHEN r.status = 'completed'    THEN '關聯的巡場報告已完成'
                     ELSE                                '關聯的巡場報告已撤回（無指派追蹤者）'
                   END AS reason
            FROM notifications n
            LEFT JOIN vet_patrol_reports r ON r.id = n.related_entity_id
            WHERE n.priority > 0
              AND n.related_entity_type = 'vet_patrol_reports'
              -- entity_id 為 NULL＝無從判斷，不是「實體不存在」。少了這個條件，
              -- LEFT JOIN 會讓 r.id IS NULL 成立而無條件降級。見下方 NULL 桶。
              AND n.related_entity_id IS NOT NULL
              AND (r.id IS NULL
                   OR r.deleted_at IS NOT NULL
                   OR r.status = 'completed'
                   OR (r.status = 'draft' AND r.follow_up_user_id IS NULL))

            UNION ALL

            -- 單據：row 已不存在（硬刪）→ 無從入庫/簽核，待辦不可能再完成
            SELECT n.id, n.title, n.related_entity_type, n.related_entity_id,
                   n.user_id, n.created_at,
                   '關聯的單據已不存在' AS reason
            FROM notifications n
            WHERE n.priority > 0
              AND n.related_entity_type = 'document'
              -- 同上：`d.id = NULL` 恆為 NULL → NOT EXISTS 恆為真 → 無條件降級。
              -- 2026-08-07 prod 實查：7 筆置頂中有 4 筆正是 entity_id IS NULL
              -- （2026-03 的舊聚合式提醒「N 筆採購單待入庫」，與現行每張 PO 一則的
              -- 產生器不同，本來就不帶 entity id）。少了這個條件會把它們全清掉，
              -- 且理由字串是假的 —— 那條規則對「PO 到底收貨了沒」一個字都沒說。
              AND n.related_entity_id IS NOT NULL
              AND NOT EXISTS (SELECT 1 FROM documents d WHERE d.id = n.related_entity_id)

            UNION ALL

            -- 請假 ①：假單 row 已不存在（硬刪）→ 任何收件人都已無事可做。
            --
            -- 這一支刻意獨立於下面兩支：假單一旦不存在，`proxy_user_id` 也讀不到，
            -- 就無從判斷這則待辦原本是給代理人還是給審核人的。與其在兩支各寫一次
            -- `lr.id IS NULL`（其中一支還會因為 NULL 比較而漏掉），不如集中在這裡。
            SELECT n.id, n.title, n.related_entity_type, n.related_entity_id,
                   n.user_id, n.created_at,
                   '關聯的請假申請已不存在' AS reason
            FROM notifications n
            WHERE n.priority > 0
              AND n.related_entity_type = 'leave_request'
              -- entity_id 為 NULL＝無從判斷，不是「實體不存在」。見下方 NULL 桶。
              AND n.related_entity_id IS NOT NULL
              AND NOT EXISTS (
                    SELECT 1 FROM leave_requests lr WHERE lr.id = n.related_entity_id
                  )

            UNION ALL

            -- 請假 ②：**代理人**的待辦——假單已離開 PENDING_PROXY → 代理人已無事可做。
            --
            -- 這則置頂待辦只在 submit_leave 的非 director_self_report 分支建立，
            -- 收件人是 proxy_user_id，而該處必定同時把 status 設成 'PENDING_PROXY'
            -- （見 services/hr/leave.rs）。所以「仍需代理人動作」等價於「仍在
            -- PENDING_PROXY」——確認（→PENDING_L1/PENDING_DIRECTOR）、退回（→DRAFT）、
            -- 申請人取消（→CANCELLED）三條解除路徑都會離開這個狀態，任一條漏接時
            -- 這道安全網都接得住。
            --
            -- ⚠️ 過去這裡只憑「收件人是不是 proxy_user_id」判斷用途，代理人若剛好也是
            -- 下一關核准人（例如代理人＝申請人的單位主管，實務上不罕見）時會誤清
            -- confirm 後才建立、真正待處理的核准 pin（Qodo review 抓到，R92-1）。
            -- 148 改用 `recipient_role`（建立時寫死的用途標記，不受「收件人剛好是誰」
            -- 影響）當主要判準；`n.user_id = lr.proxy_user_id` 保留為防禦性檢查。
            SELECT n.id, n.title, n.related_entity_type, n.related_entity_id,
                   n.user_id, n.created_at,
                   '關聯的請假單已離開待代理確認狀態' AS reason
            FROM notifications n
            JOIN leave_requests lr ON lr.id = n.related_entity_id
            WHERE n.priority > 0
              AND n.related_entity_type = 'leave_request'
              AND n.related_entity_id IS NOT NULL
              AND n.recipient_role = 'proxy'
              AND n.user_id = lr.proxy_user_id
              AND lr.status <> 'PENDING_PROXY'::leave_status

            UNION ALL

            -- 請假 ③：**審核人**的待辦——假單已離開審核中的兩關 → 這一關的人已無事可做。
            --
            -- 只有 PENDING_L1 / PENDING_DIRECTOR 是「待這一關的人動作」的狀態
            -- （見 HrService::current_stage_approvers 的 match）；DRAFT、PENDING_PROXY、
            -- 終態（APPROVED/REJECTED/CANCELLED/REVOKED）對審核人而言皆已無事可做。
            --
            -- 用 `recipient_role = 'approver'` 取代舊的 `user_id IS DISTINCT FROM
            -- lr.proxy_user_id`（R92-1）：後者在代理人與核准人同一人時會把這則 pin
            -- 整個排除在③之外（③管不到、②又誤清），改用建立時寫死的用途標記後，
            -- 不論收件人剛好是誰，「這是核准 pin」的判斷都不會失準。
            SELECT n.id, n.title, n.related_entity_type, n.related_entity_id,
                   n.user_id, n.created_at,
                   '關聯的請假申請已離開審核關卡（狀態：' || lr.status::text || '）' AS reason
            FROM notifications n
            JOIN leave_requests lr ON lr.id = n.related_entity_id
            WHERE n.priority > 0
              AND n.related_entity_type = 'leave_request'
              AND n.related_entity_id IS NOT NULL
              AND n.recipient_role = 'approver'
              AND lr.status::text NOT IN ('PENDING_L1', 'PENDING_DIRECTOR')

            ORDER BY created_at
            "#,
        )
        .fetch_all(&mut **tx)
        .await?)
    }
}
