-- ============================================
-- 遷移 027: 創建測試協議（草稿狀態）
-- ============================================

-- 插入測試協議，使用第一個可用用戶作為 PI
DO $$
DECLARE
    v_user_id UUID;
    v_protocol_id UUID;
    v_protocol_no VARCHAR(50);
    v_roc_year INTEGER;
    v_seq INTEGER;
BEGIN
    -- 獲取第一個可用的用戶 ID
    SELECT id INTO v_user_id FROM users LIMIT 1;
    
    IF v_user_id IS NULL THEN
        RAISE EXCEPTION 'No users found in database';
    END IF;
    
    -- 生成協議編號 (格式：Pre-{民國年}-{序號:03})
    SELECT EXTRACT(YEAR FROM NOW()) - 1911 INTO v_roc_year;
    SELECT COALESCE(MAX(CAST(SUBSTRING(protocol_no FROM '[0-9]+$') AS INTEGER)), 0) + 1
    INTO v_seq
    FROM protocols
    WHERE protocol_no LIKE 'Pre-' || v_roc_year || '-%';
    
    v_protocol_no := 'Pre-' || v_roc_year || '-' || LPAD(v_seq::TEXT, 3, '0');
    v_protocol_id := gen_random_uuid();
    
    -- 插入協議（草稿狀態，無 IACUC 編號）
    INSERT INTO protocols (
        id,
        protocol_no,
        iacuc_no,
        title,
        status,
        pi_user_id,
        working_content,
        start_date,
        end_date,
        created_by,
        created_at,
        updated_at
    ) VALUES (
        v_protocol_id,
        v_protocol_no,
        NULL,
        '草稿測試協議 - 豬隻免疫反應研究',
        'DRAFT',
        v_user_id,
        '{
            "basic": {
                "is_glp": true,
                "registration_authorities": ["TFDA"],
                "study_title": "草稿測試協議 - 豬隻免疫反應研究",
                "apply_study_number": "STUDY-2025-002",
                "start_date": "2025-03-01",
                "end_date": "2025-11-30",
                "project_type": "臨床試驗",
                "project_category": "疫苗研發",
                "test_item_type": "生物製劑",
                "tech_categories": ["免疫學評估", "安全性試驗"],
                "funding_sources": ["政府補助", "公司內部預算"],
                "pi": {
                    "name": "草稿測試主持人",
                    "phone": "0923-456-789",
                    "email": "draft.pi@test.com",
                    "address": "台北市信義區測試路100號"
                },
                "sponsor": {
                    "name": "草稿委託單位有限公司",
                    "contact_person": "草稿聯絡人",
                    "contact_phone": "03-9876-5432",
                    "contact_email": "draft.sponsor@test.com"
                },
                "sd": {
                    "name": "草稿專案主持人",
                    "email": "draft.sd@test.com"
                },
                "facility": {
                    "title": "豬博士動物科技股份有限公司",
                    "address": "苗栗縣後龍鎮外埔里外埔6-15號"
                },
                "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
            },
            "purpose": {
                "significance": "本研究旨在評估新型疫苗對豬隻的免疫反應及安全性，包括抗體產生、細胞免疫反應等關鍵指標。",
                "replacement": {
                    "rationale": "疫苗效果評估必須在活體動物中進行，以觀察整體免疫系統反應及保護效果。",
                    "alt_search": {
                        "platforms": ["PubMed", "Google Scholar", "ScienceDirect"],
                        "keywords": "porcine vaccine immunogenicity safety",
                        "conclusion": "目前尚無可替代的體外方法評估疫苗完整保護效力"
                    }
                },
                "reduction": {
                    "design": "採用對照組設計，分為疫苗組、佐劑對照組及空白對照組，每組20頭豬隻，以確保統計顯著性。",
                    "grouping_plan": [
                        {
                            "group_name": "疫苗組",
                            "n": 20,
                            "treatment": "試驗疫苗",
                            "timepoints": "第0、7、14、21、28、35、42、49、56天採血"
                        },
                        {
                            "group_name": "佐劑對照組",
                            "n": 20,
                            "treatment": "生理食鹽水 + 佐劑",
                            "timepoints": "第0、7、14、21、28、35、42、49、56天採血"
                        },
                        {
                            "group_name": "空白對照組",
                            "n": 20,
                            "treatment": "生理食鹽水",
                            "timepoints": "第0、7、14、21、28、35、42、49、56天採血"
                        }
                    ]
                },
                "duplicate": {
                    "experiment": true,
                    "justification": "為驗證結果可重複性，將進行二期重複試驗"
                }
            },
            "items": {
                "use_test_item": true,
                "test_items": [
                    {
                        "name": "試驗疫苗A",
                        "source": "內部研發",
                        "purity": "符合GMP標準",
                        "storage": "2-8°C冷藏保存"
                    },
                    {
                        "name": "佐劑B",
                        "source": "商業採購",
                        "purity": "分析純",
                        "storage": "室溫保存"
                    }
                ],
                "control_items": [
                    {
                        "name": "生理食鹽水",
                        "source": "商業採購",
                        "purity": "注射級",
                        "storage": "室溫保存"
                    }
                ]
            },
            "design": {
                "anesthesia": {
                    "is_under_anesthesia": false,
                    "plan_type": "",
                    "premed_option": ""
                },
                "procedures": "1. 疫苗接種：肌肉注射，每2週一次，共接種3次\n2. 血液採集：接種前及接種後定期採集血液樣本\n3. 臨床觀察：每日觀察動物行為、食慾、體溫等指標\n4. 體重測量：每週測量體重\n5. 免疫指標檢測：ELISA檢測抗體濃度、流式細胞儀檢測T細胞反應",
                "route_justifications": [
                    {
                        "route": "肌肉注射",
                        "justification": "疫苗標準給藥途徑，確保抗原有效遞呈至免疫系統"
                    }
                ],
                "blood_withdrawals": [
                    {
                        "volume_ml": 5,
                        "frequency": "每週一次，共8次",
                        "justification": "免疫指標檢測所需樣本量",
                        "total_volume_per_animal": 40
                    }
                ],
                "imaging": [],
                "restraint": [
                    {
                        "method": "徒手保定",
                        "duration": "約5分鐘",
                        "frequency": "採血及疫苗接種時",
                        "justification": "進行注射及採血所需"
                    }
                ],
                "pain": {
                    "category": "輕微不適"
                },
                "restrictions": {
                    "is_restricted": false,
                    "types": []
                },
                "endpoints": {
                    "experimental_endpoint": "完成8週免疫反應評估或達到預定採樣次數",
                    "humane_endpoint": "如動物出現嚴重過敏反應、持續發燒、食慾不振超過48小時、體重下降超過15%等情況，經獸醫師評估後提早結束實驗。"
                },
                "final_handling": {
                    "method": "人道安樂死後進行組織採樣",
                    "transfer": {
                        "recipient_name": "",
                        "recipient_org": "",
                        "project_name": ""
                    }
                },
                "carcass_disposal": {
                    "method": "委由簽約之合格化製廠商進行化製處理\n(化製廠商名稱：金海龍生物科技股份有限公司，化製廠管編：P6001213)"
                },
                "non_pharma_grade": {
                    "used": false,
                    "description": ""
                },
                "hazards": {
                    "used": false,
                    "selected_type": null,
                    "materials": [],
                    "waste_disposal_method": "",
                    "operation_location_method": "",
                    "protection_measures": "",
                    "waste_and_carcass_disposal": ""
                },
                "controlled_substances": {
                    "used": false,
                    "items": []
                }
            },
            "guidelines": {
                "content": "遵循動物保護法、實驗動物管理與使用指引、GLP規範",
                "references": [
                    "動物保護法",
                    "實驗動物管理與使用指引",
                    "Good Laboratory Practice (GLP)",
                    "VICH GL9: Good Clinical Practice"
                ]
            },
            "surgery": {
                "surgery_type": "",
                "preop_preparation": "",
                "aseptic_techniques": [],
                "surgery_description": "",
                "surgery_steps": [],
                "monitoring": "",
                "postop_expected_impact": "",
                "multiple_surgeries": {
                    "used": false,
                    "number": 0,
                    "reason": ""
                },
                "postop_care_type": null,
                "postop_care": "",
                "drugs": [],
                "expected_end_point": ""
            },
            "animals": {
                "animals": [
                    {
                        "species": "豬",
                        "species_other": "",
                        "strain": "LYD",
                        "strain_other": "",
                        "sex": "公母各半",
                        "number": 60,
                        "age_min": 10,
                        "age_max": 16,
                        "age_unlimited": false,
                        "weight_min": 30,
                        "weight_max": 50,
                        "weight_unlimited": false,
                        "housing_location": "豬博士畜牧場"
                    }
                ],
                "total_animals": 60
            },
            "personnel": [
                {
                    "id": 1,
                    "name": "草稿研究員A",
                    "position": "資深研究員",
                    "roles": ["a", "b", "c"],
                    "roles_other_text": "",
                    "years_experience": 5,
                    "trainings": ["A", "C"],
                    "training_certificates": [
                        {
                            "training_code": "A",
                            "certificate_no": "DRAFT-2024-001"
                        },
                        {
                            "training_code": "C",
                            "certificate_no": "DRAFT-RAD-2024-001"
                        }
                    ]
                },
                {
                    "id": 2,
                    "name": "草稿研究員B",
                    "position": "研究助理",
                    "roles": ["d", "f", "g"],
                    "roles_other_text": "",
                    "years_experience": 2,
                    "trainings": ["A"],
                    "training_certificates": [
                        {
                            "training_code": "A",
                            "certificate_no": "DRAFT-2024-002"
                        }
                    ]
                },
                {
                    "id": 3,
                    "name": "草稿獸醫師",
                    "position": "獸醫師",
                    "roles": ["h"],
                    "roles_other_text": "",
                    "years_experience": 8,
                    "trainings": ["A", "B"],
                    "training_certificates": [
                        {
                            "training_code": "A",
                            "certificate_no": "DRAFT-VET-2024-001"
                        },
                        {
                            "training_code": "B",
                            "certificate_no": "DRAFT-VET-2024-002"
                        }
                    ]
                }
            ],
            "attachments": [],
            "signature": []
        }'::jsonb,
        '2025-03-01'::DATE,
        '2025-11-30'::DATE,
        v_user_id,
        NOW(),
        NOW()
    );
    
    -- 記錄狀態歷程
    INSERT INTO protocol_status_history (
        id,
        protocol_id,
        from_status,
        to_status,
        changed_by,
        remark,
        created_at
    ) VALUES (
        gen_random_uuid(),
        v_protocol_id,
        NULL,
        'DRAFT',
        v_user_id,
        '草稿測試協議建立',
        NOW()
    );
    
    -- 關聯 PI 使用者
    INSERT INTO user_protocols (
        user_id,
        protocol_id,
        role_in_protocol,
        granted_at,
        granted_by
    ) VALUES (
        v_user_id,
        v_protocol_id,
        'PI',
        NOW(),
        v_user_id
    )
    ON CONFLICT (user_id, protocol_id) DO NOTHING;
    
    RAISE NOTICE 'Draft test protocol created: % (status: DRAFT, no IACUC number)', v_protocol_no;
END $$;