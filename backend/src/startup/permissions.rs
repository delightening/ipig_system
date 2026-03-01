// 權限與角色權限初始化模組
//
// 確保所有必要的權限和角色權限在啟動時存在

use crate::Result;

/// 確保必要的權限存在於資料庫
/// 用於補充 migration 中未包含的權限
pub async fn ensure_required_permissions(pool: &sqlx::PgPool) -> Result<()> {
    // 需要確保存在的權限清單
    let required_permissions = vec![
        // 動物來源管理
        ("animal.source.manage", "管理動物來源", "animal", "可管理動物來源資料"),
        // 版本還原
        ("aup.version.restore", "還原版本", "aup", "可還原計畫版本"),
        // Amendment 分類
        ("aup.amendment.classify", "分類修正案", "aup", "可判斷修正案為 Major 或 Minor"),
        // Co-Editor 指派
        ("aup.coeditor.assign", "指派協作編輯", "aup", "可指派 Co-Editor"),
        // 緊急處置權限
        ("animal.emergency.stop", "緊急停止實驗", "animal", "可緊急叫停實驗（動物福利）"),
        ("animal.emergency.medication", "緊急用藥", "animal", "可執行緊急用藥"),
        // 安樂死權限
        ("animal.euthanasia.recommend", "建議安樂死", "animal", "可建議執行安樂死"),
        ("animal.euthanasia.approve", "核准安樂死", "animal", "可核准安樂死決策"),
        ("animal.euthanasia.execute", "執行安樂死", "animal", "可執行安樂死（需經核准）"),
        ("animal.euthanasia.arbitrate", "安樂死仲裁", "animal", "可進行安樂死爭議仲裁"),
        // Dashboard
        ("dashboard.view", "查看儀表板", "dashboard", "可查看系統儀表板"),
        // 儲位管理（migration 僅定義 view/edit，補齊 create/delete）
        ("erp.storage.create", "建立儲位", "erp", "可建立儲位"),
        ("erp.storage.delete", "刪除儲位", "erp", "可刪除儲位"),
        // 單據取消與刪除
        ("erp.document.cancel", "取消單據", "erp", "可取消單據"),
        ("erp.document.delete", "刪除單據", "erp", "可刪除單據"),
        // HR 加班全部紀錄查看
        ("hr.overtime.view_all", "查看所有加班紀錄", "hr", "可查看所有員工的加班紀錄"),
        // 人員訓練紀錄 (GLP 合規)
        ("training.view", "查看訓練紀錄", "training", "可查看人員訓練紀錄"),
        ("training.manage", "管理訓練紀錄", "training", "可新增、編輯、刪除訓練紀錄"),
        ("equipment.view", "查看設備", "equipment", "可查看設備與校正紀錄"),
        ("equipment.manage", "管理設備", "equipment", "可新增、編輯、刪除設備與校正紀錄"),
        // QAU (GLP 品質保證單位) - 唯讀檢視
        ("qau.dashboard.view", "查看 QAU 儀表板", "qau", "GLP 品質保證：可查看研究狀態、審查進度、稽核摘要"),
        ("qau.protocol.view", "QAU 檢視計畫", "qau", "唯讀檢視所有計畫書"),
        ("qau.audit.view", "QAU 檢視稽核", "qau", "唯讀檢視稽核日誌"),
        ("qau.animal.view", "QAU 檢視動物", "qau", "唯讀檢視動物紀錄"),
    ];
    
    for (code, name, module, description) in required_permissions {
        sqlx::query(r#"
            INSERT INTO permissions (id, code, name, module, description, created_at)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW())
            ON CONFLICT (code) DO NOTHING
        "#)
        .bind(code)
        .bind(name)
        .bind(module)
        .bind(description)
        .execute(pool)
        .await?;
    }
    
    tracing::info!("[Permissions] ✓ Required permissions verified");
    Ok(())
}

/// 確保所有角色擁有正確的權限
/// 在程式啟動時自動配置所有系統角色的權限
pub async fn ensure_all_role_permissions(pool: &sqlx::PgPool) -> Result<()> {
    // 定義每個角色的權限
    let role_permissions: Vec<(&str, Vec<&str>)> = vec![
        // ============================================
        // WAREHOUSE_MANAGER (倉庫管理員) - ERP 完整權限
        // ============================================
        ("WAREHOUSE_MANAGER", vec![
            // 倉庫管理
            "erp.warehouse.view", "erp.warehouse.create", "erp.warehouse.edit",
            // 產品管理
            "erp.product.view", "erp.product.create", "erp.product.edit",
            // 夥伴管理
            "erp.partner.view", "erp.partner.create", "erp.partner.edit",
            // 儲位管理（完整 CRUD）
            "erp.storage.view", "erp.storage.create", "erp.storage.edit", "erp.storage.delete",
            "erp.storage.inventory.view", "erp.storage.inventory.edit",
            // 單據管理
            "erp.document.view", "erp.document.create", "erp.document.edit", 
            "erp.document.submit", "erp.document.approve",
            "erp.document.cancel", "erp.document.delete",
            // 採購
            "erp.purchase.create", "erp.purchase.approve",
            "erp.grn.create", "erp.pr.create",
            // 庫存操作
            "erp.stock.in", "erp.stock.out", "erp.stock.view", 
            "erp.stock.adjust", "erp.stock.transfer",
            "erp.stocktake.create",
            // 報表
            "erp.report.view", "erp.report.export", "erp.report.download",
            // 庫存查看
            "erp.inventory.view",
            // Dashboard
            "dashboard.view",
        ]),
        
        // ============================================
        // PURCHASING (採購人員) - ERP 採購相關權限
        // ============================================
        ("PURCHASING", vec![
            // 基本查詢
            "erp.warehouse.view", "erp.product.view",
            // 夥伴管理
            "erp.partner.view", "erp.partner.create", "erp.partner.edit",
            // 單據管理
            "erp.document.view", "erp.document.create", "erp.document.edit", "erp.document.submit",
            // 採購
            "erp.purchase.create", "erp.grn.create", "erp.pr.create",
            // 庫存查詢
            "erp.stock.view",
            // 報表
            "erp.report.view",
            // Dashboard
            "dashboard.view",
        ]),
        
        // ============================================
        // PI (計畫主持人) - 計畫管理、動物查看
        // ============================================
        ("PI", vec![
            // 計畫管理
            "aup.protocol.view_own", "aup.protocol.create", "aup.protocol.edit", 
            "aup.protocol.submit", "aup.protocol.delete",
            // 審查流程
            "aup.review.view", "aup.review.reply",
            // 附件管理（含刪除自己的附件）
            "aup.attachment.view", "aup.attachment.download", "aup.attachment.upload",
            "aup.attachment.delete",
            // 版本管理（含還原）
            "aup.version.view", "aup.version.restore",
            // 動物管理
            "animal.animal.view_project",
            "animal.record.view",
            // 匯出
            "animal.export.medical", "animal.export.observation", "animal.export.surgery",
            // Dashboard
            "dashboard.view",
        ]),
        
        // ============================================
        // VET (獸醫師) - 審查計畫、動物查看、獸醫建議、緊急處置
        // 只看、給建議，不參與現場工作
        // ============================================
        ("VET", vec![
            // AUP 計畫審查
            "aup.protocol.view_all", "aup.protocol.view_own", "aup.protocol.review",
            "aup.review.view", "aup.review.comment",
            // AUP 附件
            "aup.attachment.view", "aup.attachment.download",
            // AUP 版本
            "aup.version.view",
            // Amendment 變更申請（審查、檢視）
            "amendment.read", "amendment.review",
            // 動物管理（只看）
            "animal.animal.view_all", "animal.animal.view_project",
            "animal.record.view",
            // 匯出（所有紀錄）
            "animal.export.medical", "animal.export.observation", "animal.export.surgery", "animal.export.experiment",
            // 獸醫師功能（所有）
            "animal.vet.recommend", "animal.vet.read",
            // 緊急處置
            "animal.emergency.stop",
            "animal.euthanasia.recommend", "animal.euthanasia.approve",
            // Dashboard
            "dashboard.view",
        ]),
        
        // ============================================
        // REVIEWER (審查委員) - 查看所有計畫、計畫審查權限
        // ============================================
        ("REVIEWER", vec![
            // 計畫審查（查看所有計畫）
            "aup.protocol.view_all", "aup.protocol.view_own", "aup.protocol.review",
            // 審查流程
            "aup.review.view", "aup.review.comment",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download",
            // 版本管理
            "aup.version.view",
            // Amendment 變更申請（審查、檢視）
            "amendment.read", "amendment.review",
            // Dashboard
            "dashboard.view",
        ]),
        
        // ============================================
        // IACUC_CHAIR (IACUC 主席) - 計畫核准、審查人員指派、安樂死仲裁
        // 注意：IACUC_CHAIR 不是公司員工，無 HR 權限
        // ============================================
        ("IACUC_CHAIR", vec![
            // 計畫管理
            "aup.protocol.view_all", "aup.protocol.view_own", "aup.protocol.review", 
            "aup.protocol.approve", "aup.protocol.change_status",
            // 審查流程
            "aup.review.view", "aup.review.comment", "aup.review.assign",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download",
            // 版本管理
            "aup.version.view",
            // 動物管理 - 僅查看，不含來源管理
            "animal.animal.view_all",
            "animal.record.view",
            // 安樂死仲裁權限（IACUC_CHAIR 為最終決策者）
            "animal.euthanasia.approve", "animal.euthanasia.arbitrate",
            // Dashboard
            "dashboard.view",
        ]),
        
        // ============================================
        // IACUC_STAFF (執行秘書) - 所有 AUP 權限
        // ============================================
        ("IACUC_STAFF", vec![
            // AUP 計畫管理（全部）
            "aup.protocol.view_all", "aup.protocol.view_own", 
            "aup.protocol.create", "aup.protocol.edit", "aup.protocol.submit",
            "aup.protocol.review", "aup.protocol.approve", "aup.protocol.change_status",
            "aup.protocol.delete",
            // AUP 審查流程（全部）
            "aup.review.view", "aup.review.assign", "aup.review.comment", "aup.review.reply",
            // AUP 附件管理（全部）
            "aup.attachment.view", "aup.attachment.download", "aup.attachment.upload", "aup.attachment.delete",
            // AUP 版本管理
            "aup.version.view", "aup.version.restore",
            // AUP 額外功能
            "aup.amendment.classify",   // 分類修正案（執行秘書負責判斷 Major/Minor）
            "aup.coeditor.assign",      // 指派協作編輯
            "aup.protocol.assign_co_editor", // 確保相容性
            // Dashboard
            "dashboard.view",
        ]),
        
        // ============================================
        // EXPERIMENT_STAFF (試驗工作人員) - Co-Editor 協助編輯、動物紀錄、ERP 查詢
        // ============================================
        ("EXPERIMENT_STAFF", vec![
            // 計畫管理（僅 Co-Editor 權限，不可獨立建立/提交/刪除計畫）
            "aup.protocol.view_own", "aup.protocol.edit",
            // 審查流程
            "aup.review.view", "aup.review.reply",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download", 
            "aup.attachment.upload", "aup.attachment.delete",
            // 版本管理
            "aup.version.view",
            // 物種管理
            "species.read", "species.create", "species.update",
            // 動物管理 - 可查看所有動物、新增、編輯、匯入
            "animal.animal.view_all", "animal.animal.create", "animal.animal.edit", "animal.animal.import",
            "animal.record.view", "animal.record.create", "animal.record.edit",
            "animal.record.observation", "animal.record.surgery", 
            "animal.record.weight", "animal.record.vaccine", "animal.record.sacrifice",
            // 動物來源管理
            "animal.source.manage",
            // 緊急處置權限
            "animal.emergency.medication", "animal.emergency.stop",
            // 安樂死執行權限（需由 PI 或 VET 核准）
            "animal.euthanasia.execute",
            // 匯出（含病歷）
            "animal.export.medical", "animal.export.observation", "animal.export.surgery", "animal.export.experiment",
            // ERP 查詢（僅讀取）+ 請購單建立 + 單據建立
            "erp.warehouse.view", "erp.product.view", "erp.partner.view",
            "erp.stock.view", "erp.inventory.view",
            "erp.pr.create",  // 可建立請購單
            // 單據管理（可建立銷售單等）
            "erp.document.view", "erp.document.create", "erp.document.edit",
            "erp.document.submit",
            // HR 權限（內部員工基本權限）
            "hr.attendance.view", "hr.attendance.clock",
            "hr.leave.view", "hr.leave.create",
            "hr.overtime.view", "hr.overtime.create",
            "hr.balance.view",
            // Dashboard 權限
            "dashboard.view",
        ]),
        
        // ============================================
        // ADMIN_STAFF (行政) - 全部 HR 權限 + 庫存報表 Audit + 管理階級 Audit
        // ============================================
        ("ADMIN_STAFF", vec![
            // HR 權限（全部）
            "hr.attendance.view", "hr.attendance.view_all", "hr.attendance.clock", "hr.attendance.correct",
            "hr.overtime.view", "hr.overtime.view_all", "hr.overtime.create", "hr.overtime.approve",
            "hr.leave.view", "hr.leave.view_all", "hr.leave.create", "hr.leave.approve", "hr.leave.manage",
            "hr.balance.view", "hr.balance.manage",
            "hr.calendar.config", "hr.calendar.view", "hr.calendar.sync", "hr.calendar.conflicts",
            // ERP 倉庫管理權限
            "erp.warehouse.view", "erp.warehouse.create", "erp.warehouse.edit",
            "erp.product.view", "erp.product.create", "erp.product.edit",
            "erp.partner.view", "erp.partner.create", "erp.partner.edit",
            // 儲位管理
            "erp.storage.view", "erp.storage.create", "erp.storage.edit", "erp.storage.delete",
            "erp.storage.inventory.view", "erp.storage.inventory.edit",
            // 單據管理
            "erp.document.view", "erp.document.create", "erp.document.edit",
            "erp.document.submit",
            // 庫存報表 Audit 權限
            "erp.stock.view", "erp.inventory.view", "erp.report.view",
            // 管理階級 Audit 權限（全部 5 個）
            "audit.logs.view", "audit.logs.export", "audit.timeline.view", "audit.alerts.view", "audit.alerts.manage",
            // 人員訓練紀錄 (GLP 合規)
            "training.view", "training.manage",
            "equipment.view", "equipment.manage",
            // Dashboard 權限
            "dashboard.view",
        ]),
        
        // ============================================
        // QAU (品質保證單位) - GLP 唯讀檢視，獨立於研究執行
        // ============================================
        ("QAU", vec![
            "qau.dashboard.view", "qau.protocol.view", "qau.audit.view", "qau.animal.view",
            "aup.protocol.view_all", "aup.review.view", "aup.attachment.view", "aup.attachment.download",
            "aup.version.view", "audit.logs.view", "animal.animal.view_all", "animal.record.view",
            "dashboard.view",
        ]),
        
        // ============================================
        // CLIENT (委託人) - 計畫/動物查看（僅自己相關）
        // ============================================
        ("CLIENT", vec![
            // 計畫查看
            "aup.protocol.view_own",
            // 審查流程
            "aup.review.view",
            // 附件管理
            "aup.attachment.view", "aup.attachment.download",
            // 版本管理
            "aup.version.view",
            // 動物查看
            "animal.animal.view_project",
            "animal.record.view",
            // 匯出
            "animal.export.medical", "animal.export.observation", "animal.export.surgery",
            // Dashboard
            "dashboard.view",
        ]),
    ];
    
    let mut total_assigned = 0;
    
    for (role_code, permissions) in &role_permissions {
        // 為角色分配權限
        let result = sqlx::query(r#"
            INSERT INTO role_permissions (role_id, permission_id)
            SELECT r.id, p.id
            FROM roles r
            CROSS JOIN permissions p
            WHERE r.code = $1
            AND p.code = ANY($2::text[])
            ON CONFLICT DO NOTHING
        "#)
        .bind(role_code)
        .bind(&permissions[..])
        .execute(pool)
        .await?;
        
        let assigned = result.rows_affected();
        if assigned > 0 {
            tracing::debug!("[Permissions] {} -> {} new permissions", role_code, assigned);
            total_assigned += assigned;
        }
    }
    
    if total_assigned > 0 {
        tracing::info!("[Permissions] ✓ {} total permissions assigned to all roles", total_assigned);
    } else {
        tracing::info!("[Permissions] ✓ All role permissions already configured");
    }
    
    Ok(())
}
