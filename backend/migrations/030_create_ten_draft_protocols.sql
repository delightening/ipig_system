-- ============================================
-- 遷移 030: 創建10個測試協議（草稿狀態）
-- ============================================

DO $$
DECLARE
    v_user_id UUID;
    v_protocol_id UUID;
    v_protocol_no VARCHAR(50);
    v_roc_year INTEGER;
    v_seq INTEGER;
    v_protocol_data JSONB;
BEGIN
    -- 獲取管理員用戶 ID (admin@ipig.local)
    SELECT id INTO v_user_id FROM users WHERE email = 'admin@ipig.local' LIMIT 1;
    
    IF v_user_id IS NULL THEN
        -- 如果找不到管理員，則使用第一個可用的用戶
        SELECT id INTO v_user_id FROM users LIMIT 1;
        IF v_user_id IS NULL THEN
            RAISE EXCEPTION 'No users found in database';
        END IF;
    END IF;
    
    -- 生成協議編號的基礎資訊
    SELECT EXTRACT(YEAR FROM NOW()) - 1911 INTO v_roc_year;
    
    -- 協議1: 營養代謝研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻營養代謝研究",
            "apply_study_number": "",
            "start_date": "2025-10-01",
            "end_date": "2026-03-31",
            "project_type": "研究",
            "project_category": "營養科學",
            "test_item_type": "營養添加劑",
            "tech_categories": ["代謝評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "營養測試PI", "phone": "0912-345-678", "email": "nutrition.pi@test.com", "address": "台北市信義區測試大道100號"},
            "sponsor": {"name": "營養研究機構", "contact_person": "營養聯絡人", "contact_phone": "02-1111-2222", "contact_email": "nutrition@test.com"},
            "sd": {"name": "營養專案主持人", "email": "nutrition.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估新型營養添加劑對豬隻代謝效率的影響",
            "replacement": {"rationale": "代謝研究需完整消化系統", "alt_search": {"platforms": ["PubMed"], "keywords": "porcine nutrition metabolism", "conclusion": "需活體代謝系統"}},
            "reduction": {"design": "對照組與試驗組，每組12頭", "grouping_plan": [{"group_name": "對照組", "n": 12, "treatment": "標準飼料", "timepoints": "每日觀察"}, {"group_name": "試驗組", "n": 12, "treatment": "添加營養劑", "timepoints": "每日觀察"}]},
            "duplicate": {"experiment": false, "justification": "初步研究"}
        },
        "items": {"use_test_item": true, "test_items": [{"name": "營養添加劑F", "source": "測試供應商", "purity": "95%", "storage": "常溫保存"}], "control_items": [{"name": "標準飼料", "source": "標準配方", "purity": "N/A", "storage": "常溫保存"}]},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "每日餵食、記錄採食量、每週測量體重、定期採血檢測代謝指標",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 3, "frequency": "每2週一次", "justification": "代謝指標檢測", "total_volume_per_animal": 18}],
            "imaging": [],
            "restraint": [{"method": "徒手保定", "duration": "約3分鐘", "frequency": "採血時", "justification": "採血所需"}],
            "pain": {"category": "無痛或輕微不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成6個月代謝評估", "humane_endpoint": "如出現嚴重代謝異常，終止實驗"},
            "final_handling": {"method": "人道安樂死", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "委由簽約之合格化製廠商進行化製處理\n(化製廠商名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法", "實驗動物管理與使用指引"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 24, "age_min": 8, "age_max": 12, "age_unlimited": false, "weight_min": 25, "weight_max": 35, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 24},
        "personnel": [{"id": 1, "name": "營養研究員1", "position": "研究員", "roles": ["b", "c"], "roles_other_text": "", "years_experience": 3, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "NUTR-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻營養代謝研究', 'DRAFT', v_user_id, v_protocol_data, '2025-10-01'::DATE, '2026-03-31'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議2: 生長性能研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻生長性能研究",
            "apply_study_number": "",
            "start_date": "2025-11-01",
            "end_date": "2026-04-30",
            "project_type": "研究",
            "project_category": "生長科學",
            "test_item_type": "生長促進劑",
            "tech_categories": ["生長評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "生長測試PI", "phone": "0923-456-789", "email": "growth.pi@test.com", "address": "新竹市東區測試街200號"},
            "sponsor": {"name": "生長研究機構", "contact_person": "生長聯絡人", "contact_phone": "03-2222-3333", "contact_email": "growth@test.com"},
            "sd": {"name": "生長專案主持人", "email": "growth.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估不同飼料配方對豬隻生長性能的影響",
            "replacement": {"rationale": "生長性能需完整生長週期觀察", "alt_search": {"platforms": ["Google Scholar"], "keywords": "porcine growth performance", "conclusion": "需完整生長觀察"}},
            "reduction": {"design": "三組不同配方，每組10頭", "grouping_plan": [{"group_name": "對照組", "n": 10, "treatment": "標準配方", "timepoints": "每週測量"}, {"group_name": "試驗組A", "n": 10, "treatment": "配方A", "timepoints": "每週測量"}, {"group_name": "試驗組B", "n": 10, "treatment": "配方B", "timepoints": "每週測量"}]},
            "duplicate": {"experiment": false, "justification": "配方比較研究"}
        },
        "items": {"use_test_item": true, "test_items": [{"name": "生長促進劑G", "source": "測試供應商", "purity": "90%", "storage": "常溫保存"}], "control_items": [{"name": "標準飼料", "source": "標準配方", "purity": "N/A", "storage": "常溫保存"}]},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "每日餵食、每週測量體重及體尺、記錄採食量、計算飼料轉換率",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 2, "frequency": "每月一次", "justification": "生長指標檢測", "total_volume_per_animal": 12}],
            "imaging": [],
            "restraint": [{"method": "徒手保定", "duration": "約2分鐘", "frequency": "測量及採血時", "justification": "操作所需"}],
            "pain": {"category": "無痛或不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成6個月生長評估", "humane_endpoint": "如出現生長異常，終止實驗"},
            "final_handling": {"method": "實驗結束後正常飼養", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "實驗結束後正常飼養，無安樂死"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 30, "age_min": 4, "age_max": 8, "age_unlimited": false, "weight_min": 15, "weight_max": 25, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 30},
        "personnel": [{"id": 1, "name": "生長研究員1", "position": "研究助理", "roles": ["b"], "roles_other_text": "", "years_experience": 2, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "GROW-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻生長性能研究', 'DRAFT', v_user_id, v_protocol_data, '2025-11-01'::DATE, '2026-04-30'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議3: 肉質評估研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻肉質評估研究",
            "apply_study_number": "",
            "start_date": "2025-12-01",
            "end_date": "2026-05-31",
            "project_type": "研究",
            "project_category": "肉品科學",
            "test_item_type": "飼料添加物",
            "tech_categories": ["肉質評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "肉質測試PI", "phone": "0934-567-890", "email": "meat.pi@test.com", "address": "彰化縣員林市測試路300號"},
            "sponsor": {"name": "肉質研究機構", "contact_person": "肉質聯絡人", "contact_phone": "04-3333-4444", "contact_email": "meat@test.com"},
            "sd": {"name": "肉質專案主持人", "email": "meat.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估不同飼養方式對豬隻肉質特性的影響",
            "replacement": {"rationale": "肉質評估需完整肌肉發育", "alt_search": {"platforms": ["PubMed"], "keywords": "porcine meat quality", "conclusion": "需完整肌肉系統評估"}},
            "reduction": {"design": "對照組與試驗組，每組14頭", "grouping_plan": [{"group_name": "對照組", "n": 14, "treatment": "標準飼養", "timepoints": "整個飼養期"}, {"group_name": "試驗組", "n": 14, "treatment": "特殊飼養", "timepoints": "整個飼養期"}]},
            "duplicate": {"experiment": false, "justification": "肉質特性研究"}
        },
        "items": {"use_test_item": true, "test_items": [{"name": "肉質改善劑H", "source": "測試供應商", "purity": "92%", "storage": "常溫保存"}], "control_items": [{"name": "標準飼料", "source": "標準配方", "purity": "N/A", "storage": "常溫保存"}]},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "標準飼養管理、定期測量、最終進行肉質分析",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 4, "frequency": "每月一次", "justification": "生理指標檢測", "total_volume_per_animal": 24}],
            "imaging": [],
            "restraint": [{"method": "徒手保定", "duration": "約3分鐘", "frequency": "採血時", "justification": "採血所需"}],
            "pain": {"category": "無痛或輕微不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成6個月飼養及肉質評估", "humane_endpoint": "如出現嚴重健康問題，終止實驗"},
            "final_handling": {"method": "人道安樂死後進行肉質分析", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "委由簽約之合格化製廠商進行化製處理\n(化製廠商名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法", "實驗動物管理與使用指引"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 28, "age_min": 6, "age_max": 10, "age_unlimited": false, "weight_min": 20, "weight_max": 30, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 28},
        "personnel": [{"id": 1, "name": "肉質研究員1", "position": "研究員", "roles": ["b", "c"], "roles_other_text": "", "years_experience": 4, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "MEAT-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻肉質評估研究', 'DRAFT', v_user_id, v_protocol_data, '2025-12-01'::DATE, '2026-05-31'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議4: 腸道健康研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻腸道健康研究",
            "apply_study_number": "",
            "start_date": "2026-01-01",
            "end_date": "2026-06-30",
            "project_type": "研究",
            "project_category": "腸道科學",
            "test_item_type": "益生菌",
            "tech_categories": ["腸道評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "腸道測試PI", "phone": "0945-678-901", "email": "gut.pi@test.com", "address": "雲林縣斗六市測試街400號"},
            "sponsor": {"name": "腸道研究機構", "contact_person": "腸道聯絡人", "contact_phone": "05-4444-5555", "contact_email": "gut@test.com"},
            "sd": {"name": "腸道專案主持人", "email": "gut.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估益生菌對豬隻腸道健康及微生物群落的影響",
            "replacement": {"rationale": "腸道健康需完整消化系統", "alt_search": {"platforms": ["Google Scholar"], "keywords": "porcine gut health probiotics", "conclusion": "需活體腸道系統"}},
            "reduction": {"design": "對照組與試驗組，每組16頭", "grouping_plan": [{"group_name": "對照組", "n": 16, "treatment": "標準飼料", "timepoints": "每日觀察"}, {"group_name": "試驗組", "n": 16, "treatment": "添加益生菌", "timepoints": "每日觀察"}]},
            "duplicate": {"experiment": false, "justification": "腸道微生物研究"}
        },
        "items": {"use_test_item": true, "test_items": [{"name": "益生菌I", "source": "測試供應商", "purity": "1x10^9 CFU/g", "storage": "2-8°C冷藏保存"}], "control_items": [{"name": "標準飼料", "source": "標準配方", "purity": "N/A", "storage": "常溫保存"}]},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "每日餵食、觀察糞便性狀、定期採集糞便樣本、檢測腸道微生物",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 3, "frequency": "每3週一次", "justification": "腸道健康指標檢測", "total_volume_per_animal": 15}],
            "imaging": [],
            "restraint": [{"method": "徒手保定", "duration": "約2分鐘", "frequency": "採血時", "justification": "採血所需"}],
            "pain": {"category": "無痛或不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成6個月腸道健康評估", "humane_endpoint": "如出現嚴重腸道疾病，終止實驗"},
            "final_handling": {"method": "人道安樂死後進行腸道組織採樣", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "委由簽約之合格化製廠商進行化製處理\n(化製廠商名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 32, "age_min": 6, "age_max": 10, "age_unlimited": false, "weight_min": 20, "weight_max": 30, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 32},
        "personnel": [{"id": 1, "name": "腸道研究員1", "position": "研究員", "roles": ["b", "c"], "roles_other_text": "", "years_experience": 3, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "GUT-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻腸道健康研究', 'DRAFT', v_user_id, v_protocol_data, '2026-01-01'::DATE, '2026-06-30'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議5: 應激反應研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻應激反應研究",
            "apply_study_number": "",
            "start_date": "2026-02-01",
            "end_date": "2026-07-31",
            "project_type": "研究",
            "project_category": "應激科學",
            "test_item_type": "應激管理",
            "tech_categories": ["應激評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "應激測試PI", "phone": "0956-789-012", "email": "stress.pi@test.com", "address": "嘉義市西區測試路500號"},
            "sponsor": {"name": "應激研究機構", "contact_person": "應激聯絡人", "contact_phone": "05-5555-6666", "contact_email": "stress@test.com"},
            "sd": {"name": "應激專案主持人", "email": "stress.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估不同管理方式對豬隻應激反應的影響",
            "replacement": {"rationale": "應激反應需完整神經內分泌系統", "alt_search": {"platforms": ["PubMed"], "keywords": "porcine stress response", "conclusion": "需活體應激系統"}},
            "reduction": {"design": "對照組與試驗組，每組12頭", "grouping_plan": [{"group_name": "對照組", "n": 12, "treatment": "標準管理", "timepoints": "定期檢測"}, {"group_name": "試驗組", "n": 12, "treatment": "應激管理", "timepoints": "定期檢測"}]},
            "duplicate": {"experiment": false, "justification": "應激管理研究"}
        },
        "items": {"use_test_item": false, "test_items": [], "control_items": []},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "模擬應激情境、監測行為反應、檢測應激激素、記錄生理指標",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 4, "frequency": "每2週一次", "justification": "應激激素檢測", "total_volume_per_animal": 20}],
            "imaging": [],
            "restraint": [{"method": "徒手保定", "duration": "約3分鐘", "frequency": "採血時", "justification": "採血所需"}],
            "pain": {"category": "無痛或輕微不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成6個月應激評估", "humane_endpoint": "如出現嚴重應激反應，終止實驗"},
            "final_handling": {"method": "實驗結束後恢復標準管理", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "實驗結束後正常飼養，無安樂死"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法", "實驗動物管理與使用指引"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 24, "age_min": 8, "age_max": 12, "age_unlimited": false, "weight_min": 25, "weight_max": 35, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 24},
        "personnel": [{"id": 1, "name": "應激研究員1", "position": "研究員", "roles": ["b", "c"], "roles_other_text": "", "years_experience": 4, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "STRESS-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻應激反應研究', 'DRAFT', v_user_id, v_protocol_data, '2026-02-01'::DATE, '2026-07-31'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議6: 飼料效率研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻飼料效率研究",
            "apply_study_number": "",
            "start_date": "2026-03-01",
            "end_date": "2026-08-31",
            "project_type": "研究",
            "project_category": "效率科學",
            "test_item_type": "飼料配方",
            "tech_categories": ["效率評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "效率測試PI", "phone": "0967-890-123", "email": "efficiency.pi@test.com", "address": "屏東縣屏東市測試街600號"},
            "sponsor": {"name": "效率研究機構", "contact_person": "效率聯絡人", "contact_phone": "08-6666-7777", "contact_email": "efficiency@test.com"},
            "sd": {"name": "效率專案主持人", "email": "efficiency.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估不同飼料配方對豬隻飼料轉換效率的影響",
            "replacement": {"rationale": "飼料效率需完整消化代謝系統", "alt_search": {"platforms": ["Google Scholar"], "keywords": "porcine feed efficiency", "conclusion": "需活體效率系統"}},
            "reduction": {"design": "四組不同配方，每組8頭", "grouping_plan": [{"group_name": "對照組", "n": 8, "treatment": "標準配方", "timepoints": "每日記錄"}, {"group_name": "試驗組A", "n": 8, "treatment": "配方A", "timepoints": "每日記錄"}, {"group_name": "試驗組B", "n": 8, "treatment": "配方B", "timepoints": "每日記錄"}, {"group_name": "試驗組C", "n": 8, "treatment": "配方C", "timepoints": "每日記錄"}]},
            "duplicate": {"experiment": false, "justification": "配方效率比較"}
        },
        "items": {"use_test_item": true, "test_items": [{"name": "效率提升劑J", "source": "測試供應商", "purity": "88%", "storage": "常溫保存"}], "control_items": [{"name": "標準飼料", "source": "標準配方", "purity": "N/A", "storage": "常溫保存"}]},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "精確記錄每日採食量、每週測量體重、計算飼料轉換率",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 2, "frequency": "每月一次", "justification": "代謝指標檢測", "total_volume_per_animal": 12}],
            "imaging": [],
            "restraint": [{"method": "徒手保定", "duration": "約2分鐘", "frequency": "測量及採血時", "justification": "操作所需"}],
            "pain": {"category": "無痛或不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成6個月效率評估", "humane_endpoint": "如出現效率異常，終止實驗"},
            "final_handling": {"method": "實驗結束後正常飼養", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "實驗結束後正常飼養，無安樂死"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 32, "age_min": 4, "age_max": 8, "age_unlimited": false, "weight_min": 15, "weight_max": 25, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 32},
        "personnel": [{"id": 1, "name": "效率研究員1", "position": "研究助理", "roles": ["b"], "roles_other_text": "", "years_experience": 2, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "EFF-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻飼料效率研究', 'DRAFT', v_user_id, v_protocol_data, '2026-03-01'::DATE, '2026-08-31'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議7: 免疫系統研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": true,
            "registration_authorities": ["TFDA"],
            "study_title": "草稿測試協議 - 豬隻免疫系統研究",
            "apply_study_number": "STUDY-2026-001",
            "start_date": "2026-04-01",
            "end_date": "2026-09-30",
            "project_type": "臨床試驗",
            "project_category": "免疫科學",
            "test_item_type": "免疫調節劑",
            "tech_categories": ["免疫評估"],
            "funding_sources": ["政府補助", "公司內部預算"],
            "pi": {"name": "免疫測試PI", "phone": "0978-901-234", "email": "immune.pi@test.com", "address": "花蓮縣花蓮市測試路700號"},
            "sponsor": {"name": "免疫研究機構", "contact_person": "免疫聯絡人", "contact_phone": "03-7777-8888", "contact_email": "immune@test.com"},
            "sd": {"name": "免疫專案主持人", "email": "immune.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估免疫調節劑對豬隻免疫系統功能的影響",
            "replacement": {"rationale": "免疫系統評估需完整免疫系統", "alt_search": {"platforms": ["PubMed", "ScienceDirect"], "keywords": "porcine immune system", "conclusion": "需活體免疫系統"}},
            "reduction": {"design": "對照組與試驗組，每組15頭", "grouping_plan": [{"group_name": "對照組", "n": 15, "treatment": "標準處理", "timepoints": "定期檢測"}, {"group_name": "試驗組", "n": 15, "treatment": "免疫調節劑", "timepoints": "定期檢測"}]},
            "duplicate": {"experiment": true, "justification": "為驗證結果，進行重複試驗"}
        },
        "items": {"use_test_item": true, "test_items": [{"name": "免疫調節劑K", "source": "內部研發", "purity": "符合藥典標準", "storage": "2-8°C冷藏保存"}], "control_items": [{"name": "標準處理", "source": "商業採購", "purity": "符合藥典標準", "storage": "常溫保存"}]},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "免疫調節劑給藥、定期採血檢測免疫指標、評估免疫反應",
            "route_justifications": [{"route": "口服", "justification": "免疫調節劑標準給藥途徑"}],
            "blood_withdrawals": [{"volume_ml": 5, "frequency": "每2週一次，共10次", "justification": "免疫指標檢測", "total_volume_per_animal": 50}],
            "imaging": [],
            "restraint": [{"method": "徒手保定", "duration": "約5分鐘", "frequency": "給藥及採血時", "justification": "操作所需"}],
            "pain": {"category": "輕微不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成6個月免疫系統評估", "humane_endpoint": "如出現嚴重免疫異常，終止實驗並進行治療"},
            "final_handling": {"method": "人道安樂死後進行免疫組織採樣", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "委由簽約之合格化製廠商進行化製處理\n(化製廠商名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法、實驗動物管理與使用指引、GLP規範", "references": ["動物保護法", "實驗動物管理與使用指引", "Good Laboratory Practice (GLP)"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 30, "age_min": 8, "age_max": 12, "age_unlimited": false, "weight_min": 25, "weight_max": 40, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 30},
        "personnel": [{"id": 1, "name": "免疫研究員1", "position": "資深研究員", "roles": ["a", "b", "c"], "roles_other_text": "", "years_experience": 7, "trainings": ["A", "B"], "training_certificates": [{"training_code": "A", "certificate_no": "IMMUNE-2024-001"}, {"training_code": "B", "certificate_no": "IMMUNE-VET-2024-001"}]}, {"id": 2, "name": "免疫獸醫師", "position": "獸醫師", "roles": ["h"], "roles_other_text": "", "years_experience": 9, "trainings": ["A", "B"], "training_certificates": [{"training_code": "A", "certificate_no": "IMMUNE-VET-2024-002"}, {"training_code": "B", "certificate_no": "IMMUNE-VET-2024-003"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻免疫系統研究', 'DRAFT', v_user_id, v_protocol_data, '2026-04-01'::DATE, '2026-09-30'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議8: 骨骼發育研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻骨骼發育研究",
            "apply_study_number": "",
            "start_date": "2026-05-01",
            "end_date": "2026-10-31",
            "project_type": "研究",
            "project_category": "骨骼科學",
            "test_item_type": "礦物質補充",
            "tech_categories": ["骨骼評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "骨骼測試PI", "phone": "0989-012-345", "email": "bone.pi@test.com", "address": "台東縣台東市測試街800號"},
            "sponsor": {"name": "骨骼研究機構", "contact_person": "骨骼聯絡人", "contact_phone": "089-8888-9999", "contact_email": "bone@test.com"},
            "sd": {"name": "骨骼專案主持人", "email": "bone.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估礦物質補充對豬隻骨骼發育的影響",
            "replacement": {"rationale": "骨骼發育需完整骨骼系統", "alt_search": {"platforms": ["PubMed"], "keywords": "porcine bone development", "conclusion": "需活體骨骼系統"}},
            "reduction": {"design": "對照組與試驗組，每組13頭", "grouping_plan": [{"group_name": "對照組", "n": 13, "treatment": "標準飼料", "timepoints": "定期檢測"}, {"group_name": "試驗組", "n": 13, "treatment": "礦物質補充", "timepoints": "定期檢測"}]},
            "duplicate": {"experiment": false, "justification": "骨骼發育研究"}
        },
        "items": {"use_test_item": true, "test_items": [{"name": "礦物質補充劑L", "source": "測試供應商", "purity": "85%", "storage": "常溫保存"}], "control_items": [{"name": "標準飼料", "source": "標準配方", "purity": "N/A", "storage": "常溫保存"}]},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "每日餵食、定期X光檢查、測量骨骼密度、評估骨骼發育",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 3, "frequency": "每月一次", "justification": "礦物質代謝檢測", "total_volume_per_animal": 18}],
            "imaging": [{"type": "X光", "frequency": "每2個月一次", "justification": "骨骼發育監測"}],
            "restraint": [{"method": "徒手保定", "duration": "約5分鐘", "frequency": "X光檢查及採血時", "justification": "檢查所需"}],
            "pain": {"category": "無痛或輕微不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成6個月骨骼發育評估", "humane_endpoint": "如出現嚴重骨骼問題，終止實驗"},
            "final_handling": {"method": "人道安樂死後進行骨骼組織採樣", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "委由簽約之合格化製廠商進行化製處理\n(化製廠商名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法", "實驗動物管理與使用指引"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 26, "age_min": 4, "age_max": 8, "age_unlimited": false, "weight_min": 15, "weight_max": 25, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 26},
        "personnel": [{"id": 1, "name": "骨骼研究員1", "position": "研究員", "roles": ["b", "c"], "roles_other_text": "", "years_experience": 5, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "BONE-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻骨骼發育研究', 'DRAFT', v_user_id, v_protocol_data, '2026-05-01'::DATE, '2026-10-31'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議9: 心血管健康研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻心血管健康研究",
            "apply_study_number": "",
            "start_date": "2026-06-01",
            "end_date": "2026-11-30",
            "project_type": "研究",
            "project_category": "心血管科學",
            "test_item_type": "心血管保健",
            "tech_categories": ["心血管評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "心血管測試PI", "phone": "0910-123-456", "email": "cardio.pi@test.com", "address": "基隆市信義區測試路900號"},
            "sponsor": {"name": "心血管研究機構", "contact_person": "心血管聯絡人", "contact_phone": "02-9999-0000", "contact_email": "cardio@test.com"},
            "sd": {"name": "心血管專案主持人", "email": "cardio.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估不同飼養方式對豬隻心血管健康的影響",
            "replacement": {"rationale": "心血管健康需完整循環系統", "alt_search": {"platforms": ["Google Scholar"], "keywords": "porcine cardiovascular health", "conclusion": "需活體循環系統"}},
            "reduction": {"design": "對照組與試驗組，每組11頭", "grouping_plan": [{"group_name": "對照組", "n": 11, "treatment": "標準飼養", "timepoints": "定期檢測"}, {"group_name": "試驗組", "n": 11, "treatment": "心血管保健", "timepoints": "定期檢測"}]},
            "duplicate": {"experiment": false, "justification": "心血管健康研究"}
        },
        "items": {"use_test_item": true, "test_items": [{"name": "心血管保健劑M", "source": "測試供應商", "purity": "93%", "storage": "常溫保存"}], "control_items": [{"name": "標準飼料", "source": "標準配方", "purity": "N/A", "storage": "常溫保存"}]},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "每日餵食、定期心電圖檢查、測量血壓、檢測心血管指標",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 4, "frequency": "每3週一次", "justification": "心血管指標檢測", "total_volume_per_animal": 24}],
            "imaging": [{"type": "心電圖", "frequency": "每2個月一次", "justification": "心血管功能監測"}],
            "restraint": [{"method": "徒手保定", "duration": "約5分鐘", "frequency": "檢查及採血時", "justification": "檢查所需"}],
            "pain": {"category": "無痛或輕微不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成6個月心血管健康評估", "humane_endpoint": "如出現嚴重心血管問題，終止實驗"},
            "final_handling": {"method": "人道安樂死後進行心血管組織採樣", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "委由簽約之合格化製廠商進行化製處理\n(化製廠商名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法", "實驗動物管理與使用指引"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 22, "age_min": 8, "age_max": 12, "age_unlimited": false, "weight_min": 25, "weight_max": 35, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 22},
        "personnel": [{"id": 1, "name": "心血管研究員1", "position": "研究員", "roles": ["b", "c"], "roles_other_text": "", "years_experience": 4, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "CARDIO-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻心血管健康研究', 'DRAFT', v_user_id, v_protocol_data, '2026-06-01'::DATE, '2026-11-30'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議10: 遺傳育種研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻遺傳育種研究",
            "apply_study_number": "",
            "start_date": "2026-07-01",
            "end_date": "2027-06-30",
            "project_type": "研究",
            "project_category": "遺傳科學",
            "test_item_type": "育種管理",
            "tech_categories": ["遺傳評估"],
            "funding_sources": ["政府補助", "公司內部預算"],
            "pi": {"name": "遺傳測試PI", "phone": "0921-234-567", "email": "genetic.pi@test.com", "address": "南投縣南投市測試路1000號"},
            "sponsor": {"name": "遺傳研究機構", "contact_person": "遺傳聯絡人", "contact_phone": "049-0000-1111", "contact_email": "genetic@test.com"},
            "sd": {"name": "遺傳專案主持人", "email": "genetic.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估不同育種策略對豬隻遺傳特性的影響",
            "replacement": {"rationale": "遺傳育種需完整繁殖系統", "alt_search": {"platforms": ["PubMed"], "keywords": "porcine genetics breeding", "conclusion": "需完整繁殖系統"}},
            "reduction": {"design": "三組不同育種策略，每組20頭", "grouping_plan": [{"group_name": "對照組", "n": 20, "treatment": "標準育種", "timepoints": "整個繁殖週期"}, {"group_name": "試驗組A", "n": 20, "treatment": "育種策略A", "timepoints": "整個繁殖週期"}, {"group_name": "試驗組B", "n": 20, "treatment": "育種策略B", "timepoints": "整個繁殖週期"}]},
            "duplicate": {"experiment": false, "justification": "長期育種研究"}
        },
        "items": {"use_test_item": false, "test_items": [], "control_items": []},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "記錄配種、懷孕、分娩、仔豬性狀、進行遺傳分析",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 5, "frequency": "每個繁殖階段一次", "justification": "遺傳標記檢測", "total_volume_per_animal": 25}],
            "imaging": [{"type": "超音波", "frequency": "懷孕確認及監測", "justification": "非侵入性監測"}],
            "restraint": [{"method": "徒手保定", "duration": "約5分鐘", "frequency": "超音波檢查及採血時", "justification": "檢查所需"}],
            "pain": {"category": "無痛或不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成一個完整繁殖週期及遺傳評估", "humane_endpoint": "如出現嚴重繁殖問題，終止實驗"},
            "final_handling": {"method": "實驗結束後正常繁殖管理", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "正常繁殖管理，無安樂死"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法", "實驗動物管理與使用指引"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "母", "number": 60, "age_min": 8, "age_max": 24, "age_unlimited": false, "weight_min": 100, "weight_max": 200, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 60},
        "personnel": [{"id": 1, "name": "遺傳研究員1", "position": "資深研究員", "roles": ["a", "b"], "roles_other_text": "", "years_experience": 8, "trainings": ["A", "B"], "training_certificates": [{"training_code": "A", "certificate_no": "GENETIC-2024-001"}, {"training_code": "B", "certificate_no": "GENETIC-VET-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻遺傳育種研究', 'DRAFT', v_user_id, v_protocol_data, '2026-07-01'::DATE, '2027-06-30'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    RAISE NOTICE 'Ten draft test protocols created successfully';
END $$;
