// 權限與角色權限初始化模組
//
// 確保所有必要的權限和角色權限在啟動時存在

use crate::Result;

/// 確保必要的權限存在於資料庫
/// 用於補充 migration 中未包含的權限
pub async fn ensure_required_permissions(pool: &sqlx::PgPool) -> Result<()> {
    // 需要確保存在的權限清單
    let required_permissions = vec![
        // 動物管理（刪除）— admin 專屬，staff 不可刪除動物
        ("animal.animal.delete", "刪除動物", "animal", "可永久刪除（軟刪除）動物紀錄，僅限系統管理員"),
        // 動物來源管理
        ("animal.source.manage", "管理動物來源", "animal", "可管理動物來源資料"),
        // 動物預約與試驗規劃：檢視 / 操作分離。
        // 原本讀寫共用 animal.info.assign → SD 與試驗工作人員連頁面都進不來；
        // 且該權限同時被 batch_assign_animals 使用、由 EXPERIMENT_STAFF / VET 持有，
        // 沿用就做不到「僅執秘可操作」。故本頁自帶兩個專屬權限。
        // 見 docs/audit/button-permission-gate-2026-08-07.md §6。
        ("animal.planning.view", "檢視動物預約與試驗規劃", "animal", "可檢視全場動物按試驗分組的分配清冊與缺口（唯讀）"),
        ("animal.planning.manage", "管理動物預約與試驗規劃", "animal", "可新增預定試驗、批次預約 / 解除預約、正式分配進實驗、編輯規劃頁備註"),
        // 動物轉讓的協調段（發起 / 指定新計畫 / 完成 / 拒絕）。
        //
        // P0-3（2026-09-05 使用者裁定選項 B）：五段簽核原本有四段共用
        // `animal.record.create`，任何持該碼又能存取該動物的人可獨力把流程從發起推到完成
        // ——只有第 4 段（PI 同意）有職責分離。本碼把協調段從「登錄動物紀錄」的能力中分離
        // 出來，授予執行秘書（IACUC_STAFF），與第 2 段（獸醫評估，`animal.vet.recommend`）
        // 及第 4 段（簽署權責，`check_transfer_signing_authority`）形成三方分權。
        //
        // ⚠️ 這是**取代**而非疊加：原本持有 `animal.record.create` 的 EXPERIMENT_STAFF /
        // INTERN 不再能推進轉讓流程。他們原本做得到只是權限發錯的副作用——這條流程
        // 本來就該由執秘跑（見 docs/reviews/2026-09-03-code-side-issues.md §P0-3）。
        ("animal.transfer.manage", "管理動物轉讓流程", "animal", "可發起動物轉讓、指定轉入計畫、完成或拒絕轉讓（獸醫評估與 PI 同意另有專屬權責，不含在內）"),
        // 血檢項目管理（模板、組合、常用組合）
        ("animal.blood_test_template.manage", "血檢項目管理", "animal", "可檢視與編輯血檢項目模板、組合、常用組合"),
        // 版本還原
        ("aup.version.restore", "還原版本", "aup", "可還原計畫版本"),
        // Amendment 分類
        ("aup.amendment.classify", "分類修正案", "aup", "可判斷修正案為 Major 或 Minor"),
        // Amendment 決定（H6 / GLP §11.70 / ISO A.5.18）：
        // record_amendment_decision handler 除既有 reviewer-assignment 檢查外，再要求此明確權限。
        // 防禦深度：「assignment 表存在」與「明確授權」雙層守衛。
        ("aup.amendment.approve", "決定修正案", "aup", "可對被指派的修正案投票（APPROVE/REJECT/REVISION）"),
        // 匯入已核准計畫（場內既有、已通過審查的計劃直接建立成 APPROVED，跳過審查流程）
        ("aup.protocol.import_approved", "匯入已核准計畫", "aup", "可直接建立已核准狀態之計畫（跳過 IACUC 審查），用於匯入場內既有已通過計劃以進行會計/管理"),
        // 緊急處置權限
        ("animal.emergency.stop", "緊急停止實驗", "animal", "可緊急叫停實驗（動物福利）"),
        ("animal.emergency.medication", "緊急用藥", "animal", "可執行緊急用藥"),
        // 安樂死權限
        ("animal.euthanasia.recommend", "建議安樂死", "animal", "可建議執行安樂死"),
        ("animal.euthanasia.approve", "核准安樂死", "animal", "可核准安樂死決策"),
        ("animal.euthanasia.execute", "執行安樂死", "animal", "可執行安樂死（需經核准）"),
        ("animal.euthanasia.arbitrate", "安樂死仲裁", "animal", "可進行安樂死爭議仲裁"),
        // R53-2: 廢棄物再利用紀錄（byproduct reuse）
        // PI 角色不帶（R53-6 audit blacklist 配套）
        ("animal.byproduct_sample.view", "檢視廢棄物再利用紀錄", "animal", "可檢視結案豬隻組織/血液再利用紀錄"),
        ("animal.byproduct_sample.write", "編輯廢棄物再利用紀錄", "animal", "可新增 / 修改 / 刪除廢棄物再利用紀錄"),
        // Dashboard
        ("dashboard.view", "查看儀表板", "dashboard", "可查看系統儀表板"),
        // 儲位管理（migration 僅定義 view/edit，補齊 create/delete）
        ("erp.storage.create", "建立儲位", "erp", "可建立儲位"),
        ("erp.storage.delete", "刪除儲位", "erp", "可刪除儲位"),
        // ⚠️ 死碼：定義了、也授予了 admin，但**沒有任何 handler 檢查它**（2026-08-17 全樹確認）。
        // 找「大額調整單終審」請用下方的 `erp.document.final_approve`，不要復用本碼——
        // 它的敘述講的是倉管階段的核准，語意不同。保留是為了不動既有 role_permissions 列，
        // 清除另案（R97 backlog）。
        ("erp.adj.approve", "核准調整單（死碼）", "erp", "⚠️ 未被任何 handler 檢查。終審請用 erp.document.final_approve"),
        // ────────────────────────────────────────────────────────────────
        // R97-1c：把「終審」從技術角色（is_admin）改為業務授權（權限碼）
        //
        // 大額調整單的最終核准、以及沖銷單的核准，本質是**業務決策**（誰對這批資產
        // 負責），不是技術操作。原本綁在 `is_admin()` 上等於只有系統管理員能做——
        // 那正是本檔上方 2026-08-09 註解說的「只有管理員做得到是意外，不是設計」。
        //
        // 改為權限碼後：授權集中於 permissions 表、可從角色管理頁看見與調整；
        // 換負責人時移轉 DIRECTOR 角色即可，不必記得「還要另外開什麼」。
        //
        // ⚠️ 破窗路徑：`has_permission` 對 admin / SYSTEM_ADMIN 短路回 true
        // （`middleware/auth.rs:88`），故系統管理員帳號**自動保有**這兩個權限，
        // 供負責人不在時緊急處理，且動用會留稽核紀錄。
        ("erp.document.final_approve", "單據終審核准", "erp", "可對已經倉管核准的大額調整單做最終核准或駁回（業務終審關）"),
        ("erp.document.reverse_approve", "核准沖銷單", "erp", "可核准沖銷單，使其反向抵銷原單的庫存與帳務"),
        // 單據取消與刪除
        ("erp.document.cancel", "取消單據", "erp", "可取消單據"),
        ("erp.document.delete", "刪除單據", "erp", "可刪除單據"),
        // HR 加班全部紀錄查看
        ("hr.overtime.view_all", "查看所有加班紀錄", "hr", "可查看所有員工的加班紀錄"),
        // HR Google Calendar 同步
        ("hr.calendar.view", "查看行事曆事件", "hr", "可查看 Google Calendar 公司行事曆事件"),
        ("hr.calendar.config", "設定行事曆同步", "hr", "可設定 Google Calendar 同步連線"),
        ("hr.calendar.sync", "執行行事曆同步", "hr", "可手動觸發 Google Calendar 同步"),
        ("hr.calendar.conflicts", "處理行事曆衝突", "hr", "可查看並解決行事曆同步衝突"),
        // 請假行事曆（原生，非 Google）。與 hr.leave.view_all 分開：行事曆只揭露
        // 「誰哪天不在、誰代理」，不含事由與附件，故可開給一般員工；
        // 實際看得到誰，再由部門與代理關係動態決定（見 services/hr/leave_calendar.rs）。
        ("hr.leave.view_calendar", "查看請假行事曆", "hr", "可查看請假與代理行事曆（可見範圍依部門與代理關係決定）"),
        // 人員訓練紀錄 (GLP 合規)
        ("training.view", "查看訓練紀錄", "training", "可查看人員訓練紀錄"),
        ("training.manage", "管理訓練紀錄", "training", "可新增、編輯、刪除訓練紀錄"),
        ("training.manage_own", "管理自己的訓練紀錄", "training", "可新增、編輯、刪除自己的訓練紀錄"),
        ("equipment.view", "查看設備", "equipment", "可查看設備與校正紀錄"),
        ("equipment.manage", "管理設備", "equipment", "可新增、編輯、刪除設備與校正紀錄"),
        ("equipment.maintenance.manage", "管理維修保養紀錄", "equipment", "可新增、編輯、刪除維修保養紀錄"),
        ("equipment.plan.manage", "管理年度校正計畫", "equipment", "可產生、新增、編輯、刪除年度校正計畫"),
        ("equipment.disposal.approve", "核准設備報廢申請", "equipment", "可審核並核准設備報廢申請"),
        ("equipment.maintenance.review", "驗收維修保養紀錄", "equipment", "可驗收維修保養紀錄並簽核"),
        // QAU (GLP 品質保證單位) - 唯讀檢視
        ("qau.dashboard.view", "查看 QAU 儀表板", "qau", "GLP 品質保證：可查看研究狀態、審查進度、稽核摘要"),
        ("qau.protocol.view", "QAU 檢視計畫", "qau", "唯讀檢視所有計畫書"),
        ("qau.audit.view", "QAU 檢視稽核", "qau", "唯讀檢視稽核日誌"),
        ("qau.animal.view", "QAU 檢視動物", "qau", "唯讀檢視動物紀錄"),
        // QAU 計畫管理（稽查報告、NC、SOP、稽查排程）
        ("qau.inspection.view", "QAU 檢視稽查報告", "qau", "查看稽查報告列表與詳情"),
        ("qau.inspection.manage", "QAU 管理稽查報告", "qau", "建立、編輯、關閉稽查報告"),
        ("qau.nc.view", "QAU 檢視不符合事項", "qau", "查看 NC 與 CAPA 列表"),
        ("qau.nc.manage", "QAU 管理不符合事項", "qau", "建立、指派、結案不符合事項"),
        ("qau.sop.view", "QAU 檢視 SOP", "qau", "查看 SOP 文件列表"),
        ("qau.sop.manage", "QAU 管理 SOP", "qau", "建立、版本控制 SOP 文件"),
        ("qau.schedule.view", "QAU 檢視稽查排程", "qau", "查看年度稽查計畫"),
        ("qau.schedule.manage", "QAU 管理稽查排程", "qau", "建立、維護年度稽查計畫"),
        // 全庫 IDXF 匯出/匯入（一鍵輸出/匯入整個資料庫）
        ("admin.data.export", "全庫資料匯出", "admin", "可一鍵匯出整個資料庫為 IDXF 格式"),
        ("admin.data.import", "全庫資料匯入", "admin", "可上傳 IDXF JSON 匯入資料庫"),
        // 角色 API（handlers 使用 dev.role.*，對應 admin.role.manage 細分）
        ("dev.role.create", "建立角色", "dev", "API: 可建立角色"),
        ("dev.role.view", "查看角色", "dev", "API: 可查看角色列表與詳情"),
        ("dev.role.edit", "編輯角色", "dev", "API: 可編輯角色"),
        ("dev.role.delete", "刪除角色", "dev", "API: 可刪除角色"),
        // 邀請管理（R19）
        ("invitation.create", "建立邀請", "invitation", "可建立客戶邀請"),
        ("invitation.view", "查看邀請", "invitation", "可查看邀請列表"),
        ("invitation.revoke", "撤銷邀請", "invitation", "可撤銷待接受的邀請"),
        ("invitation.resend", "重新發送邀請", "invitation", "可重新發送邀請 Email"),
        // GLP 合規模組 (Migration 016)
        ("glp.study_director.designate", "指定 Study Director", "glp", "可指定研究之 Study Director"),
        // 2026-09-05：本碼從未接上任何檢查（見 docs/reviews/2026-09-03-code-side-issues.md P0-1）。
        // 最終報告簽署已改走身分即授權（actor.id == protocol.study_director_user_id，
        // 見 GlpComplianceService::sign_study_report），比照 protocol_closure 不設 admin 例外——
        // SD 不是全域角色，任何角色授予都表達不出「該報告的 SD 才能簽」。
        // 保留此定義供歷史追溯，實際不再由任何 handler 檢查；待 P2-7 死碼清理一併移除。
        ("glp.study_report.sign", "簽署最終報告", "glp", "Study Director 簽署最終研究報告"),
        ("glp.compliance.overview", "GLP 遵循總覽", "glp", "查看 GLP 遵循狀態儀表板"),
        ("glp.management_review.view", "查看管理審查", "glp", "檢視管理審查紀錄"),
        ("glp.management_review.manage", "管理管理審查", "glp", "建立、編輯管理審查"),
        // 文件控制系統 (DMS)
        ("dms.document.view", "查看受控文件", "dms", "檢視受控文件列表與詳情"),
        ("dms.document.manage", "管理受控文件", "dms", "建立、編輯、審核受控文件"),
        ("dms.document.approve", "核准受控文件", "dms", "核准受控文件發行"),
        // 風險管理
        ("risk.register.view", "查看風險登記簿", "risk", "檢視風險評估紀錄"),
        ("risk.register.manage", "管理風險登記簿", "risk", "建立、評估、處理風險"),
        // 變更控制
        ("change.request.view", "查看變更申請", "change", "檢視變更申請紀錄"),
        ("change.request.manage", "管理變更申請", "change", "建立、提交變更申請"),
        ("change.request.approve", "核准變更申請", "change", "審核並核准變更申請"),
        // 環境監控
        ("env.monitoring.view", "查看環境監控", "env", "檢視環境監控點與紀錄"),
        ("env.monitoring.manage", "管理環境監控", "env", "建立監控點、登錄環境數據"),
        // 能力評鑑
        ("competency.assessment.view", "查看能力評鑑", "competency", "檢視能力評鑑紀錄"),
        ("competency.assessment.manage", "管理能力評鑑", "competency", "建立、執行能力評鑑"),
        // 最終報告
        ("study.report.view", "查看最終報告", "study", "檢視研究最終報告（無此權限者仍可檢視本人擔任 SD 的計畫報告，見 can_view_study_report）"),
        // 2026-09-05：撰寫／編輯報告本文改走身分即授權（見 glp.study_report.sign 的同一則註解），
        // 不再由任何 handler 檢查此碼；保留定義待 P2-7 死碼清理一併移除。
        ("study.report.manage", "管理最終報告", "study", "建立、編輯研究最終報告"),
        ("qau.report_statement.write", "填寫 QAU 品保聲明", "qau", "GLP 最終報告 QAU 品保聲明填寫，與報告本文分開授權；同時要求填寫者不得為該計畫 SD 本人"),
        // 配製紀錄
        ("formulation.record.view", "查看配製紀錄", "formulation", "檢視試驗物質配製紀錄"),
        ("formulation.record.manage", "管理配製紀錄", "formulation", "建立、編輯配製紀錄"),
        // R40-A 站內信
        ("messaging.send", "使用站內信", "messaging", "可寄送、接收站內信（受 access matrix 限制）"),
        ("messaging.admin_view", "管理員查看任意對話", "messaging", "可讀取所有 thread / message 內容（per R40-7 admin 全可看）"),
        // ────────────────────────────────────────────────────────────────
        // 死權限碼補齊（2026-08-08，docs/audit/dead-permission-codes-2026-08-08.md）
        //
        // 以下 11 個碼被 handler 的 require_permission! / has_permission 檢查，
        // 卻從來不在 permissions 表裡 —— has_permission 對它們永遠回 false，
        // 功能只靠 is_admin() 短路才能用。「只有管理員做得到」是意外，不是設計。
        //
        // 根因：下方 ensure_all_role_permissions 的授予 SQL 是
        //   INSERT ... SELECT ... FROM roles CROSS JOIN permissions WHERE p.code = ANY($2)
        // JOIN 的是 permissions 表；清單裡有不存在的碼時 JOIN 不產生列，
        // 沒有錯誤也沒有警告。防呆見 tests/permission_codes_exist.rs。
        //
        // ⚠️ 補進目錄本身**不改變任何人的權限**（沒有角色被授予，仍只有管理員通得過），
        // 唯一例外是 aup.review.reply —— 它早已寫在五個角色的授予清單裡，
        // 補進目錄後那五筆授予才會真的生效（使用者 2026-08-08 裁定：補齊讓它生效）。
        ("facility.manage", "管理設施", "facility", "可新增 / 編輯 / 刪除建築、區域、欄舍等設施資料"),
        ("system.admin", "系統管理", "system", "系統層級管理操作（AI / agent 端點）"),
        ("admin.treatment_drug.view", "查看治療用藥主檔", "admin", "可查看治療用藥主檔"),
        ("admin.treatment_drug.create", "建立治療用藥", "admin", "可新增治療用藥主檔項目"),
        ("admin.treatment_drug.edit", "編輯治療用藥", "admin", "可編輯治療用藥主檔項目"),
        ("admin.treatment_drug.delete", "刪除治療用藥", "admin", "可刪除治療用藥主檔項目"),
        ("erp.product.delete", "刪除產品", "erp", "可刪除產品主檔"),
        ("erp.partner.delete", "刪除夥伴", "erp", "可刪除夥伴主檔"),
        ("hr.attendance.manage", "管理出勤紀錄", "hr", "可代員工新增 / 修改出勤紀錄"),
        ("animal.euthanasia.create", "開立安樂死單", "animal", "可開立安樂死單據"),
        ("aup.review.reply", "回覆審查意見", "aup", "可回覆被指派的審查意見（非計畫擁有者亦可）"),
        // 同一批漏補：這兩個碼同樣寫在 PI / IACUC_STAFF / EXPERIMENT_STAFF /
        // INTERN 四個角色的授予清單裡，也同樣靜默落空。
        // 差別是目前**沒有任何 handler 檢查它們**（附件上傳/刪除走其他授權路徑），
        // 所以補上不改變任何行為 —— 但留著不補，防呆測試會一直紅，
        // 且日後有人真的拿它們來上閘時又會踩同一個坑。
        // 這兩個是 permission_codes_exist 測試寫完後**當場抓到**的，
        // 不在 2026-08-08 首次人工掃描的 11 個名單內（那次只掃了被檢查的碼）。
        ("aup.attachment.upload", "上傳計畫附件", "aup", "可上傳計畫書附件"),
        ("aup.attachment.delete", "刪除計畫附件", "aup", "可刪除計畫書附件"),
        // ────────────────────────────────────────────────────────────────
        // 11 處 role 硬判改 permission code（2026-08-09）
        //
        // 這 4 個是原本用 has_role() 直判、無對應權限碼可重用的識別型檢查，
        // 補上等價權限碼後改用 has_permission()，讓授權集中於 permissions 表
        // （可稽核、可透過角色管理頁調整），不再散落於程式碼字串比對角色名。
        ("erp.document.view_all", "查看所有單據", "erp", "可查看所有建立者的單據列表（不限本人建立），供倉庫管理員監督用"),
        ("aup.protocol.vet_review", "填寫獸醫審查表", "aup", "可為任一計畫填寫獸醫審查表（不限被指派審查該計畫的獸醫）"),
        ("aup.review.identity_view", "檢視審查者真實身分", "aup", "盲審例外：可檢視審查意見的真實審查者姓名/email，其餘角色僅見匿名代稱"),
        // R89-8（2026-08-13）：計畫擁有人（PI/SD）結案自己計畫的窄縫通道。
        // aup.protocol.change_status 開放太多狀態轉換能力（審查流程各階段），僅限
        // IACUC_CHAIR / IACUC_STAFF；PI/SD 只該能把「自己」已核准的計畫轉為「已結案」，
        // 實際的本人身分與狀態限制在 ProtocolService::change_status_tx 內驗證。
        ("aup.protocol.close_own", "結案自己的計畫", "aup", "計畫擁有人（PI/SD）可將自己已核准（含附條件）的計畫結案，不含其他狀態轉換能力"),
    ];

    for (code, name, module, description) in required_permissions {
        // ON CONFLICT DO UPDATE（而非 DO NOTHING）：已存在的權限碼若改了中文名稱/
        // module/說明文字，啟動時同步進資料庫，而非永遠停留在當初補進去那一版的
        // 舊文字（CodeRabbit PR #73：animal.euthanasia.create 的說明文字曾因
        // DO NOTHING 而卡在「現行 handler 另接受 ROLE_VET」的過期敘述）。
        // 不動 id / created_at，只同步人類可讀的目錄欄位；role_permissions 授予
        // 不受影響。
        sqlx::query(
            r#"
            INSERT INTO permissions (id, code, name, module, description, created_at)
            VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW())
            ON CONFLICT (code) DO UPDATE
            SET name = EXCLUDED.name, module = EXCLUDED.module, description = EXCLUDED.description
        "#,
        )
        .bind(code)
        .bind(name)
        .bind(module)
        .bind(description)
        .execute(pool)
        .await?;
    }

    // 將 admin.data.export / admin.data.import / dev.role.* 指派給 admin 角色
    sqlx::query(
        r#"
        INSERT INTO role_permissions (role_id, permission_id)
        SELECT r.id, p.id FROM roles r, permissions p
        WHERE r.code = 'admin' AND p.code IN (
            'admin.data.export', 'admin.data.import',
            'dev.role.create', 'dev.role.view', 'dev.role.edit', 'dev.role.delete',
            'erp.adj.approve',
            'invitation.create', 'invitation.view', 'invitation.revoke', 'invitation.resend',
            'animal.animal.delete',
            'aup.protocol.close_own',
            -- R53-2: admin 也可檢視 / 編輯廢棄物再利用紀錄（QA 後援）
            'animal.byproduct_sample.view', 'animal.byproduct_sample.write'
        )
        ON CONFLICT (role_id, permission_id) DO NOTHING
    "#,
    )
    .execute(pool)
    .await?;

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
        (
            "WAREHOUSE_MANAGER",
            vec![
                // 倉庫管理
                "erp.warehouse.view",
                "erp.warehouse.create",
                "erp.warehouse.edit",
                "erp.warehouse.delete",
                // 產品管理
                "erp.product.view",
                "erp.product.create",
                "erp.product.edit",
                // 夥伴管理
                "erp.partner.view",
                "erp.partner.create",
                "erp.partner.edit",
                // 儲位管理（完整 CRUD）
                "erp.storage.view",
                "erp.storage.create",
                "erp.storage.edit",
                "erp.storage.delete",
                "erp.storage.inventory.view",
                "erp.storage.inventory.edit",
                // 單據管理
                "erp.document.view",
                "erp.document.create",
                "erp.document.edit",
                "erp.document.submit",
                "erp.document.approve",
                "erp.document.cancel",
                "erp.document.delete",
                // 採購
                "erp.purchase.create",
                "erp.purchase.approve",
                "erp.grn.create",
                "erp.pr.create",
                // 庫存操作
                "erp.stock.in",
                "erp.stock.out",
                "erp.stock.view",
                "erp.stock.adjust",
                "erp.stock.transfer",
                "erp.stocktake.create",
                // 報表
                "erp.report.view",
                "erp.report.export",
                "erp.report.download",
                // 單據監督（可查看所有建立者的單據，不限本人建立）
                "erp.document.view_all",
                // Dashboard
                "dashboard.view",
            ],
        ),
        // ============================================
        // PURCHASING (採購人員) - ERP 採購相關權限
        // ============================================
        (
            "PURCHASING",
            vec![
                // 基本查詢
                "erp.warehouse.view",
                "erp.product.view",
                // 夥伴管理
                "erp.partner.view",
                "erp.partner.create",
                "erp.partner.edit",
                // 單據管理
                "erp.document.view",
                "erp.document.create",
                "erp.document.edit",
                "erp.document.submit",
                // 採購
                "erp.purchase.create",
                "erp.grn.create",
                "erp.pr.create",
                // 庫存查詢
                "erp.stock.view",
                // 報表
                "erp.report.view",
                // Dashboard
                "dashboard.view",
            ],
        ),
        // ============================================
        // PI (計畫主持人) - 計畫管理、動物查看
        // ============================================
        (
            "PI",
            vec![
                // 計畫管理
                "aup.protocol.view_own",
                "aup.protocol.create",
                "aup.protocol.edit",
                "aup.protocol.submit",
                "aup.protocol.delete",
                // R89-8：結案自己已核准的計畫（窄縫，不含其他狀態轉換能力）
                "aup.protocol.close_own",
                // 審查流程
                "aup.review.view",
                "aup.review.reply",
                // 附件管理（含刪除自己的附件）
                "aup.attachment.view",
                "aup.attachment.download",
                "aup.attachment.upload",
                "aup.attachment.delete",
                // 版本管理（含還原）
                "aup.version.view",
                "aup.version.restore",
                // 動物管理
                "animal.animal.view_project",
                "animal.record.view",
                // 匯出
                "animal.export.medical",
                "animal.export.observation",
                "animal.export.surgery",
                // Dashboard
                "dashboard.view",
            ],
        ),
        // ============================================
        // VET (獸醫師) - 審查計畫、動物查看、獸醫建議、緊急處置
        // 只看、給建議，不參與現場工作
        // ============================================
        (
            "VET",
            vec![
                // AUP 計畫審查
                "aup.protocol.view_all",
                "aup.protocol.view_own",
                "aup.protocol.review",
                "aup.review.view",
                "aup.review.comment",
                // AUP 附件
                "aup.attachment.view",
                "aup.attachment.download",
                // AUP 版本
                "aup.version.view",
                // Amendment 變更申請（審查、檢視、決定）
                "amendment.read",
                "amendment.review",
                "aup.amendment.approve", // H6：可對被指派的修正案投票
                // 動物管理（只看）
                "animal.animal.view_all",
                "animal.animal.view_project",
                "animal.record.view",
                // 匯出（所有紀錄）
                "animal.export.medical",
                "animal.export.observation",
                "animal.export.surgery",
                "animal.export.experiment",
                // 獸醫師功能（所有）
                "animal.vet.recommend",
                "animal.vet.read",
                // 緊急處置
                "animal.emergency.stop",
                "animal.euthanasia.recommend",
                "animal.euthanasia.approve",
                // 安樂死開立 / 執行
                "animal.euthanasia.create",
                "animal.euthanasia.execute",
                // 獸醫審查表填寫（不限被指派審查該計畫）
                "aup.protocol.vet_review",
                // 盲審例外：可見審查者真實身分
                "aup.review.identity_view",
                // R53-2: 廢棄物再利用紀錄（採樣 / 檢視 / 修改）
                "animal.byproduct_sample.view",
                "animal.byproduct_sample.write",
                // Dashboard
                "dashboard.view",
            ],
        ),
        // ============================================
        // REVIEWER (審查委員) - 查看所有計畫、計畫審查權限
        // ============================================
        (
            "REVIEWER",
            vec![
                // 計畫審查（查看所有計畫）
                "aup.protocol.view_all",
                "aup.protocol.view_own",
                "aup.protocol.review",
                // 審查流程
                "aup.review.view",
                "aup.review.comment",
                // 附件管理
                "aup.attachment.view",
                "aup.attachment.download",
                // 版本管理
                "aup.version.view",
                // Amendment 變更申請（審查、檢視、決定）
                "amendment.read",
                "amendment.review",
                "aup.amendment.approve", // H6：可對被指派的修正案投票
                // 動物紀錄查看（血檢分析等，與動物權限綁定）
                "animal.animal.view_all",
                "animal.record.view",
                // 盲審例外：可見審查者真實身分
                "aup.review.identity_view",
                // Dashboard
                "dashboard.view",
            ],
        ),
        // ============================================
        // IACUC_CHAIR (IACUC 主席) - 計畫核准、審查人員指派、安樂死仲裁
        // 注意：IACUC_CHAIR 不是公司員工，無 HR 權限
        // ============================================
        (
            "IACUC_CHAIR",
            vec![
                // 計畫管理
                "aup.protocol.view_all",
                "aup.protocol.view_own",
                "aup.protocol.review",
                "aup.protocol.approve",
                "aup.protocol.change_status",
                // Amendment 決定（H6：CHAIR 為終決者，需有此權限）
                "aup.amendment.approve",
                // 審查流程
                "aup.review.view",
                "aup.review.comment",
                "aup.review.assign",
                // 附件管理
                "aup.attachment.view",
                "aup.attachment.download",
                // 版本管理
                "aup.version.view",
                // 動物管理 - 僅查看，不含來源管理
                "animal.animal.view_all",
                "animal.record.view",
                // 安樂死仲裁權限（IACUC_CHAIR 為最終決策者）
                "animal.euthanasia.approve",
                "animal.euthanasia.arbitrate",
                // 盲審例外：可見審查者真實身分
                "aup.review.identity_view",
                // Dashboard
                "dashboard.view",
            ],
        ),
        // ============================================
        // IACUC_STAFF (執行秘書) - 所有 AUP 權限
        // ============================================
        (
            "IACUC_STAFF",
            vec![
                // 動物預約與試驗規劃：檢視 + 操作（執秘是唯一有操作權的角色）
                "animal.planning.view",
                "animal.planning.manage",
                // 動物轉讓的協調段（發起 / 指定新計畫 / 完成 / 拒絕）。
                // P0-3：執秘是這條流程的協調者，獸醫評估與 PI 同意各自另有權責把關。
                "animal.transfer.manage",
                // AUP 計畫管理：執秘對計畫內容唯讀（不含 edit / submit，對齊原始 spec §4.1
                // 「編輯草稿 / 提交計畫 ✗」）；保留審查指派 / 核准 / 變更狀態等協調權。
                "aup.protocol.view_all",
                "aup.protocol.view_own",
                "aup.protocol.create",
                "aup.protocol.review",
                "aup.protocol.approve",
                "aup.protocol.change_status",
                "aup.protocol.delete",
                // AUP 審查流程（全部）
                "aup.review.view",
                "aup.review.assign",
                "aup.review.comment",
                "aup.review.reply",
                // AUP 附件管理（全部）
                "aup.attachment.view",
                "aup.attachment.download",
                "aup.attachment.upload",
                "aup.attachment.delete",
                // AUP 版本管理
                "aup.version.view",
                "aup.version.restore",
                // AUP 額外功能
                "aup.amendment.classify", // 分類修正案（執行秘書負責判斷 Major/Minor）
                // 盲審例外：可見審查者真實身分
                "aup.review.identity_view",
                // 動物批次分配至計畫（IACUC 執行秘書在審查核准後執行分配）
                "animal.info.assign",
                // 邀請管理（R19）
                "invitation.create",
                "invitation.view",
                "invitation.revoke",
                "invitation.resend",
                // Dashboard
                "dashboard.view",
            ],
        ),
        // ============================================
        // EXPERIMENT_STAFF (試驗工作人員) - Co-Editor 協助編輯、動物紀錄、ERP 查詢
        // ============================================
        (
            "EXPERIMENT_STAFF",
            vec![
                // 動物預約與試驗規劃：僅檢視（操作限執秘）。SD 由本名單指派而來，
                // 故全體試驗工作人員都給檢視權。
                "animal.planning.view",
                // 計畫管理：唯讀全覽（R76-2 拆除 CO_EDITOR 後，內容編輯改為 SD 制——
                // 是不是計畫的 study_director_user_id 由 can_edit_protocol 逐筆判斷，
                // 不再靠這裡的角色權限），不可自行建立/提交計畫。
                "aup.protocol.view_own",
                "aup.protocol.view_all",
                // 匯入已核准計畫（場內既有已通過計劃補登，跳過審查）
                "aup.protocol.import_approved",
                // R98-1（2026-08-25，使用者裁定 (a)）：結案自己擔任 SD 的計畫。
                //
                // SD 只存在於 `protocols.study_director_user_id`，指派資格由
                // `validate_and_authorize_sd` 把關為 EXPERIMENT_STAFF——但這個角色
                // 原本沒有 close_own，於是 `services/protocol/status.rs` 的順序
                //（先驗權限碼、才驗擁有人）讓「只有 SD 身分」的人永遠停在第一關 403，
                // 連同檔的 `study_director_user_id == u.id` 擁有人判定都走不到。
                // 實測 6 位現任 SD 有 5 位卡在這裡，25 份已核准計畫的 SD 按不下結案。
                //
                // ⚠️ 授予範圍看似比實際能力大，那是刻意的取捨：權限碼給全體
                // EXPERIMENT_STAFF，但實際能結案的範圍仍被 status.rs 的三道檢查鎖死
                //——① 狀態必須是 APPROVED/APPROVED_WITH_CONDITIONS；
                // ② 必須是該計畫的 pi_user_id / study_director_user_id 本人，或
                // user_protocols 的委派 PI；③ 不符合就 403。所以持有此碼的人
                // **只能結自己被指派為 SD 的那幾份**，不能碰別人的。
                //
                // 選這條而非「在 status.rs 對 SD 身分豁免權限碼檢查」，是為了維持
                // 本 repo「權限碼是唯一授權來源」的慣例：豁免會在 service 層開一條
                // 不經角色的授權路徑，日後稽核權限表時看不到這個能力。
                "aup.protocol.close_own",
                // 審查流程
                "aup.review.view",
                "aup.review.reply",
                // 附件管理
                "aup.attachment.view",
                "aup.attachment.download",
                "aup.attachment.upload",
                "aup.attachment.delete",
                // 版本管理
                "aup.version.view",
                // 動物管理 - 可查看所有動物、新增、編輯（含設定 species_id tag）、匯入
                "animal.animal.view_all",
                "animal.animal.create",
                "animal.animal.edit",
                "animal.animal.import",
                "animal.record.view",
                "animal.record.create",
                "animal.record.edit",
                "animal.record.delete",
                "animal.blood_test_template.manage",
                "animal.record.observation",
                "animal.record.surgery",
                "animal.record.weight",
                "animal.record.vaccine",
                "animal.record.sacrifice",
                // 病理報告（需要查看與上傳）
                "animal.pathology.view",
                "animal.pathology.upload",
                // 紀錄複製與緊急觀察
                "animal.record.copy",
                "animal.record.emergency",
                // 獸醫附件上傳
                "animal.vet.upload_attachment",
                // 動物來源管理
                "animal.source.manage",
                // 緊急處置權限
                "animal.emergency.medication",
                "animal.emergency.stop",
                // 安樂死執行權限（需由 PI 或 VET 核准）
                "animal.euthanasia.execute",
                // 匯出（含病歷）
                "animal.export.medical",
                "animal.export.observation",
                "animal.export.surgery",
                "animal.export.experiment",
                // ERP 查詢（僅讀取）+ 採購退貨建立 + 單據建立
                "erp.warehouse.view",
                "erp.product.view",
                "erp.partner.view",
                "erp.stock.view",
                // R81-9：`pr` 指 `DocType::PR`＝**採購退貨**（Purchase Return，見
                // models/document.rs），不是請購單——本系統沒有請購流程。
                // 權限字串本身不改（改動要連 DB seed 一起遷移），只正名說明。
                "erp.pr.create",
                // 單據管理（可建立銷貨單等）
                "erp.document.view",
                "erp.document.create",
                "erp.document.edit",
                "erp.document.submit",
                // HR 權限（內部員工基本權限）
                "hr.attendance.view",
                "hr.attendance.clock",
                "hr.leave.view",
                "hr.leave.create",
                "hr.overtime.view",
                "hr.overtime.create",
                "hr.balance.view",
                "hr.calendar.view",
                "hr.leave.view_calendar",
                // 人員訓練紀錄（僅管理自己的）
                "training.view",
                "training.manage_own",
                // Dashboard 權限
                "dashboard.view",
            ],
        ),
        // ============================================
        // INTERN (實習生) - 帳號有到期日；計畫書唯讀範圍與 EXPERIMENT_STAFF 相同，
        // 其餘模組（設備/ERP倉儲/配方/修正案等）較窄，見下方個別權限
        // ============================================
        (
            "INTERN",
            vec![
                // 計畫管理：唯讀全覽（與 EXPERIMENT_STAFF 同，見該處註解；不含
                // import_approved——跳過審查的匯入權限維持 EXPERIMENT_STAFF 專屬）
                "aup.protocol.view_own",
                "aup.protocol.view_all",
                // 審查流程
                "aup.review.view",
                "aup.review.reply",
                // 附件管理
                "aup.attachment.view",
                "aup.attachment.download",
                "aup.attachment.upload",
                "aup.attachment.delete",
                // 版本管理
                "aup.version.view",
                // 動物管理（含設定 species_id tag）
                "animal.animal.view_all",
                "animal.animal.create",
                "animal.animal.edit",
                "animal.animal.import",
                "animal.record.view",
                "animal.record.create",
                "animal.record.edit",
                "animal.record.delete",
                "animal.blood_test_template.manage",
                "animal.record.observation",
                "animal.record.surgery",
                "animal.record.weight",
                "animal.record.vaccine",
                "animal.record.sacrifice",
                // 病理報告（需要查看與上傳）
                "animal.pathology.view",
                "animal.pathology.upload",
                // 紀錄複製與緊急觀察
                "animal.record.copy",
                "animal.record.emergency",
                // 獸醫附件上傳
                "animal.vet.upload_attachment",
                // 動物來源管理
                "animal.source.manage",
                // 緊急處置權限
                "animal.emergency.medication",
                "animal.emergency.stop",
                // 安樂死執行權限（需由 PI 或 VET 核准）
                "animal.euthanasia.execute",
                // 匯出（含病歷）
                "animal.export.medical",
                "animal.export.observation",
                "animal.export.surgery",
                "animal.export.experiment",
                // ERP 查詢（僅讀取）+ 採購退貨建立 + 單據建立（`pr` 見上方說明：採購退貨，非請購單）
                "erp.warehouse.view",
                "erp.product.view",
                "erp.partner.view",
                "erp.stock.view",
                "erp.pr.create",
                // 單據管理
                "erp.document.view",
                "erp.document.create",
                "erp.document.edit",
                "erp.document.submit",
                // HR 權限（內部員工基本權限）
                "hr.attendance.view",
                "hr.attendance.clock",
                "hr.leave.view",
                "hr.leave.create",
                "hr.overtime.view",
                "hr.overtime.create",
                "hr.balance.view",
                "hr.calendar.view",
                "hr.leave.view_calendar",
                // 人員訓練紀錄（僅管理自己的）
                "training.view",
                "training.manage_own",
                // Dashboard 權限
                "dashboard.view",
            ],
        ),
        // ============================================
        // ADMIN_STAFF (行政) - 全部 HR 權限 + 庫存報表 Audit + 管理階級 Audit
        // ============================================
        (
            "ADMIN_STAFF",
            vec![
                // HR 權限（全部）
                "hr.attendance.view",
                "hr.attendance.view_all",
                "hr.attendance.clock",
                "hr.attendance.correct",
                "hr.overtime.view",
                "hr.overtime.view_all",
                "hr.overtime.create",
                "hr.overtime.approve",
                "hr.leave.view",
                "hr.leave.view_all",
                "hr.leave.create",
                "hr.leave.approve",
                "hr.leave.manage",
                "hr.balance.view",
                "hr.balance.manage",
                "hr.calendar.config",
                "hr.calendar.view",
                "hr.leave.view_calendar",
                "hr.calendar.sync",
                "hr.calendar.conflicts",
                // ERP 倉庫管理權限
                "erp.warehouse.view",
                "erp.warehouse.create",
                "erp.warehouse.edit",
                "erp.warehouse.delete",
                "erp.product.view",
                "erp.product.create",
                "erp.product.edit",
                "erp.partner.view",
                "erp.partner.create",
                "erp.partner.edit",
                // 儲位管理
                "erp.storage.view",
                "erp.storage.create",
                "erp.storage.edit",
                "erp.storage.delete",
                "erp.storage.inventory.view",
                "erp.storage.inventory.edit",
                // 單據管理
                "erp.document.view",
                "erp.document.create",
                "erp.document.edit",
                "erp.document.submit",
                // 庫存報表 Audit 權限
                "erp.stock.view",
                "erp.report.view",
                // 管理階級 Audit 權限（全部 5 個）
                "audit.logs.view",
                "audit.logs.export",
                "audit.timeline.view",
                "audit.alerts.view",
                "audit.alerts.manage",
                // 人員訓練紀錄 (GLP 合規)
                "training.view",
                "training.manage",
                "equipment.view",
                "equipment.manage",
                // Dashboard 權限
                "dashboard.view",
            ],
        ),
        // ============================================
        // DIRECTOR (負責人) - 請假 / 加班 / ERP 終審關 + 自身 HR 使用
        //
        // R97-1c 設計原則（2026-08-17 使用者裁定）：
        //   **負責人是監督與終審，不是操作者。不建單、不改單、不送審。**
        //   除緊急狀況外不碰第一線。
        //
        // 這條原則與 R97-1 的職務分離互相加強：只要負責人從不建單，他就永遠有
        // 資格核准任何單據，不會出現「自己審自己而卡住」的死結。故本清單
        // **刻意不含** create / edit / submit / delete / cancel，也不含倉庫、
        // 產品、夥伴、儲位的任何管理權——那些是倉管的職責。
        //
        // 換負責人時只需移轉 DIRECTOR 角色，權限自動跟著走。
        (
            "DIRECTOR",
            vec![
                // 動物預約與試驗規劃：僅檢視（操作限執秘）
                "animal.planning.view",
                // ERP 終審關：看得到 + 審得了，但不能建單（見上方原則）
                //
                // view_all 是必要的而非額外放寬：負責人不建單，少了它會「什麼都
                // 看不到」——一般 view 只涵蓋自己建立的單據。
                // stock.view 用於審核時判斷合理性（調減 200 個時要知道現在剩多少），
                // 否則終審只是蓋章。
                "erp.document.view",
                "erp.document.view_all",
                "erp.stock.view",
                "erp.document.final_approve",
                "erp.document.reverse_approve",
                // 請假：檢視全體待審 + 終審核准（實際審核授權於 services 依角色判定）
                "hr.leave.view",
                "hr.leave.view_all",
                "hr.leave.create",
                "hr.leave.approve",
                // 加班：檢視全體 + 核准（與請假終審一致的管理視角）
                "hr.overtime.view",
                "hr.overtime.view_all",
                "hr.overtime.approve",
                // 自身出勤 / 餘額 / 行事曆
                "hr.attendance.view",
                "hr.attendance.clock",
                "hr.overtime.create",
                "hr.balance.view",
                "hr.calendar.view",
                "hr.leave.view_calendar",
                // 2026-09-06（P0-1，使用者裁定）：撤銷電子簽章，與 QAU 同時授予。
                // 理由見 QAU 那列的同名註解；負責人這一側的定位是「品保不在時的
                // 第二個可執行者」，而不是把它變成日常操作——本碼的說明文字
                // （`003_seed.sql:329`）本來就寫著「僅供稀有／緊急情境使用」。
                "signature.invalidate",
                // Dashboard
                "dashboard.view",
            ],
        ),
        // ============================================
        // QAU (品質保證單位) - GLP 唯讀檢視 + 計畫管理，獨立於研究執行
        // ============================================
        (
            "QAU",
            vec![
                "qau.dashboard.view",
                "qau.protocol.view",
                "qau.audit.view",
                "qau.animal.view",
                // QA 計畫管理（稽查報告、NC、SOP、稽查排程）
                "qau.inspection.view",
                "qau.inspection.manage",
                "qau.nc.view",
                "qau.nc.manage",
                "qau.sop.view",
                "qau.sop.manage",
                "qau.schedule.view",
                "qau.schedule.manage",
                // 2026-09-06（P0-1，使用者裁定）：撤銷電子簽章。
                // 此碼定義在 `003_seed.sql:329`（不在本檔的 required_permissions 清單），
                // 在此之前**授予零角色**＝只有 admin 靠 has_permission 短路做得到，
                // 而「簽章作廢的執行者＝系統管理員」在 GLP 稽核上站不住：作廢是品保
                // 判斷（簽錯人、離職撤回、key compromise），不是有 root 權限的人該決定的事。
                // 前端入口已存在（`AuditLogsPage` 的「撤銷簽章」鈕 → `InvalidateSignatureDialog`，
                // 要求 signature id + 理由 + 密碼二次確認），授予後即可點得到。
                "signature.invalidate",
                // 2026-09-05：GLP 最終報告——QAU 需要看到報告才能出具品保聲明，
                // 且聲明填寫與報告本文分開授權（study.report.manage 已改走身分即授權，見上）
                "study.report.view",
                "qau.report_statement.write",
                // 跨模組唯讀
                "aup.protocol.view_all",
                "aup.review.view",
                "aup.attachment.view",
                "aup.attachment.download",
                "aup.version.view",
                "audit.logs.view",
                "animal.animal.view_all",
                "animal.record.view",
                // R53-2: 廢棄物再利用紀錄唯讀（QAU 稽核需要）
                "animal.byproduct_sample.view",
                "dashboard.view",
            ],
        ),
        // ============================================
        // EQUIPMENT_MAINTENANCE (設備維護人員) - 設備與校準紀錄管理
        // ============================================
        (
            "EQUIPMENT_MAINTENANCE",
            vec![
                "equipment.view",
                "equipment.manage",
                "equipment.maintenance.manage",
                "equipment.maintenance.review",
                "equipment.plan.manage",
                "equipment.disposal.approve",
                "erp.partner.view",
                "erp.partner.create",
                "erp.partner.edit",
                "training.view",
                "training.manage_own",
                "dashboard.view",
            ],
        ),
        // STUDY_DIRECTOR 角色已移除（2026-08-17）：SD 只存在於「該計畫的 SD」
        // （protocols.study_director_user_id），不存在全域 SD 身分。
        // 角色列與其 role_permissions 由 migration 刪除——本檔的授予同步是
        // additive-only（ON CONFLICT DO NOTHING），從這裡拿掉不會撤銷任何既有列。

        // TEST_FACILITY_MANAGEMENT 已於 migration 029 刪除（2026-04-16）
        // 此角色定位等同 admin，有此需求的人員直接指派 admin 角色。

        // ============================================
        // CLIENT (委託人) - 計畫/動物查看（僅自己相關）
        // ============================================
        (
            "CLIENT",
            vec![
                // 計畫查看
                "aup.protocol.view_own",
                // 審查流程
                "aup.review.view",
                // 附件管理
                "aup.attachment.view",
                "aup.attachment.download",
                // 版本管理
                "aup.version.view",
                // 動物查看
                "animal.animal.view_project",
                "animal.record.view",
                // 匯出
                "animal.export.medical",
                "animal.export.observation",
                "animal.export.surgery",
                // Dashboard
                "dashboard.view",
            ],
        ),
    ];

    let mut total_assigned = 0;

    for (role_code, permissions) in &role_permissions {
        // 為角色分配權限
        let result = sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission_id)
            SELECT r.id, p.id
            FROM roles r
            CROSS JOIN permissions p
            WHERE r.code = $1
            AND p.code = ANY($2::text[])
            ON CONFLICT DO NOTHING
        "#,
        )
        .bind(role_code)
        .bind(&permissions[..])
        .execute(pool)
        .await?;

        let assigned = result.rows_affected();
        if assigned > 0 {
            tracing::debug!(
                "[Permissions] {} -> {} new permissions",
                role_code,
                assigned
            );
            total_assigned += assigned;
        }
    }

    if total_assigned > 0 {
        tracing::info!(
            "[Permissions] ✓ {} total permissions assigned to all roles",
            total_assigned
        );
    } else {
        tracing::info!("[Permissions] ✓ All role permissions already configured");
    }

    // R40-A 站內信權限分配：messaging.send 給所有 messaging-eligible roles，
    // messaging.admin_view 只給 admin。CLIENT/GUEST 不給。
    let messaging_send_roles = [
        "admin",
        "SYSTEM_ADMIN",
        "PI",
        "DIRECTOR",
        "VET",
        "ADMIN_STAFF",
        "EXPERIMENT_STAFF",
        "INTERN",
        "PURCHASING",
        "WAREHOUSE_MANAGER",
        "EQUIPMENT_MAINTENANCE",
        "IACUC_STAFF",
        "IACUC_CHAIR",
        "QAU",
        "REVIEWER",
    ];
    sqlx::query(
        r#"
        INSERT INTO role_permissions (role_id, permission_id)
        SELECT r.id, p.id
        FROM roles r CROSS JOIN permissions p
        WHERE r.code = ANY($1::text[]) AND p.code = 'messaging.send'
        ON CONFLICT DO NOTHING
    "#,
    )
    .bind(&messaging_send_roles[..])
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO role_permissions (role_id, permission_id)
        SELECT r.id, p.id
        FROM roles r CROSS JOIN permissions p
        WHERE r.code IN ('admin', 'SYSTEM_ADMIN') AND p.code = 'messaging.admin_view'
        ON CONFLICT DO NOTHING
    "#,
    )
    .execute(pool)
    .await?;

    // 合併 erp.inventory.view 至 erp.stock.view（啟動時清理，不修改 migration）
    merge_inventory_view_into_stock_view(pool).await?;

    // 收回 EXPERIMENT_STAFF / INTERN 的過期計畫書權限（啟動時清理，不修改 migration）
    revoke_stale_experiment_staff_protocol_permissions(pool).await?;

    Ok(())
}

/// 移除 erp.inventory.view 並合併至 erp.stock.view
/// 從 role_permissions 移除該權限，並從 permissions 表刪除
async fn merge_inventory_view_into_stock_view(pool: &sqlx::PgPool) -> Result<()> {
    let result = sqlx::query(
        r#"
        DELETE FROM role_permissions
        WHERE permission_id = (SELECT id FROM permissions WHERE code = 'erp.inventory.view')
    "#,
    )
    .execute(pool)
    .await?;

    let deleted_rp = result.rows_affected();
    if deleted_rp > 0 {
        tracing::info!(
            "[Permissions] ✓ Removed erp.inventory.view from {} role(s)",
            deleted_rp
        );
    }

    let result = sqlx::query(r#"DELETE FROM permissions WHERE code = 'erp.inventory.view'"#)
        .execute(pool)
        .await?;

    if result.rows_affected() > 0 {
        tracing::info!("[Permissions] ✓ Merged erp.inventory.view into erp.stock.view");
    }

    Ok(())
}

/// 收回 EXPERIMENT_STAFF / INTERN 身上過期的計畫書權限（啟動時清理，2026-08-13）。
///
/// 這份角色權限清單本身只增不減（`INSERT ... ON CONFLICT DO NOTHING`），單純把
/// `aup.protocol.edit` 從上面兩個角色的清單裡拿掉，不會撤銷資料庫既有的授予——
/// 需要這支顯式 DELETE 才會真的收回。
///
/// - `aup.protocol.edit`（兩個角色皆收回）：R76-2 已拆除 CO_EDITOR，計畫內容編輯
///   改為 SD 制（`can_edit_protocol` 逐筆判斷，不看角色權限），這個權限碼繼續留著
///   沒有實際作用，只會讓「還能不能編輯」的認知跟實際行為脫鉤。
/// - `aup.protocol.create` / `aup.protocol.submit`（僅 EXPERIMENT_STAFF）：這兩個
///   權限碼從未出現在上面的角色清單裡，卻存在於 prod 資料庫——本次盤點才發現的
///   歷史 drift（清單曾經比較寬鬆、後來收窄成 Co-Editor 模型，但資料庫的舊授予
///   從未跟著撤銷）。收回後 EXPERIMENT_STAFF 不再能自行建立/提交計畫，回到目前
///   清單真正描述的範圍。
async fn revoke_stale_experiment_staff_protocol_permissions(pool: &sqlx::PgPool) -> Result<()> {
    let result = sqlx::query(
        r#"
        DELETE FROM role_permissions
        WHERE permission_id = (SELECT id FROM permissions WHERE code = 'aup.protocol.edit')
          AND role_id IN (SELECT id FROM roles WHERE code IN ('EXPERIMENT_STAFF', 'INTERN'))
    "#,
    )
    .execute(pool)
    .await?;
    if result.rows_affected() > 0 {
        tracing::info!(
            "[Permissions] ✓ Revoked stale aup.protocol.edit from {} EXPERIMENT_STAFF/INTERN grant(s) (CO_EDITOR removed in R76-2, edit is now SD-based)",
            result.rows_affected()
        );
    }

    let result = sqlx::query(
        r#"
        DELETE FROM role_permissions
        WHERE permission_id IN (
            SELECT id FROM permissions WHERE code IN ('aup.protocol.create', 'aup.protocol.submit')
        )
        AND role_id = (SELECT id FROM roles WHERE code = 'EXPERIMENT_STAFF')
    "#,
    )
    .execute(pool)
    .await?;
    if result.rows_affected() > 0 {
        tracing::info!(
            "[Permissions] ✓ Revoked drifted aup.protocol.create/submit from {} EXPERIMENT_STAFF grant(s) (never in role permission list)",
            result.rows_affected()
        );
    }

    Ok(())
}
