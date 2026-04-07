use axum::{
    routing::{get, post, put},
    Router,
};

use crate::{handlers, AppState};

/// 實驗動物及所有子紀錄路由（觀察/手術/體重/疫苗/安樂死/轉讓/病理/照護/血液檢查）
pub fn routes() -> Router<AppState> {
    Router::new()
        // Animals
        .route(
            "/animals",
            get(handlers::list_animals).post(handlers::create_animal),
        )
        .route("/animals/stats", get(handlers::get_animal_stats))
        .route("/animals/by-pen", get(handlers::list_animals_by_pen))
        .route(
            "/animals/batch/assign",
            post(handlers::batch_assign_animals),
        )
        .route("/animals/vet-comments", get(handlers::get_vet_comments))
        .route(
            "/animals/:id",
            get(handlers::get_animal)
                .put(handlers::update_animal)
                .delete(handlers::delete_animal),
        )
        .route("/animals/:id/delete", post(handlers::delete_animal))
        .route("/animals/:id/events", get(handlers::get_animal_events))
        .route(
            "/animals/:id/vet-read",
            post(handlers::mark_animal_vet_read),
        )
        // 動物欄位修正申請
        .route(
            "/animals/:id/field-corrections",
            get(handlers::list_animal_field_corrections)
                .post(handlers::create_animal_field_correction_request),
        )
        .route(
            "/animals/animal-field-corrections/pending",
            get(handlers::list_pending_animal_field_corrections),
        )
        .route(
            "/animals/animal-field-corrections/:id/review",
            post(handlers::review_animal_field_correction),
        )
        // Animal Records - Observations
        .route(
            "/animals/:id/observations",
            get(handlers::list_animal_observations)
                .post(handlers::create_animal_observation),
        )
        .route(
            "/animals/:id/observations/with-recommendations",
            get(handlers::list_animal_observations_with_recommendations),
        )
        .route(
            "/animals/:id/observations/copy",
            post(handlers::copy_animal_observation),
        )
        .route(
            "/observations/:id",
            get(handlers::get_animal_observation)
                .put(handlers::update_animal_observation)
                .delete(handlers::delete_animal_observation),
        )
        .route(
            "/observations/:id/delete",
            post(handlers::delete_animal_observation),
        )
        .route(
            "/observations/:id/vet-read",
            post(handlers::mark_observation_vet_read),
        )
        .route(
            "/observations/:id/versions",
            get(handlers::get_observation_versions),
        )
        // Animal Records - Surgeries
        .route(
            "/animals/:id/surgeries",
            get(handlers::list_animal_surgeries).post(handlers::create_animal_surgery),
        )
        .route(
            "/animals/:id/surgeries/with-recommendations",
            get(handlers::list_animal_surgeries_with_recommendations),
        )
        .route(
            "/animals/:id/surgeries/copy",
            post(handlers::copy_animal_surgery),
        )
        .route(
            "/surgeries/:id",
            get(handlers::get_animal_surgery)
                .put(handlers::update_animal_surgery)
                .delete(handlers::delete_animal_surgery),
        )
        .route(
            "/surgeries/:id/delete",
            post(handlers::delete_animal_surgery),
        )
        .route(
            "/surgeries/:id/vet-read",
            post(handlers::mark_surgery_vet_read),
        )
        .route(
            "/surgeries/:id/versions",
            get(handlers::get_surgery_versions),
        )
        // Animal Records - Weights
        .route(
            "/animals/:id/weights",
            get(handlers::list_animal_weights).post(handlers::create_animal_weight),
        )
        .route(
            "/weights/:id",
            put(handlers::update_animal_weight).delete(handlers::delete_animal_weight),
        )
        .route("/weights/:id/delete", post(handlers::delete_animal_weight))
        // Animal Records - Vaccinations
        .route(
            "/animals/:id/vaccinations",
            get(handlers::list_animal_vaccinations)
                .post(handlers::create_animal_vaccination),
        )
        .route(
            "/vaccinations/:id",
            put(handlers::update_animal_vaccination)
                .delete(handlers::delete_animal_vaccination),
        )
        .route(
            "/vaccinations/:id/delete",
            post(handlers::delete_animal_vaccination),
        )
        // Animal Records - Sacrifice
        .route(
            "/animals/:id/sacrifice",
            get(handlers::get_animal_sacrifice).post(handlers::upsert_animal_sacrifice),
        )
        // Animal Records - Sudden Death
        .route(
            "/animals/:id/sudden-death",
            get(handlers::get_animal_sudden_death)
                .post(handlers::create_animal_sudden_death),
        )
        // Animal Transfers
        .route(
            "/animals/:id/data-boundary",
            get(handlers::get_animal_data_boundary),
        )
        .route(
            "/animals/:id/transfers",
            get(handlers::list_animal_transfers).post(handlers::initiate_transfer),
        )
        .route("/transfers/:id", get(handlers::get_transfer))
        .route(
            "/transfers/:id/vet-evaluate",
            post(handlers::vet_evaluate_transfer),
        )
        .route(
            "/transfers/:id/vet-evaluation",
            get(handlers::get_transfer_vet_evaluation),
        )
        .route(
            "/transfers/:id/assign-plan",
            put(handlers::assign_transfer_plan),
        )
        .route("/transfers/:id/approve", post(handlers::approve_transfer))
        .route("/transfers/:id/complete", post(handlers::complete_transfer))
        .route("/transfers/:id/reject", post(handlers::reject_transfer))
        // Animal Records - Pathology
        .route(
            "/animals/:id/pathology",
            get(handlers::get_animal_pathology_report)
                .post(handlers::upsert_animal_pathology_report),
        )
        // Animal Records - Care Records
        .route(
            "/animals/:id/care-records",
            get(handlers::list_care_records).post(handlers::create_care_record),
        )
        .route(
            "/care-records/:id",
            put(handlers::update_care_record).delete(handlers::delete_care_record),
        )
        .route(
            "/care-records/:id/delete",
            post(handlers::delete_care_record),
        )
        .route(
            "/observations/:id/care-records",
            get(handlers::list_observation_care_records),
        )
        // Animal Records - Blood Tests
        .route(
            "/animals/:id/blood-tests",
            get(handlers::list_animal_blood_tests)
                .post(handlers::create_animal_blood_test),
        )
        .route(
            "/blood-tests/:id",
            get(handlers::get_animal_blood_test)
                .put(handlers::update_animal_blood_test)
                .delete(handlers::delete_animal_blood_test),
        )
        .route(
            "/blood-tests/:id/delete",
            post(handlers::delete_animal_blood_test),
        )
        // Blood Test Templates
        .route(
            "/blood-test-templates",
            get(handlers::list_blood_test_templates)
                .post(handlers::create_blood_test_template),
        )
        .route(
            "/blood-test-templates/all",
            get(handlers::list_all_blood_test_templates),
        )
        .route(
            "/blood-test-templates/:id",
            put(handlers::update_blood_test_template)
                .delete(handlers::delete_blood_test_template),
        )
        .route(
            "/blood-test-templates/:id/delete",
            post(handlers::delete_blood_test_template),
        )
        // Blood Test Panels
        .route(
            "/blood-test-panels",
            get(handlers::list_blood_test_panels)
                .post(handlers::create_blood_test_panel),
        )
        .route(
            "/blood-test-panels/all",
            get(handlers::list_all_blood_test_panels),
        )
        .route(
            "/blood-test-panels/:id",
            put(handlers::update_blood_test_panel)
                .delete(handlers::delete_blood_test_panel),
        )
        .route(
            "/blood-test-panels/:id/delete",
            post(handlers::delete_blood_test_panel),
        )
        .route(
            "/blood-test-panels/:id/items",
            put(handlers::update_blood_test_panel_items),
        )
        // Blood Test Presets
        .route(
            "/blood-test-presets",
            get(handlers::list_blood_test_presets)
                .post(handlers::create_blood_test_preset),
        )
        .route(
            "/blood-test-presets/all",
            get(handlers::list_all_blood_test_presets),
        )
        .route(
            "/blood-test-presets/:id",
            put(handlers::update_blood_test_preset)
                .delete(handlers::delete_blood_test_preset),
        )
        .route(
            "/blood-test-presets/:id/delete",
            post(handlers::delete_blood_test_preset),
        )
        // Vet Recommendations
        .route(
            "/observations/:id/recommendations",
            get(handlers::get_observation_vet_recommendations)
                .post(handlers::add_observation_vet_recommendation),
        )
        .route(
            "/observations/:id/recommendations/with-attachments",
            post(handlers::add_observation_vet_recommendation_with_attachments),
        )
        .route(
            "/surgeries/:id/recommendations",
            get(handlers::get_surgery_vet_recommendations)
                .post(handlers::add_surgery_vet_recommendation),
        )
        .route(
            "/surgeries/:id/recommendations/with-attachments",
            post(handlers::add_surgery_vet_recommendation_with_attachments),
        )
        // Animal Export
        .route(
            "/animals/:id/export",
            post(handlers::export_animal_medical_data),
        )
        .route(
            "/projects/:iacuc_no/export",
            post(handlers::export_project_medical_data),
        )
        // Gotenberg PDF 匯出
        .route(
            "/animals/:id/export-pdf",
            post(handlers::export_animal_medical_pdf),
        )
        .route(
            "/projects/:iacuc_no/export-pdf",
            post(handlers::export_project_medical_pdf),
        )
        .route(
            "/animals/export-pen-report",
            get(handlers::export_pen_report),
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
}
