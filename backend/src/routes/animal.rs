use axum::{
    routing::{get, patch, post, put},
    Router,
};

use crate::{handlers, AppState};

/// 實驗動物及所有子紀錄路由（觀察/手術/體重/疫苗/安樂死/轉讓/病理/照護/血液檢查）
pub fn routes() -> Router<AppState> {
    Router::new()
        // 動物預約與試驗規劃：預定試驗登記（執行秘書 IACUC_STAFF + admin，Phase 1）
        .route(
            "/planned-experiments",
            get(handlers::list_planned_experiments).post(handlers::create_planned_experiment),
        )
        .route(
            "/planned-experiments/{id}",
            get(handlers::get_planned_experiment)
                .put(handlers::update_planned_experiment)
                .delete(handlers::delete_planned_experiment),
        )
        // 動物預約：批次預約 / 解除 / 搜尋可預約動物（Phase 2）
        .route("/animals/batch/reserve", post(handlers::reserve_animals))
        .route(
            "/animals/batch/unreserve",
            post(handlers::unreserve_animals),
        )
        .route(
            "/animals/reservable",
            get(handlers::list_reservable_animals),
        )
        .route(
            "/reservation-planning",
            get(handlers::get_reservation_planning),
        )
        // 規劃頁備註 inline 編輯（Phase 5）
        .route(
            "/animals/{id}/remark",
            patch(handlers::update_animal_remark),
        )
        // Animals
        .route(
            "/animals",
            get(handlers::list_animals).post(handlers::create_animal),
        )
        .route("/animals/stats", get(handlers::get_animal_stats))
        .route("/animals/available", get(handlers::list_available_pigs))
        .route("/animals/by-pen", get(handlers::list_animals_by_pen))
        .route(
            "/animals/batch/assign",
            post(handlers::batch_assign_animals),
        )
        .route("/animals/vet-comments", get(handlers::get_vet_comments))
        .route(
            "/animals/{id}",
            get(handlers::get_animal)
                .put(handlers::update_animal)
                .delete(handlers::delete_animal),
        )
        .route("/animals/{id}/delete", post(handlers::delete_animal))
        .route("/animals/{id}/events", get(handlers::get_animal_events))
        .route(
            "/animals/{id}/client-info",
            get(handlers::get_animal_client_info),
        )
        .route(
            "/animals/{id}/vet-read",
            post(handlers::mark_animal_vet_read),
        )
        .route(
            "/animals/{id}/vet-advice",
            get(handlers::get_animal_vet_advice).put(handlers::upsert_animal_vet_advice),
        )
        // 獸醫師建議紀錄（唯讀）——內容由巡場報告「完成」時自動歸位（單一來源、單向）。
        // 手動 create/update/delete 已移除：獸醫撰寫入口統一在巡場報告。
        .route(
            "/animals/{id}/vet-advice-records",
            get(handlers::list_vet_advice_records),
        )
        // 動物欄位修正申請
        .route(
            "/animals/{id}/field-corrections",
            get(handlers::list_animal_field_corrections)
                .post(handlers::create_animal_field_correction_request),
        )
        .route(
            "/animals/animal-field-corrections/pending",
            get(handlers::list_pending_animal_field_corrections),
        )
        .route(
            "/animals/animal-field-corrections/{id}/review",
            post(handlers::review_animal_field_correction),
        )
        // Animal Records - Observations
        .route(
            "/animals/{id}/observations",
            get(handlers::list_animal_observations).post(handlers::create_animal_observation),
        )
        .route(
            "/animals/{id}/observations/with-recommendations",
            get(handlers::list_animal_observations_with_recommendations),
        )
        .route(
            "/animals/{id}/observations/copy",
            post(handlers::copy_animal_observation),
        )
        .route(
            "/observations/{id}",
            get(handlers::get_animal_observation)
                .put(handlers::update_animal_observation)
                .delete(handlers::delete_animal_observation),
        )
        .route(
            "/observations/{id}/delete",
            post(handlers::delete_animal_observation),
        )
        .route(
            "/observations/{id}/vet-read",
            post(handlers::mark_observation_vet_read),
        )
        .route(
            "/observations/{id}/versions",
            get(handlers::get_observation_versions),
        )
        // Animal Records - Surgeries
        .route(
            "/animals/{id}/surgeries",
            get(handlers::list_animal_surgeries).post(handlers::create_animal_surgery),
        )
        .route(
            "/animals/{id}/surgeries/with-recommendations",
            get(handlers::list_animal_surgeries_with_recommendations),
        )
        .route(
            "/animals/{id}/surgeries/copy",
            post(handlers::copy_animal_surgery),
        )
        .route(
            "/surgeries/{id}",
            get(handlers::get_animal_surgery)
                .put(handlers::update_animal_surgery)
                .delete(handlers::delete_animal_surgery),
        )
        .route(
            "/surgeries/{id}/delete",
            post(handlers::delete_animal_surgery),
        )
        .route(
            "/surgeries/{id}/vet-read",
            post(handlers::mark_surgery_vet_read),
        )
        .route(
            "/surgeries/{id}/versions",
            get(handlers::get_surgery_versions),
        )
        // R32-A8b：手術紀錄 PDF 匯出（v3 docx → Word COM）
        .route(
            "/surgeries/{id}/export-pdf-v3",
            get(handlers::export_surgery_pdf_v3),
        )
        // Animal Records - Weights
        .route(
            "/animals/{id}/weights",
            get(handlers::list_animal_weights).post(handlers::create_animal_weight),
        )
        .route(
            "/weights/{id}",
            put(handlers::update_animal_weight).delete(handlers::delete_animal_weight),
        )
        .route("/weights/{id}/delete", post(handlers::delete_animal_weight))
        // Animal Records - Vaccinations
        .route(
            "/animals/{id}/vaccinations",
            get(handlers::list_animal_vaccinations).post(handlers::create_animal_vaccination),
        )
        .route(
            "/vaccinations/{id}",
            put(handlers::update_animal_vaccination).delete(handlers::delete_animal_vaccination),
        )
        .route(
            "/vaccinations/{id}/delete",
            post(handlers::delete_animal_vaccination),
        )
        // Animal Records - Sacrifice
        .route(
            "/animals/{id}/sacrifice",
            get(handlers::get_animal_sacrifice).post(handlers::upsert_animal_sacrifice),
        )
        // Animal Records - Sudden Death
        .route(
            "/animals/{id}/sudden-death",
            get(handlers::get_animal_sudden_death).post(handlers::create_animal_sudden_death),
        )
        // Animal Transfers
        .route(
            "/animals/{id}/data-boundary",
            get(handlers::get_animal_data_boundary),
        )
        .route(
            "/animals/{id}/transfers",
            get(handlers::list_animal_transfers).post(handlers::initiate_transfer),
        )
        .route("/transfers/{id}", get(handlers::get_transfer))
        .route(
            "/transfers/{id}/vet-evaluate",
            post(handlers::vet_evaluate_transfer),
        )
        .route(
            "/transfers/{id}/vet-evaluation",
            get(handlers::get_transfer_vet_evaluation),
        )
        .route(
            "/transfers/{id}/assign-plan",
            put(handlers::assign_transfer_plan),
        )
        .route("/transfers/{id}/approve", post(handlers::approve_transfer))
        .route(
            "/transfers/{id}/complete",
            post(handlers::complete_transfer),
        )
        .route("/transfers/{id}/reject", post(handlers::reject_transfer))
        // Animal Records - Pathology
        .route(
            "/animals/{id}/pathology",
            get(handlers::get_animal_pathology_report)
                .post(handlers::upsert_animal_pathology_report),
        )
        // Animal Records - Care Records
        .route(
            "/animals/{id}/care-records",
            get(handlers::list_care_records).post(handlers::create_care_record),
        )
        .route(
            "/care-records/{id}",
            put(handlers::update_care_record).delete(handlers::delete_care_record),
        )
        .route(
            "/care-records/{id}/delete",
            post(handlers::delete_care_record),
        )
        .route(
            "/observations/{id}/care-records",
            get(handlers::list_observation_care_records),
        )
        .route(
            "/surgeries/{id}/care-records",
            get(handlers::list_surgery_care_records),
        )
        // Animal Records - Blood Tests
        .route(
            "/animals/{id}/blood-tests",
            get(handlers::list_animal_blood_tests).post(handlers::create_animal_blood_test),
        )
        .route(
            "/blood-tests/{id}",
            get(handlers::get_animal_blood_test)
                .put(handlers::update_animal_blood_test)
                .delete(handlers::delete_animal_blood_test),
        )
        .route(
            "/blood-tests/{id}/delete",
            post(handlers::delete_animal_blood_test),
        )
        // R30-16: 修正單筆 item（append-only supersede）+ 完整修正歷史
        .route(
            "/blood-tests/{test_id}/items/{item_id}/correct",
            post(handlers::correct_blood_test_item),
        )
        .route(
            "/blood-tests/{test_id}/items/history",
            get(handlers::get_blood_test_item_history),
        )
        // Blood Test Templates
        .route(
            "/blood-test-templates",
            get(handlers::list_blood_test_templates).post(handlers::create_blood_test_template),
        )
        .route(
            "/blood-test-templates/all",
            get(handlers::list_all_blood_test_templates),
        )
        .route(
            "/blood-test-templates/{id}",
            put(handlers::update_blood_test_template).delete(handlers::delete_blood_test_template),
        )
        .route(
            "/blood-test-templates/{id}/delete",
            post(handlers::delete_blood_test_template),
        )
        // Blood Test Panels
        .route(
            "/blood-test-panels",
            get(handlers::list_blood_test_panels).post(handlers::create_blood_test_panel),
        )
        .route(
            "/blood-test-panels/all",
            get(handlers::list_all_blood_test_panels),
        )
        .route(
            "/blood-test-panels/{id}",
            put(handlers::update_blood_test_panel).delete(handlers::delete_blood_test_panel),
        )
        .route(
            "/blood-test-panels/{id}/delete",
            post(handlers::delete_blood_test_panel),
        )
        .route(
            "/blood-test-panels/{id}/items",
            put(handlers::update_blood_test_panel_items),
        )
        // Blood Test Presets
        .route(
            "/blood-test-presets",
            get(handlers::list_blood_test_presets).post(handlers::create_blood_test_preset),
        )
        .route(
            "/blood-test-presets/all",
            get(handlers::list_all_blood_test_presets),
        )
        .route(
            "/blood-test-presets/{id}",
            put(handlers::update_blood_test_preset).delete(handlers::delete_blood_test_preset),
        )
        .route(
            "/blood-test-presets/{id}/delete",
            post(handlers::delete_blood_test_preset),
        )
        // Animal Export
        .route(
            "/animals/{id}/export",
            post(handlers::export_animal_medical_data),
        )
        .route(
            "/projects/{iacuc_no}/export",
            post(handlers::export_project_medical_data),
        )
        // Gotenberg PDF 匯出
        .route(
            "/animals/{id}/export-pdf",
            post(handlers::export_animal_medical_pdf),
        )
        .route(
            "/projects/{iacuc_no}/export-pdf",
            post(handlers::export_project_medical_pdf),
        )
        // 移除：/animals/export-pen-report (R32 後 frontend 改走 vet-patrol/export-v3，
        // pen_inspection.html + handler 已成 dead code)
        .route(
            "/animals/{id}/export-blood-test-pdf",
            get(handlers::export_blood_test_pdf),
        )
        // Import Batches
        .route(
            "/animals/import/batches",
            get(handlers::list_import_batches),
        )
        .route(
            "/animals/import/template/basic",
            get(handlers::download_basic_import_template),
        )
        .route(
            "/animals/import/template/weight",
            get(handlers::download_weight_import_template),
        )
        // 獸醫巡場報告
        .route(
            "/vet-patrol-reports",
            get(handlers::list_vet_patrol_reports).post(handlers::create_vet_patrol_report),
        )
        .route(
            "/vet-patrol-reports/{id}",
            get(handlers::get_vet_patrol_report)
                .put(handlers::update_vet_patrol_report)
                .delete(handlers::delete_vet_patrol_report),
        )
        // 與其他 39 個資源同一個約定：DELETE 為主、POST …/delete 為備用，
        // 備用是為了避開「部分代理／tunnel 對 DELETE 回 405」（見 handlers/warehouse.rs:120）。
        // ⚠️ 2026-09-08 之前這裡**只有** POST 這一條，是全系統 40 條 POST …/delete 裡
        // 唯一「base 沒有 DELETE 方法」的例外。照字面把「備用路由」整批移除會直接刪掉
        // 巡場報告的唯一刪除途徑，所以補上 DELETE 讓它回到約定內。
        .route(
            "/vet-patrol-reports/{id}/delete",
            post(handlers::delete_vet_patrol_report),
        )
        .route(
            "/vet-patrol-reports/{id}/export-pdf",
            post(handlers::export_vet_patrol_report_pdf),
        )
        // R39++ 三階段流程
        .route(
            "/vet-patrol-reports/{id}/submit-for-followup",
            post(handlers::submit_for_followup),
        )
        .route(
            "/vet-patrol-reports/{id}/acknowledge",
            post(handlers::acknowledge_receipt),
        )
        .route(
            "/vet-patrol-reports/{id}/complete-followup",
            post(handlers::complete_followup),
        )
        .route(
            "/vet-patrol-reports/{id}/discard",
            post(handlers::discard_vet_patrol_draft),
        )
        .route(
            "/vet-patrol-reports/{id}/retract",
            post(handlers::retract_vet_patrol_report),
        )
        .route(
            "/vet-patrol-reports/{id}/photos",
            get(handlers::list_vet_patrol_photos).post(handlers::upload_vet_patrol_photo),
        )
        .route(
            "/vet-patrol-photos/{photo_id}",
            put(handlers::update_vet_patrol_photo_caption)
                .delete(handlers::delete_vet_patrol_photo),
        )
        .route(
            "/vet-patrol-photos/{photo_id}/download",
            get(handlers::download_vet_patrol_photo),
        )
        // R39: entry-level 照片附件
        .route(
            "/vet-patrol-entries/{entry_id}/photos",
            get(handlers::list_vet_patrol_entry_photos)
                .post(handlers::upload_vet_patrol_entry_photo),
        )
        .route(
            "/vet-patrol-entry-photos/{photo_id}",
            put(handlers::update_vet_patrol_entry_photo_caption)
                .delete(handlers::delete_vet_patrol_entry_photo),
        )
        .route(
            "/vet-patrol-entry-photos/{photo_id}/download",
            get(handlers::download_vet_patrol_entry_photo),
        )
        // R32-A3b 收尾：v3 (xlsx 範本 + Gotenberg LibreOffice)
        .route(
            "/animals/vet-patrol/export-v3",
            get(handlers::export_vet_patrol_v3),
        )
        // R53-4 廢棄物再利用紀錄（byproduct samples）
        .route(
            "/euthanasia/{euth_id}/byproduct-samples",
            post(handlers::create_byproduct_sample)
                .get(handlers::list_byproduct_samples_by_euthanasia),
        )
        .route(
            "/animals/{animal_id}/byproduct-samples",
            get(handlers::list_byproduct_samples_by_animal)
                // #445：計劃內犧牲路徑建立（不需安樂死單）
                .post(handlers::create_byproduct_sample_for_animal),
        )
        .route(
            "/protocols/{protocol_id}/byproduct-samples",
            get(handlers::list_byproduct_samples_by_protocol),
        )
        .route(
            "/byproduct-samples/{id}",
            get(handlers::get_byproduct_sample)
                .patch(handlers::update_byproduct_sample)
                .delete(handlers::delete_byproduct_sample),
        )
}
