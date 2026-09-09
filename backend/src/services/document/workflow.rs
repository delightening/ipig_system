use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::{Acquire, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    middleware::{ActorContext, CurrentUser},
    models::{
        audit_diff::{AuditRedact, DataDiff},
        DocStatus, DocType, Document, DocumentAuditSnapshot, DocumentLine, DocumentWithLines,
    },
    services::{
        audit::{ActivityLogEntry, AuditEntity},
        stock::StorageDriftEvent,
        AccountingService, AuditService, NotificationService, StockService,
    },
    AppError, Result,
};

use super::DocumentService;

/// GRN 軟擋稽核 payload：核准一張採購入庫單時，哪幾行未落貨架、合計未上架量。
/// 寫入 audit after_data，供 GLP 稽核追「誰在何時有意識地留下未分配庫存」。
#[derive(Serialize)]
struct GrnUnshelvedAudit {
    doc_no: String,
    unshelved_line_nos: Vec<i32>,
    unshelved_qty_total: Decimal,
}
impl AuditRedact for GrnUnshelvedAudit {}

/// 領用被儲位庫存擋下時：先確實收掉 transaction，再通知倉管與操作者，最後把原錯誤原樣傳回。
///
/// 🔴 順序是這個函式存在的**唯一**理由。擋下的當下 tx 一定會回滾，
/// 通知若寫在 tx 裡會跟著消失——所以必須先 rollback、再用 pool 另開連線寫通知。
///
/// 顯式 `rollback().await` 而不是靠 `drop`：drop 的回滾是丟到背景做的，
/// 不等它結束就去搶同一批資料的鎖，等於自己跟自己競爭。
///
/// 通知失敗只 warn 不改回傳：**操作者該看到的是庫存錯誤本身**。
/// 讓通知的失敗蓋掉它，等於用次要問題掩蓋主要問題。
pub(super) async fn rollback_then_notify_block(
    pool: &PgPool,
    tx: Transaction<'_, Postgres>,
    err: AppError,
    doc_no: &str,
    operator_id: Uuid,
) -> AppError {
    if let Err(e) = tx.rollback().await {
        tracing::warn!("領用卡關後回滾 transaction 失敗 doc={doc_no}: {e}");
    }
    if let AppError::InsufficientStock {
        message,
        storage_location_id,
        product_id,
        ..
    } = &err
    {
        let notif = NotificationService::new(pool.clone());
        if let Err(e) = notif
            .notify_stock_blocked(
                *storage_location_id,
                *product_id,
                message,
                doc_no,
                Some(operator_id),
            )
            .await
        {
            tracing::warn!("領用卡關通知失敗 doc={doc_no}: {e}");
        }
    }
    err
}

/// 儲位帳失真事件的通知（best-effort）。**只在 `tx.commit()` 成功之後呼叫。**
///
/// 提早發會在「核准後續步驟失敗、整張回滾」時留下一則描述根本沒發生的事的假警報，
/// 而收到的人無從分辨真假——一則不可信的警示比沒有警示更糟。
pub(super) async fn notify_storage_drift_after_commit(
    pool: &PgPool,
    drift: &[StorageDriftEvent],
    operator_id: Uuid,
) {
    if drift.is_empty() {
        return;
    }
    let notif = NotificationService::new(pool.clone());
    // 先依 (儲位, 品項) 去重（CodeRabbit 於 MR !3 指出）。
    //
    // 一張單的多個明細行可能都缺 SLI row，同一組 (儲位, 品項) 於是出現好幾次。
    // 每次 dispatch 都要查儲位與品項名稱、撈全體倉管、再跑一次 dedup 查詢，
    // 而 `notify_storage_drift` 的標題只由 (SKU, 儲位) 決定——**第二次之後不會產生
    // 任何新通知**，那些查詢純屬白做。
    //
    // ⚠️ CodeRabbit 同時點出反面，值得寫下來：去重也壓掉了「第一次 insert 失敗時，
    // 後面的重複事件恰好構成重試」這個效果。這裡仍選擇去重，因為那個重試是偶然不是設計：
    // 它只在「同一 (儲位,品項) 剛好有多行」時存在，而那不保證。把可靠性建立在這種巧合上
    // 比沒有重試更糟——它會讓人以為有重試。真需要重試就該明確實作。
    //
    // 保留每組第一筆，qty 因此只反映第一行而非總和：這則通知要傳達的是
    // 「這個儲位的帳失真了，去盤它」，精確數量本來就不是重點
    //（真正的數量以盤點為準，而促成盤點正是這則通知的目的）。
    let mut seen = std::collections::HashSet::new();
    for ev in drift {
        if !seen.insert((ev.storage_location_id, ev.product_id)) {
            continue;
        }
        if let Err(e) = notif
            .notify_storage_drift(
                ev.storage_location_id,
                ev.product_id,
                ev.qty,
                &ev.doc_no,
                Some(operator_id),
            )
            .await
        {
            tracing::warn!("儲位帳失真通知失敗 doc={}: {e}", ev.doc_no);
        }
    }
}

/// 盤點差異比對用：某貨架上某品項（含批號/效期維度）的系統現存量。
#[derive(sqlx::FromRow)]
struct ShelfOnHandRow {
    storage_location_id: Uuid,
    product_id: Uuid,
    batch_no: Option<String>,
    expiry_date: Option<chrono::NaiveDate>,
    on_hand_qty: Decimal,
}

/// R97-1 SoD：哪些單據類型禁止「建立者自己核准」。
///
/// **只涵蓋 ADJ（庫存調整／報廢）與 GRN（採購入庫）**（2026-08-16 使用者裁定）。
/// 這兩類的風險最集中：ADJ 憑一人意思即可增減任意數量庫存；GRN 決定進貨成本，
/// 是平均成本法的源頭。
///
/// ⚠️ **TR / STK / PO / SO 是刻意不納入，不是漏的**：
/// - `TR`：`assign_unassigned`（未分配上架）會自動建立「同倉自核准 TR」記帳，
///   使用者 2026-07 已裁定該路徑刻意繞過 WM 二人核准。納入會直接打壞它。
/// - `STK` / `PO` / `SO`：使用者裁定維持現狀，避免 5 位倉管的日常作業增加摩擦。
///
/// 要擴大範圍請改本清單，並同步更新 `tests/erp_sod_self_approval.rs`。
const SELF_APPROVAL_FORBIDDEN_DOC_TYPES: &[DocType] = &[DocType::ADJ, DocType::GRN];

/// 對受管制的單據類型套用「建立者不得自核」。其餘類型直接放行。
fn assert_document_sod(document: &Document, approver_id: Uuid) -> Result<()> {
    if !SELF_APPROVAL_FORBIDDEN_DOC_TYPES.contains(&document.doc_type) {
        return Ok(());
    }
    // 系統自動產生的單據豁免（migration 152）。
    //
    // 唯一來源是盤點單核准後自動產生的差異調整單：它的 `created_by` 被記成
    // 「核准 STK 的那個人」，但那張單沒有真正的作者——對它套用「建立者不得自核」
    // 防的是一個不存在的人，還會讓剛盤完點的倉管核准不了系統為他產生的調整單
    // （`create_stocktake_reconciliation` 的設計本就是「倉管單一核准即套用」）。
    // 2026-08-16 使用者裁定恢復原樣，與 2026-07 對自動 TR 的裁定同一原則。
    //
    // ⚠️ 判斷依據刻意用 `system_generated` 而非「`source_doc_id` 指向 STK」：
    // 後者是 request DTO 可填的欄位，而具 create+approve 權限者只要把它填成
    // 任意一張 STK 就能自核，等於重新打開本守衛要堵的洞。
    if document.system_generated {
        return Ok(());
    }
    crate::services::access::assert_not_self_approval(
        document.created_by,
        approver_id,
        &format!(
            "職務分離：{} 的建立者不得自行核准（單號 {}），請由其他倉庫管理員或管理員核准",
            document.doc_type.prefix(),
            document.doc_no
        ),
    )
}

impl DocumentService {
    /// 檢查單據存取權限（IDOR 防護）
    /// 建立者、持有 erp.document.view_all（現行僅授予倉庫管理員，見
    /// startup/permissions.rs）或管理員可存取；與 list_documents 的
    /// created_by_scope 判斷同一權限碼，避免列表看得到、詳情卻 403。
    pub fn check_access(current_user: &CurrentUser, created_by: Uuid) -> Result<()> {
        let is_creator = current_user.id == created_by;
        let has_view_all = current_user.has_permission("erp.document.view_all");

        if is_creator || has_view_all {
            Ok(())
        } else {
            Err(AppError::Forbidden("無權存取此文件".into()))
        }
    }

    /// 以 SAVEPOINT 隔離會計過帳：對多數單據，會計為附加功能，其失敗不應阻擋核准。
    ///
    /// 直接在外層 tx 內呼叫並吞掉錯誤是不夠的 —— PostgreSQL 中任一 statement
    /// 失敗會使整個 tx 進入 aborted 狀態（25P02），後續 statement（如核准的
    /// document UPDATE）全部失敗，最終回傳通用「資料庫操作失敗」。改用巢狀
    /// SAVEPOINT：會計失敗時只回滾到 savepoint，外層 tx 保持可用，核准流程
    /// 得以繼續 commit。
    ///
    /// **例外：SO / DO 銷貨類為硬失敗**（2026-07-20 裁定，#1004 follow-up）。
    /// 一段式銷貨「核准＝扣庫存＋認列營收」，若過帳失敗仍放行核准，會變成
    /// 庫存已出、帳上無銷貨收入/成本的靜默漏認列。銷貨類過帳失敗時把錯誤
    /// 往外傳，讓整張核准（含扣庫存）一起 rollback，帳與庫存永遠一致。
    async fn post_accounting_best_effort(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        document: &Document,
        lines: &[DocumentLine],
        approved_by: Uuid,
    ) -> Result<()> {
        let mut sp = tx.begin().await?;
        match AccountingService::post_document(&mut sp, document, lines, approved_by).await {
            Ok(()) => {
                sp.commit().await?;
            }
            Err(e) => {
                let doc_type = document.doc_type.prefix();
                // 先發 counter + warn，再 rollback：rollback 失敗（極罕見，如連線中斷）會
                // 因 `?` 提前 return，先記錄可確保原始過帳錯誤的觀測訊號不被吞掉。
                // 非銷貨類過帳為 best-effort，失敗不阻擋核准；若過帳全面失敗、核准仍成功 →
                // 財務帳靜默漂移，counter 讓 ops 能對「被吞掉的過帳失敗」設告警。
                metrics::counter!(
                    "ipig_accounting_posting_failures_total",
                    "doc_type" => doc_type,
                )
                .increment(1);
                tracing::warn!(
                    "會計過帳跳過或失敗 (document {}, type {}): {}",
                    document.id,
                    doc_type,
                    e
                );
                sp.rollback().await?;
                // 銷貨類（SO）硬失敗：錯誤外傳 → 外層 approve tx 整張 rollback。
                if document.doc_type == DocType::SO {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// 送審 — Service-driven audit
    /// 對於調整單(ADJ)，若報廢金額超過門檻，需要主管簽核
    pub async fn submit(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<DocumentWithLines> {
        let _ = actor.require_user()?;

        let mut tx = pool.begin().await?;

        // FOR UPDATE 鎖 document + before snapshot
        let document =
            sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status != DocStatus::Draft {
            return Err(AppError::BusinessRule(
                "Document must be in draft status to submit".to_string(),
            ));
        }

        let before_lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let before_doc = document.clone();

        // 對於調整單(報廢)，計算總金額並檢查是否需要主管簽核
        let doc_type = document.doc_type;
        let (requires_manager_approval, scrap_total_amount) = if doc_type == DocType::ADJ {
            // R97-1：金額不再只看選填的 `unit_price`。
            //
            // 舊算式是 `COALESCE(dl.unit_price, 0)`，而 ADJ 的單價完全選填
            // （`crud.rs::validate_line_qty_price` 只對 GRN 強制、對 SO 禁止），
            // 於是「單價留空 → 金額 0 → 永遠低於門檻 → 永遠不需要 ADMIN」。
            // 2026-08-16 查 prod：69 張 ADJ 全部金額為 0/NULL、經 ADMIN 二審者 0 張，
            // 門檻機制自上線以來一次都沒生效過。
            //
            // 新算式改以 `unit_price` → `products.cost_price` 的順序取價，
            // **兩者皆無時視為「無法估價」→ 一律送 ADMIN 二審**（絕不可再退回 0，
            // 那正是原本的漏洞）。
            //
            // ⚠️ 取價條件是 **> 0 而非 IS NOT NULL**（CodeAnt／Qodo 於 PR #139 各自指出）：
            // `validate_line_qty_price` 對 ADJ 只擋負數，`unit_price = 0` 是合法輸入。
            // 若寫成 `COALESCE(dl.unit_price, p.cost_price)`，明確填 0 會被當成有效價格
            // ——既不 fallback 到 cost_price、也不算「無單價」——金額仍是 0 而不需二審，
            // 等於把剛堵起來的漏洞換一扇門重開。
            let (total_amount, has_unpriced_line): (Option<Decimal>, bool) = sqlx::query_as(
                r#"
                SELECT
                    SUM(ABS(dl.qty) * v.price) AS total,
                    COALESCE(bool_or(v.price IS NULL), false) AS has_unpriced
                FROM document_lines dl
                JOIN products p ON p.id = dl.product_id
                CROSS JOIN LATERAL (
                    SELECT CASE
                        WHEN dl.unit_price > 0 THEN dl.unit_price
                        WHEN p.cost_price  > 0 THEN p.cost_price
                        ELSE NULL
                    END AS price
                ) v
                WHERE dl.document_id = $1
                "#,
            )
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;

            let scrap_amount = total_amount.unwrap_or(Decimal::ZERO);

            // 從系統設定取得報廢簽核門檻
            let threshold: Option<Decimal> = sqlx::query_scalar(
                "SELECT (value #>> '{}')::DECIMAL FROM system_settings WHERE key = 'scrap_approval_threshold'"
            )
            .fetch_optional(&mut *tx)
            .await?;

            let threshold_amount = threshold.unwrap_or(Decimal::new(5000, 0)); // 預設 5000

            // 無法估價的明細一律升級為需二審——寧可多送一次 ADMIN，
            // 也不要讓「沒填價」成為繞過門檻的方法。
            if has_unpriced_line || scrap_amount >= threshold_amount {
                (Some(true), Some(scrap_amount))
            } else {
                (Some(false), Some(scrap_amount))
            }
        } else {
            (None, None)
        };

        // 更新單據狀態
        if requires_manager_approval == Some(true) {
            // 需要主管簽核的報廢單
            sqlx::query(
                r#"
                UPDATE documents SET 
                    status = $1, 
                    requires_manager_approval = true,
                    scrap_total_amount = $2,
                    manager_approval_status = 'pending',
                    updated_at = NOW()
                WHERE id = $3
                "#,
            )
            .bind(DocStatus::Submitted)
            .bind(scrap_total_amount)
            .bind(id)
            .execute(&mut *tx)
            .await?;

            tracing::info!(
                "[Scrap Approval] Document {} requires manager approval. Total amount: {:?}",
                id,
                scrap_total_amount
            );
        } else {
            // 一般單據或金額未超過門檻
            sqlx::query(
                r#"
                UPDATE documents SET 
                    status = $1, 
                    requires_manager_approval = COALESCE($2, false),
                    scrap_total_amount = $3,
                    updated_at = NOW()
                WHERE id = $4
                "#,
            )
            .bind(DocStatus::Submitted)
            .bind(requires_manager_approval)
            .bind(scrap_total_amount)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        }

        // Audit: 讀取 after 狀態 + 寫入 audit
        let after_doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let display = format!("{}: {}", doc_type.prefix(), after_doc.doc_no);
        let before_snap = DocumentAuditSnapshot {
            document: &before_doc,
            lines: &before_lines,
        };
        let after_snap = DocumentAuditSnapshot {
            document: &after_doc,
            lines: &before_lines, // submit 不改 lines
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_SUBMIT",
                entity: Some(AuditEntity::new("document", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Self::get_by_id(pool, id).await
    }

    /// 核准（寫入庫存流水）— Service-driven audit（跨 Stock + Accounting tx）
    ///
    /// 採購單核准後僅標記 receipt_status = 'pending'（待入庫），不再自動產生入庫單；
    /// 入庫單由使用者手動建立（create_additional_grn）。
    /// 大金額 ADJ 調整單：WAREHOUSE_MANAGER 核准後進入 wm_approved 狀態，等待 ADMIN 最終核准。
    pub async fn approve(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<DocumentWithLines> {
        let user = actor.require_user()?;
        let approved_by = user.id;

        let mut tx = pool.begin().await?;

        // FOR UPDATE 鎖 document，避免 race
        let document =
            sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status != DocStatus::Submitted {
            return Err(AppError::BusinessRule(
                "Document must be in submitted status to approve".to_string(),
            ));
        }

        // R84-5：沖銷單必須走 approve_reversal，這條一般核准路徑會毀掉沖銷語意。
        //
        // 為何一般核准擋不住它：`create_reversal` 建出的沖銷單是
        // status=submitted + requires_manager_approval=true + manager_approval_status='wm_approved'，
        // 下方的 `needs_admin` 判定要求 manager_approval_status == "pending" → 對沖銷單為 false，
        // 於是直接落到一般核准分支，呼叫 `StockService::process_document` 用**當下庫存狀態**
        // 重跑一次業務邏輯（GRN 沖銷會再寫一筆 `in`），而不是鏡射原單；且此路徑只要
        // WAREHOUSE_MANAGER 即可執行，等同繞過沖銷設計的 ADMIN 最終核准（SoD）。
        // 對稱的擋板已在 `admin_approve`，此處補上（CodeRabbit 於 #1039 指出）。
        if document.reverses_doc_id.is_some() {
            return Err(AppError::BusinessRule(
                "沖銷單請走沖銷核准流程（reverse-approve），不可用一般核准".to_string(),
            ));
        }

        // R97-1 SoD：ADJ / GRN 的建立者不得自行核准（其餘類型放行，理由見常數註解）。
        assert_document_sod(&document, approved_by)?;

        let lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

        let before_doc = document.clone();

        // 大金額 ADJ：WAREHOUSE_MANAGER 核准後不直接生效，改為等待 ADMIN 核准
        let needs_admin = document.requires_manager_approval == Some(true)
            && document.manager_approval_status.as_deref() == Some("pending");

        if needs_admin {
            sqlx::query(
                r#"
                UPDATE documents SET
                    manager_approval_status = 'wm_approved',
                    approved_by = $1,
                    approved_at = NOW(),
                    updated_at = NOW()
                WHERE id = $2
                "#,
            )
            .bind(approved_by)
            .bind(id)
            .execute(&mut *tx)
            .await?;

            // Audit: WM 核准事件
            let after_doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
            let display = format!(
                "{}: {} (WM 核准)",
                document.doc_type.prefix(),
                after_doc.doc_no
            );
            let before_snap = DocumentAuditSnapshot {
                document: &before_doc,
                lines: &lines,
            };
            let after_snap = DocumentAuditSnapshot {
                document: &after_doc,
                lines: &lines,
            };
            AuditService::log_activity_tx(
                &mut tx,
                actor,
                ActivityLogEntry {
                    event_category: "ERP",
                    event_type: "DOC_WM_APPROVE",
                    entity: Some(AuditEntity::new("document", id, &display)),
                    data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                    request_context: None,
                },
            )
            .await?;

            tx.commit().await?;
            tracing::info!(
                "[ADJ Approval] Document {} approved by warehouse manager, awaiting admin approval",
                id
            );

            return Self::get_by_id(pool, id).await;
        }

        // 檢查庫存並寫入流水（同 tx；stock ledger 為此次 approve 的 side effect）
        //
        // 不用 `?`：領用被儲位庫存擋下時要先回滾、再通知倉管與操作者，
        // 而 `?` 會立刻 return，讓 tx 在背景 drop、通知也就沒機會發出去。
        let mut stock_drift: Vec<StorageDriftEvent> = Vec::new();
        if document.doc_type.affects_stock() {
            match StockService::process_document(&mut tx, &document, &lines).await {
                Ok(events) => stock_drift = events,
                Err(e) => {
                    return Err(rollback_then_notify_block(
                        pool,
                        tx,
                        e,
                        &document.doc_no,
                        approved_by,
                    )
                    .await)
                }
            }
        }

        // 會計過帳（GRN/SO 等產生傳票分錄）— 以 SAVEPOINT 隔離；非銷貨類失敗不阻擋核准，
        // 銷貨類（SO/DO）失敗則整張核准 rollback（見 post_accounting_best_effort doc）。
        Self::post_accounting_best_effort(&mut tx, &document, &lines, approved_by).await?;

        // 更新單據狀態
        sqlx::query(
            r#"
            UPDATE documents SET
                status = $1,
                approved_by = $2,
                approved_at = NOW(),
                updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(DocStatus::Approved)
        .bind(approved_by)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // 採購單核准後標記為「待入庫」，供使用者於採購單詳情頁手動建立入庫單（GRN）。
        // 2026-07-17：移除「核准即自動產生 GRN 草稿」的行為（會在單據列表與稽核日誌
        // 產生多餘的草稿，造成使用者困擾）；改由使用者手動觸發 create_additional_grn。
        if document.doc_type == DocType::PO {
            sqlx::query("UPDATE documents SET receipt_status = 'pending' WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }

        // 如果是入庫單核准，回寫更新來源採購單的 receipt_status
        if document.doc_type == DocType::GRN {
            if let Some(po_id) = document.source_doc_id {
                // Medium-2 (#290相關)：核准後守衛超量入庫（received ≤ ordered），
                // 違反即 Conflict 回滾整個核准 tx。
                Self::ensure_no_over_receipt(&mut tx, po_id).await?;
                Self::update_po_receipt_status(&mut tx, po_id).await?;
            }
        }

        // 盤點單(STK)核准 → 依各貨架「實盤 vs 系統現存量」差異，自動產生一張待審核(Submitted)
        // 調整單(ADJ)供倉管確認；差異為 0 則不產生。ADJ 綁原儲位（不產生未分配）。
        if document.doc_type == DocType::STK {
            Self::create_stocktake_reconciliation(&mut tx, actor, &document, &lines, approved_by)
                .await?;
        }

        // Audit: 主 approve 事件（含 before/after snapshot；跨 service 的 stock/accounting
        // 變更在同 tx 內，DataDiff 主要反映 document 狀態轉換）
        let after_doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let display = format!("{}: {}", document.doc_type.prefix(), after_doc.doc_no);
        let before_snap = DocumentAuditSnapshot {
            document: &before_doc,
            lines: &lines,
        };
        let after_snap = DocumentAuditSnapshot {
            document: &after_doc,
            lines: &lines,
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_APPROVE",
                entity: Some(AuditEntity::new("document", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                request_context: None,
            },
        )
        .await?;

        // GRN 軟擋：核准時若有明細未落貨架，額外寫一筆稽核事件（同 tx，進 HMAC chain）。
        // 記錄這是「有意識地留下未分配、待事後分配上架」，而非資料遺失。
        if document.doc_type == DocType::GRN {
            let mut unshelved_line_nos: Vec<i32> = Vec::new();
            let mut unshelved_qty_total = Decimal::ZERO;
            for line in lines.iter().filter(|l| l.storage_location_id.is_none()) {
                unshelved_line_nos.push(line.line_no);
                unshelved_qty_total += line.qty;
            }
            if !unshelved_line_nos.is_empty() {
                let unshelved_audit = GrnUnshelvedAudit {
                    doc_no: after_doc.doc_no.clone(),
                    unshelved_line_nos,
                    unshelved_qty_total,
                };
                AuditService::log_activity_tx(
                    &mut tx,
                    actor,
                    ActivityLogEntry {
                        event_category: "ERP",
                        event_type: "DOC_GRN_APPROVED_UNSHELVED",
                        entity: Some(AuditEntity::new("document", id, &display)),
                        data_diff: Some(DataDiff::create_only(&unshelved_audit)),
                        request_context: None,
                    },
                )
                .await?;
            }
        }

        tx.commit().await?;

        // 儲位帳失真 → commit 之後才通知（缺 row 的靜默放行路徑；行為不變，只是讓它出聲）。
        notify_storage_drift_after_commit(pool, &stock_drift, approved_by).await;

        // 採購入庫（GRN 核准）→ 解除該 PO 的「未入庫提醒」置頂（best-effort，失敗僅 warn）。
        // 與 erp.rs notify_po_pending_receipt 的置頂互為一組：建立時置頂、入庫後降級。
        if document.doc_type == DocType::GRN {
            if let Some(po_id) = document.source_doc_id {
                let notif = crate::services::NotificationService::new(pool.clone());
                if let Err(e) = notif.resolve_pinned_notifications("document", po_id).await {
                    tracing::warn!("解除 PO 未入庫置頂通知失敗 po={po_id}: {e}");
                }
            }
        }

        Self::get_by_id(pool, id).await
    }

    /// ADMIN 最終核准（大金額 ADJ 調整單）— Service-driven audit（跨 Stock + Accounting tx）
    /// 前置條件：requires_manager_approval = true 且 manager_approval_status = 'wm_approved'
    pub async fn admin_approve(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<DocumentWithLines> {
        let user = actor.require_user()?;
        let admin_id = user.id;

        let mut tx = pool.begin().await?;

        let document =
            sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status != DocStatus::Submitted {
            return Err(AppError::BusinessRule("單據必須為已提交狀態".to_string()));
        }

        if document.requires_manager_approval != Some(true)
            || document.manager_approval_status.as_deref() != Some("wm_approved")
        {
            return Err(AppError::BusinessRule(
                "此單據尚未經倉庫管理員核准，無法進行管理員最終核准".to_string(),
            ));
        }

        // R84-5：沖銷單同樣是 requires_manager_approval + wm_approved，會落入本函式的前置條件，
        // 但**絕不能**走這條路——本函式呼叫 process_document 會用「當下庫存狀態」重跑一次業務
        // 邏輯，而沖銷要的是鏡射原單實際寫入的紀錄。走錯會算出與原單不同的結果、且不會反向
        // 扣減儲位庫存。沖銷請走 approve_reversal（POST /documents/{id}/reverse-approve）。
        if document.reverses_doc_id.is_some() {
            return Err(AppError::BusinessRule(
                "沖銷單請走沖銷核准流程（reverse-approve），不可用一般最終核准".to_string(),
            ));
        }

        // R97-1 SoD 第一道：ADMIN 不得是建單者。
        assert_document_sod(&document, admin_id)?;

        // R97-1 SoD 第二道：兩階段核准必須真的是兩個人（Qodo 於 PR #139 指出）。
        //
        // 第一道只比對建單者，擋不住「A 建單、B 先以 WM 身分核准、B 再以 ADMIN 身分
        // 最終核准」——B 同時具備兩種角色時，大金額 ADJ 的兩道關卡會塌縮成一個人，
        // 二審形同虛設。`approved_by` 在 `approve()` 的 needs_admin 分支寫入，
        // 即第一階段的 WM 核准人。
        if let Some(wm_approver) = document.approved_by {
            crate::services::access::assert_not_self_approval(
                wm_approver,
                admin_id,
                "職務分離：倉庫核准人不得再擔任最終核准人，請由其他管理員完成最終核准",
            )?;
        }

        let lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let before_doc = document.clone();

        // 寫入庫存流水（不用 `?` 的理由同 `approve`：擋下時要先回滾才發得出通知）
        let mut stock_drift: Vec<StorageDriftEvent> = Vec::new();
        if document.doc_type.affects_stock() {
            match StockService::process_document(&mut tx, &document, &lines).await {
                Ok(events) => stock_drift = events,
                Err(e) => {
                    return Err(
                        rollback_then_notify_block(pool, tx, e, &document.doc_no, admin_id).await,
                    )
                }
            }
        }

        // 會計過帳 — 以 SAVEPOINT 隔離，失敗不阻擋核准
        Self::post_accounting_best_effort(&mut tx, &document, &lines, admin_id).await?;

        // 更新單據為最終核准
        sqlx::query(
            r#"
            UPDATE documents SET
                status = $1,
                manager_approval_status = 'approved',
                manager_approved_by = $2,
                manager_approved_at = NOW(),
                updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(DocStatus::Approved)
        .bind(admin_id)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // Audit
        let after_doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let display = format!(
            "{}: {} (ADMIN 核准)",
            document.doc_type.prefix(),
            after_doc.doc_no
        );
        let before_snap = DocumentAuditSnapshot {
            document: &before_doc,
            lines: &lines,
        };
        let after_snap = DocumentAuditSnapshot {
            document: &after_doc,
            lines: &lines,
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_ADMIN_APPROVE",
                entity: Some(AuditEntity::new("document", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        // 儲位帳失真 → commit 之後才通知（理由同 `approve`）
        notify_storage_drift_after_commit(pool, &stock_drift, admin_id).await;

        tracing::info!(
            "[ADJ Admin Approval] Document {} approved by admin {}",
            id,
            admin_id
        );

        Self::get_by_id(pool, id).await
    }

    /// ADMIN 駁回（大金額 ADJ 調整單）— Service-driven audit
    /// 單據退回草稿狀態，建立者可修改後重新提交
    pub async fn admin_reject(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
        reason: &str,
    ) -> Result<DocumentWithLines> {
        let user = actor.require_user()?;
        let admin_id = user.id;

        let mut tx = pool.begin().await?;

        let document =
            sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status != DocStatus::Submitted {
            return Err(AppError::BusinessRule("單據必須為已提交狀態".to_string()));
        }

        if document.requires_manager_approval != Some(true)
            || document.manager_approval_status.as_deref() != Some("wm_approved")
        {
            return Err(AppError::BusinessRule(
                "此單據尚未經倉庫管理員核准，無法進行管理員駁回".to_string(),
            ));
        }

        let lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let before_doc = document.clone();

        // 退回草稿狀態，清除倉庫核准資訊
        sqlx::query(
            r#"
            UPDATE documents SET
                status = $1,
                manager_approval_status = 'rejected',
                manager_approved_by = $2,
                manager_approved_at = NOW(),
                manager_reject_reason = $3,
                approved_by = NULL,
                approved_at = NULL,
                updated_at = NOW()
            WHERE id = $4
            "#,
        )
        .bind(DocStatus::Draft)
        .bind(admin_id)
        .bind(reason)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // Audit
        let after_doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let display = format!(
            "{}: {} (ADMIN 駁回: {})",
            document.doc_type.prefix(),
            after_doc.doc_no,
            reason
        );
        let before_snap = DocumentAuditSnapshot {
            document: &before_doc,
            lines: &lines,
        };
        let after_snap = DocumentAuditSnapshot {
            document: &after_doc,
            lines: &lines,
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_ADMIN_REJECT",
                entity: Some(AuditEntity::new("document", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        tracing::info!(
            "[ADJ Admin Reject] Document {} rejected by admin {}. Reason: {}",
            id,
            admin_id,
            reason
        );

        Self::get_by_id(pool, id).await
    }

    /// 作廢 — Service-driven audit
    pub async fn cancel(
        pool: &PgPool,
        actor: &ActorContext,
        id: Uuid,
    ) -> Result<DocumentWithLines> {
        let _ = actor.require_user()?;

        let mut tx = pool.begin().await?;

        let document =
            sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1 FOR UPDATE")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

        if document.status == DocStatus::Approved {
            return Err(AppError::BusinessRule(
                "Cannot cancel approved documents. Use reversal instead.".to_string(),
            ));
        }

        let lines = sqlx::query_as::<_, DocumentLine>(
            "SELECT * FROM document_lines WHERE document_id = $1 ORDER BY line_no",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;
        let before_doc = document.clone();

        sqlx::query("UPDATE documents SET status = $1, updated_at = NOW() WHERE id = $2")
            .bind(DocStatus::Cancelled)
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Audit
        let after_doc = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let display = format!(
            "{}: {} (作廢)",
            document.doc_type.prefix(),
            after_doc.doc_no
        );
        let before_snap = DocumentAuditSnapshot {
            document: &before_doc,
            lines: &lines,
        };
        let after_snap = DocumentAuditSnapshot {
            document: &after_doc,
            lines: &lines,
        };
        AuditService::log_activity_tx(
            &mut tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_CANCEL",
                entity: Some(AuditEntity::new("document", id, &display)),
                data_diff: Some(DataDiff::compute(Some(&before_snap), Some(&after_snap))),
                request_context: None,
            },
        )
        .await?;

        tx.commit().await?;

        Self::get_by_id(pool, id).await
    }

    /// 盤點(STK)核准後，依各貨架「實盤 vs 系統現存量」差異，產生一張待審核(Submitted)
    /// 調整單(ADJ)供倉管確認。差異為 0 不產生；ADJ 行綁原儲位（貨架層級、不產生未分配）。
    /// 免主管簽核（`requires_manager_approval = false`），倉管單一核准即套用。
    /// 於 approve 的同一 tx 內執行。
    ///
    /// ⚠️ 本單標記 `system_generated = true`（migration 152），使 R97-1 的職務分離
    /// 守衛對其豁免。因為 `created_by` 這裡填的是**核准 STK 的那個人**，不是作者——
    /// 少了這個標記，剛盤完點的倉管會核准不了系統為他產生的調整單。
    async fn create_stocktake_reconciliation(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        actor: &ActorContext,
        stk: &Document,
        stk_lines: &[DocumentLine],
        created_by: Uuid,
    ) -> Result<()> {
        let warehouse_id = stk
            .warehouse_id
            .ok_or_else(|| AppError::BusinessRule("盤點單缺少倉庫".to_string()))?;

        // 盤點行可能以包裝單位計數（見 stocktake::counting_uom 的「整除才用盒」規則），
        // 而 storage_location_inventory.on_hand_qty 恆為 base_uom。兩者相減前必須先換到
        // 同一個單位，否則實盤「3 盒」會被當成 3 雙去減 150 雙，差異算成 −147，
        // 開出一張把該貨架清空的盤虧 ADJ。
        // 換算後 line.uom 已是 base_uom，下方 ADJ 明細沿用 line.uom 亦隨之正確。
        let stk_lines = StockService::to_base_lines(tx, stk_lines).await?;
        let stk_lines = stk_lines.as_slice();

        // 批次取相關貨架的系統現存量，避免逐盤點行查的 N+1。
        let loc_ids: Vec<Uuid> = stk_lines
            .iter()
            .filter_map(|l| l.storage_location_id)
            .collect();
        let sli_rows: Vec<ShelfOnHandRow> = sqlx::query_as(
            r#"
                SELECT storage_location_id, product_id, batch_no, expiry_date, on_hand_qty
                FROM storage_location_inventory
                WHERE storage_location_id = ANY($1)
                "#,
        )
        .bind(&loc_ids)
        .fetch_all(&mut **tx)
        .await?;

        // key 正規化與盤點行一致：batch→''、expiry→1900-01-01（對齊 sli 唯一索引）
        let sentinel = chrono::NaiveDate::from_ymd_opt(1900, 1, 1).expect("valid sentinel date");
        let mut on_hand: std::collections::HashMap<
            (Uuid, Uuid, String, chrono::NaiveDate),
            Decimal,
        > = std::collections::HashMap::new();
        for r in sli_rows {
            let key = (
                r.storage_location_id,
                r.product_id,
                r.batch_no.unwrap_or_default(),
                r.expiry_date.unwrap_or(sentinel),
            );
            *on_hand.entry(key).or_insert(Decimal::ZERO) += r.on_hand_qty;
        }

        // 各貨架行差異 = 實盤(line.qty) − 系統現存量
        let mut diff_lines: Vec<(Uuid, &DocumentLine, Decimal)> = Vec::new();
        for line in stk_lines {
            let Some(loc_id) = line.storage_location_id else {
                continue; // 僅處理貨架層級盤點行
            };
            let key = (
                loc_id,
                line.product_id,
                line.batch_no.clone().unwrap_or_default(),
                line.expiry_date.unwrap_or(sentinel),
            );
            let current = on_hand.get(&key).copied().unwrap_or(Decimal::ZERO);
            let diff = line.qty - current;
            if diff != Decimal::ZERO {
                diff_lines.push((loc_id, line, diff));
            }
        }

        if diff_lines.is_empty() {
            return Ok(()); // 盤點無差異
        }

        let adj_no = Self::generate_doc_no(tx, DocType::ADJ).await?;
        let adj_id = Uuid::new_v4();
        let remark = format!("盤點差異自動調整 from {}", stk.doc_no);

        let adj_doc = sqlx::query_as::<_, Document>(
            r#"
            INSERT INTO documents (
                id, doc_type, doc_no, status, warehouse_id, source_doc_id,
                doc_date, remark, requires_manager_approval, system_generated,
                created_by, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false, true, $9, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(adj_id)
        .bind(DocType::ADJ)
        .bind(&adj_no)
        .bind(DocStatus::Submitted)
        .bind(warehouse_id)
        .bind(stk.id)
        .bind(stk.doc_date)
        .bind(&remark)
        .bind(created_by)
        .fetch_one(&mut **tx)
        .await?;

        let mut adj_lines: Vec<DocumentLine> = Vec::new();
        for (idx, (loc_id, line, diff)) in diff_lines.iter().enumerate() {
            let adj_line = sqlx::query_as::<_, DocumentLine>(
                r#"
                INSERT INTO document_lines (
                    id, document_id, line_no, product_id, qty, uom,
                    batch_no, expiry_date, remark, storage_location_id
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING *
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(adj_id)
            .bind((idx + 1) as i32)
            .bind(line.product_id)
            .bind(*diff)
            .bind(&line.uom)
            .bind(&line.batch_no)
            .bind(line.expiry_date)
            .bind(&remark)
            .bind(*loc_id)
            .fetch_one(&mut **tx)
            .await?;
            adj_lines.push(adj_line);
        }

        let display = format!(
            "{}: {} (盤點差異 from {})",
            DocType::ADJ.prefix(),
            adj_no,
            stk.doc_no
        );
        let snap = DocumentAuditSnapshot {
            document: &adj_doc,
            lines: &adj_lines,
        };
        AuditService::log_activity_tx(
            tx,
            actor,
            ActivityLogEntry {
                event_category: "ERP",
                event_type: "DOC_CREATE",
                entity: Some(AuditEntity::new("document", adj_id, &display)),
                data_diff: Some(DataDiff::create_only(&snap)),
                request_context: None,
            },
        )
        .await?;

        tracing::info!(
            "[Stocktake] {} 核准產生差異調整單 {}（{} 行差異，待倉管核准）",
            stk.doc_no,
            adj_no,
            adj_lines.len()
        );

        Ok(())
    }
}
