-- ============================================
-- 遷移 029: 創建5個測試協議（草稿狀態）
-- ============================================

DO $$
DECLARE
    v_user_id UUID;
    v_protocol_id UUID;
    v_protocol_no VARCHAR(50);
    v_roc_year INTEGER;
    v_seq INTEGER;
    v_protocol_data JSONB;
    i INTEGER;
BEGIN
    -- 獲取第一個可用的用戶 ID
    SELECT id INTO v_user_id FROM users LIMIT 1;
    
    IF v_user_id IS NULL THEN
        RAISE EXCEPTION 'No users found in database';
    END IF;
    
    -- 生成協議編號的基礎資訊
    SELECT EXTRACT(YEAR FROM NOW()) - 1911 INTO v_roc_year;
    
    -- 協議1: 飼料安全性評估
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 飼料安全性評估研究",
            "apply_study_number": "",
            "start_date": "2025-05-01",
            "end_date": "2025-09-30",
            "project_type": "研究",
            "project_category": "安全性評估",
            "test_item_type": "飼料原料",
            "tech_categories": ["毒性評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "安全測試PI", "phone": "0945-678-901", "email": "safety.pi@test.com", "address": "台中市西區測試路300號"},
            "sponsor": {"name": "安全評估機構", "contact_person": "安全聯絡人", "contact_phone": "05-2233-4455", "contact_email": "safety@test.com"},
            "sd": {"name": "安全專案主持人", "email": "safety.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估新飼料原料的安全性，包括急性和亞慢性毒性評估。",
            "replacement": {"rationale": "安全性評估需要完整的動物代謝系統", "alt_search": {"platforms": ["PubMed"], "keywords": "feed safety toxicity", "conclusion": "需活體動物試驗"}},
            "reduction": {"design": "對照組與試驗組設計，每組10頭", "grouping_plan": [{"group_name": "對照組", "n": 10, "treatment": "標準飼料", "timepoints": "每日觀察"}, {"group_name": "試驗組", "n": 10, "treatment": "試驗飼料", "timepoints": "每日觀察"}]},
            "duplicate": {"experiment": false, "justification": "首次試驗"}
        },
        "items": {"use_test_item": true, "test_items": [{"name": "新飼料原料D", "source": "測試供應商", "purity": "98%", "storage": "常溫保存"}], "control_items": [{"name": "標準飼料", "source": "標準配方", "purity": "N/A", "storage": "常溫保存"}]},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "每日餵食、觀察健康狀況、每週測量體重",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 2, "frequency": "每2週一次", "justification": "生化指標檢測", "total_volume_per_animal": 12}],
            "imaging": [],
            "restraint": [{"method": "徒手保定", "duration": "約2分鐘", "frequency": "採血時", "justification": "採血所需"}],
            "pain": {"category": "無痛或輕微不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成12週安全性評估", "humane_endpoint": "如出現嚴重中毒症狀，立即終止實驗"},
            "final_handling": {"method": "人道安樂死", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "委由簽約之合格化製廠商進行化製處理\n(化製廠商名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法", "實驗動物管理與使用指引"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 20, "age_min": 8, "age_max": 12, "age_unlimited": false, "weight_min": 25, "weight_max": 35, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 20},
        "personnel": [{"id": 1, "name": "安全研究員1", "position": "研究員", "roles": ["b", "c"], "roles_other_text": "", "years_experience": 3, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "SAFE-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 飼料安全性評估研究', 'DRAFT', v_user_id, v_protocol_data, '2025-05-01'::DATE, '2025-09-30'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議2: 行為觀察研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻行為觀察研究",
            "apply_study_number": "",
            "start_date": "2025-06-01",
            "end_date": "2025-08-31",
            "project_type": "研究",
            "project_category": "行為科學",
            "test_item_type": "環境豐富化",
            "tech_categories": ["行為評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "行為測試PI", "phone": "0956-789-012", "email": "behavior.pi@test.com", "address": "高雄市前金區測試巷400號"},
            "sponsor": {"name": "行為研究機構", "contact_person": "行為聯絡人", "contact_phone": "07-3344-5566", "contact_email": "behavior@test.com"},
            "sd": {"name": "行為專案主持人", "email": "behavior.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "觀察不同環境豐富化條件對豬隻行為的影響",
            "replacement": {"rationale": "行為觀察需在自然環境中進行", "alt_search": {"platforms": ["Google Scholar"], "keywords": "porcine behavior enrichment", "conclusion": "需活體觀察"}},
            "reduction": {"design": "兩組對照，每組8頭", "grouping_plan": [{"group_name": "對照組", "n": 8, "treatment": "標準環境", "timepoints": "每日觀察2小時"}, {"group_name": "試驗組", "n": 8, "treatment": "環境豐富化", "timepoints": "每日觀察2小時"}]},
            "duplicate": {"experiment": false, "justification": "探索性研究"}
        },
        "items": {"use_test_item": false, "test_items": [], "control_items": []},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "每日行為觀察、錄影記錄、分析行為模式",
            "route_justifications": [],
            "blood_withdrawals": [],
            "imaging": [],
            "restraint": [],
            "pain": {"category": "無痛或不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成3個月行為觀察", "humane_endpoint": "如出現異常行為或健康問題，終止實驗"},
            "final_handling": {"method": "實驗結束後正常飼養", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "實驗結束後正常飼養，無安樂死"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 16, "age_min": 4, "age_max": 8, "age_unlimited": false, "weight_min": 15, "weight_max": 25, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 16},
        "personnel": [{"id": 1, "name": "行為研究員1", "position": "研究助理", "roles": ["b"], "roles_other_text": "", "years_experience": 2, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "BEHAV-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻行為觀察研究', 'DRAFT', v_user_id, v_protocol_data, '2025-06-01'::DATE, '2025-08-31'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議3: 繁殖性能研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻繁殖性能研究",
            "apply_study_number": "",
            "start_date": "2025-07-01",
            "end_date": "2026-06-30",
            "project_type": "研究",
            "project_category": "繁殖科學",
            "test_item_type": "繁殖管理",
            "tech_categories": ["繁殖評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "繁殖測試PI", "phone": "0967-890-123", "email": "breeding.pi@test.com", "address": "台南市東區測試路500號"},
            "sponsor": {"name": "繁殖研究機構", "contact_person": "繁殖聯絡人", "contact_phone": "06-4455-6677", "contact_email": "breeding@test.com"},
            "sd": {"name": "繁殖專案主持人", "email": "breeding.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估不同飼養管理方式對母豬繁殖性能的影響",
            "replacement": {"rationale": "繁殖性能評估需完整生殖週期", "alt_search": {"platforms": ["PubMed"], "keywords": "porcine reproduction performance", "conclusion": "需完整繁殖週期觀察"}},
            "reduction": {"design": "對照組與試驗組，每組15頭母豬", "grouping_plan": [{"group_name": "對照組", "n": 15, "treatment": "標準管理", "timepoints": "整個繁殖週期"}, {"group_name": "試驗組", "n": 15, "treatment": "改進管理", "timepoints": "整個繁殖週期"}]},
            "duplicate": {"experiment": false, "justification": "長期觀察研究"}
        },
        "items": {"use_test_item": false, "test_items": [], "control_items": []},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "記錄發情週期、配種、懷孕、分娩、仔豬數量及存活率",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 5, "frequency": "每個繁殖階段一次", "justification": "荷爾蒙檢測", "total_volume_per_animal": 20}],
            "imaging": [{"type": "超音波", "frequency": "懷孕確認及監測", "justification": "非侵入性監測"}],
            "restraint": [{"method": "徒手保定", "duration": "約5分鐘", "frequency": "超音波檢查時", "justification": "檢查所需"}],
            "pain": {"category": "無痛或不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成一個完整繁殖週期評估", "humane_endpoint": "如出現嚴重繁殖問題，終止實驗"},
            "final_handling": {"method": "實驗結束後正常繁殖管理", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "正常繁殖管理，無安樂死"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法", "實驗動物管理與使用指引"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "母", "number": 30, "age_min": 8, "age_max": 24, "age_unlimited": false, "weight_min": 100, "weight_max": 200, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 30},
        "personnel": [{"id": 1, "name": "繁殖研究員1", "position": "資深研究員", "roles": ["a", "b"], "roles_other_text": "", "years_experience": 6, "trainings": ["A", "B"], "training_certificates": [{"training_code": "A", "certificate_no": "BREED-2024-001"}, {"training_code": "B", "certificate_no": "BREED-VET-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻繁殖性能研究', 'DRAFT', v_user_id, v_protocol_data, '2025-07-01'::DATE, '2026-06-30'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議4: 環境適應性研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": false,
            "registration_authorities": [],
            "study_title": "草稿測試協議 - 豬隻環境適應性研究",
            "apply_study_number": "",
            "start_date": "2025-08-01",
            "end_date": "2025-11-30",
            "project_type": "研究",
            "project_category": "環境科學",
            "test_item_type": "環境控制",
            "tech_categories": ["適應性評估"],
            "funding_sources": ["公司內部預算"],
            "pi": {"name": "環境測試PI", "phone": "0978-901-234", "email": "environment.pi@test.com", "address": "桃園市中壢區測試街600號"},
            "sponsor": {"name": "環境研究機構", "contact_person": "環境聯絡人", "contact_phone": "03-5566-7788", "contact_email": "environment@test.com"},
            "sd": {"name": "環境專案主持人", "email": "environment.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估不同環境條件（溫度、濕度）對豬隻適應性的影響",
            "replacement": {"rationale": "環境適應性需在實際環境中觀察", "alt_search": {"platforms": ["Google Scholar"], "keywords": "porcine environmental adaptation", "conclusion": "需實際環境測試"}},
            "reduction": {"design": "三組不同環境條件，每組12頭", "grouping_plan": [{"group_name": "標準環境組", "n": 12, "treatment": "標準溫濕度", "timepoints": "每日監測"}, {"group_name": "高溫組", "n": 12, "treatment": "高溫環境", "timepoints": "每日監測"}, {"group_name": "低溫組", "n": 12, "treatment": "低溫環境", "timepoints": "每日監測"}]},
            "duplicate": {"experiment": false, "justification": "環境適應性研究"}
        },
        "items": {"use_test_item": false, "test_items": [], "control_items": []},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "控制環境條件、每日監測體溫、行為觀察、記錄生理指標",
            "route_justifications": [],
            "blood_withdrawals": [{"volume_ml": 3, "frequency": "每週一次", "justification": "生理指標檢測", "total_volume_per_animal": 12}],
            "imaging": [],
            "restraint": [{"method": "徒手保定", "duration": "約3分鐘", "frequency": "測量體溫及採血時", "justification": "操作所需"}],
            "pain": {"category": "無痛或輕微不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成4個月環境適應評估", "humane_endpoint": "如出現嚴重不適或健康問題，終止實驗"},
            "final_handling": {"method": "實驗結束後恢復標準環境", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "實驗結束後正常飼養，無安樂死"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法及實驗動物使用指引", "references": ["動物保護法"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 36, "age_min": 6, "age_max": 10, "age_unlimited": false, "weight_min": 20, "weight_max": 30, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 36},
        "personnel": [{"id": 1, "name": "環境研究員1", "position": "研究員", "roles": ["b", "c"], "roles_other_text": "", "years_experience": 4, "trainings": ["A"], "training_certificates": [{"training_code": "A", "certificate_no": "ENV-2024-001"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻環境適應性研究', 'DRAFT', v_user_id, v_protocol_data, '2025-08-01'::DATE, '2025-11-30'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    -- 協議5: 疾病抵抗力研究
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq FROM protocols WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    v_protocol_data := '{
        "basic": {
            "is_glp": true,
            "registration_authorities": ["TFDA"],
            "study_title": "草稿測試協議 - 豬隻疾病抵抗力研究",
            "apply_study_number": "STUDY-2025-003",
            "start_date": "2025-09-01",
            "end_date": "2026-02-28",
            "project_type": "臨床試驗",
            "project_category": "疾病預防",
            "test_item_type": "免疫增強劑",
            "tech_categories": ["免疫學評估", "疾病挑戰"],
            "funding_sources": ["政府補助", "公司內部預算"],
            "pi": {"name": "疾病測試PI", "phone": "0989-012-345", "email": "disease.pi@test.com", "address": "新北市板橋區測試路700號"},
            "sponsor": {"name": "疾病研究機構", "contact_person": "疾病聯絡人", "contact_phone": "02-6677-8899", "contact_email": "disease@test.com"},
            "sd": {"name": "疾病專案主持人", "email": "disease.sd@test.com"},
            "facility": {"title": "豬博士動物科技股份有限公司", "address": "苗栗縣後龍鎮外埔里外埔6-15號"},
            "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
        },
        "purpose": {
            "significance": "評估免疫增強劑對豬隻疾病抵抗力的影響，包括疫苗效果及自然感染抵抗力",
            "replacement": {"rationale": "疾病抵抗力評估需完整免疫系統", "alt_search": {"platforms": ["PubMed", "ScienceDirect"], "keywords": "porcine disease resistance immunity", "conclusion": "需活體免疫系統評估"}},
            "reduction": {"design": "對照組與試驗組，每組18頭", "grouping_plan": [{"group_name": "對照組", "n": 18, "treatment": "標準疫苗", "timepoints": "接種後定期檢測"}, {"group_name": "試驗組", "n": 18, "treatment": "標準疫苗 + 免疫增強劑", "timepoints": "接種後定期檢測"}]},
            "duplicate": {"experiment": true, "justification": "為驗證結果，進行重複試驗"}
        },
        "items": {"use_test_item": true, "test_items": [{"name": "免疫增強劑E", "source": "內部研發", "purity": "符合藥典標準", "storage": "2-8°C冷藏保存"}], "control_items": [{"name": "標準疫苗", "source": "商業採購", "purity": "符合藥典標準", "storage": "2-8°C冷藏保存"}]},
        "design": {
            "anesthesia": {"is_under_anesthesia": false, "plan_type": "", "premed_option": ""},
            "procedures": "疫苗接種、免疫增強劑給藥、定期採血檢測抗體、疾病挑戰試驗",
            "route_justifications": [{"route": "肌肉注射", "justification": "疫苗及免疫增強劑標準給藥途徑"}],
            "blood_withdrawals": [{"volume_ml": 5, "frequency": "每2週一次，共8次", "justification": "免疫指標檢測", "total_volume_per_animal": 40}],
            "imaging": [],
            "restraint": [{"method": "徒手保定", "duration": "約5分鐘", "frequency": "接種及採血時", "justification": "操作所需"}],
            "pain": {"category": "輕微不適"},
            "restrictions": {"is_restricted": false, "types": []},
            "endpoints": {"experimental_endpoint": "完成16週免疫及疾病抵抗力評估", "humane_endpoint": "如出現嚴重疾病症狀或無法恢復，終止實驗並進行治療"},
            "final_handling": {"method": "人道安樂死後進行組織採樣", "transfer": {"recipient_name": "", "recipient_org": "", "project_name": ""}},
            "carcass_disposal": {"method": "委由簽約之合格化製廠商進行化製處理\n(化製廠商名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)"},
            "non_pharma_grade": {"used": false, "description": ""},
            "hazards": {"used": false, "selected_type": null, "materials": [], "waste_disposal_method": "", "operation_location_method": "", "protection_measures": "", "waste_and_carcass_disposal": ""},
            "controlled_substances": {"used": false, "items": []}
        },
        "guidelines": {"content": "遵循動物保護法、實驗動物管理與使用指引、GLP規範", "references": ["動物保護法", "實驗動物管理與使用指引", "Good Laboratory Practice (GLP)"]},
        "surgery": {"surgery_type": "", "preop_preparation": "", "aseptic_techniques": [], "surgery_description": "", "surgery_steps": [], "monitoring": "", "postop_expected_impact": "", "multiple_surgeries": {"used": false, "number": 0, "reason": ""}, "postop_care_type": null, "postop_care": "", "drugs": [], "expected_end_point": ""},
        "animals": {"animals": [{"species": "豬", "species_other": "", "strain": "LYD", "strain_other": "", "sex": "公母各半", "number": 36, "age_min": 8, "age_max": 12, "age_unlimited": false, "weight_min": 25, "weight_max": 40, "weight_unlimited": false, "housing_location": "豬博士畜牧場"}], "total_animals": 36},
        "personnel": [{"id": 1, "name": "疾病研究員1", "position": "資深研究員", "roles": ["a", "b", "c"], "roles_other_text": "", "years_experience": 7, "trainings": ["A", "B"], "training_certificates": [{"training_code": "A", "certificate_no": "DISEASE-2024-001"}, {"training_code": "B", "certificate_no": "DISEASE-VET-2024-001"}]}, {"id": 2, "name": "疾病獸醫師", "position": "獸醫師", "roles": ["h"], "roles_other_text": "", "years_experience": 10, "trainings": ["A", "B"], "training_certificates": [{"training_code": "A", "certificate_no": "DISEASE-VET-2024-002"}, {"training_code": "B", "certificate_no": "DISEASE-VET-2024-003"}]}],
        "attachments": [],
        "signature": []
    }'::jsonb;
    
    INSERT INTO protocols (id, protocol_no, iacuc_no, title, status, pi_user_id, working_content, start_date, end_date, created_by, created_at, updated_at)
    VALUES (v_protocol_id, v_protocol_no, NULL, '草稿測試協議 - 豬隻疾病抵抗力研究', 'DRAFT', v_user_id, v_protocol_data, '2025-09-01'::DATE, '2026-02-28'::DATE, v_user_id, NOW(), NOW());
    INSERT INTO protocol_status_history (id, protocol_id, from_status, to_status, changed_by, remark, created_at)
    VALUES (gen_random_uuid(), v_protocol_id, NULL, 'DRAFT', v_user_id, '草稿測試協議建立', NOW());
    INSERT INTO user_protocols (user_id, protocol_id, role_in_protocol, granted_at, granted_by)
    VALUES (v_user_id, v_protocol_id, 'PI', NOW(), v_user_id) ON CONFLICT DO NOTHING;
    
    RAISE NOTICE 'Five draft test protocols created successfully';
END $$;