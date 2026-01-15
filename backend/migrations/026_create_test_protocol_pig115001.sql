-- ============================================
-- 遷移 026: 創建測試協議 PIG-115001
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
    
    -- 插入協議
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
        'PIG-115001',
        '測試用計畫書 - 豬隻生長性能評估研究',
        'DRAFT',
        v_user_id,
        '{
            "basic": {
                "is_glp": false,
                "registration_authorities": [],
                "study_title": "測試用計畫書 - 豬隻生長性能評估研究",
                "apply_study_number": "STUDY-2025-001",
                "start_date": "2025-02-01",
                "end_date": "2025-12-31",
                "project_type": "研究",
                "project_category": "其他",
                "test_item_type": "飼料添加物",
                "tech_categories": ["生長性能評估"],
                "funding_sources": ["公司內部預算"],
                "pi": {
                    "name": "測試主持人",
                    "phone": "0912-345-678",
                    "email": "pi@test.com",
                    "address": "測試地址"
                },
                "sponsor": {
                    "name": "測試委託單位",
                    "contact_person": "測試聯絡人",
                    "contact_phone": "02-1234-5678",
                    "contact_email": "sponsor@test.com"
                },
                "sd": {
                    "name": "測試專案主持人",
                    "email": "sd@test.com"
                },
                "facility": {
                    "title": "豬博士動物科技股份有限公司",
                    "address": "苗栗縣後龍鎮外埔里外埔6-15號"
                },
                "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
            },
            "purpose": {
                "significance": "本研究的目的是評估新的飼料添加物對豬隻生長性能的影響，包括體重增長、飼料轉換率等指標。",
                "replacement": {
                    "rationale": "本實驗無法以非動物模型取代，因為需要評估整體生長性能及代謝反應。",
                    "alt_search": {
                        "platforms": ["PubMed", "Google Scholar"],
                        "keywords": "pig growth performance feed additive",
                        "conclusion": "未發現可替代的體外研究方法"
                    }
                },
                "reduction": {
                    "design": "採用完全隨機設計，分為對照組和試驗組，每組至少15頭豬隻，以確保統計效力。",
                    "grouping_plan": [
                        {
                            "group_name": "對照組",
                            "n": 15,
                            "treatment": "基礎飼料",
                            "timepoints": "每週測量一次"
                        },
                        {
                            "group_name": "試驗組",
                            "n": 15,
                            "treatment": "基礎飼料 + 添加物",
                            "timepoints": "每週測量一次"
                        }
                    ]
                },
                "duplicate": {
                    "experiment": false,
                    "justification": "本實驗為首次進行，無重複實驗情況"
                }
            },
            "items": {
                "use_test_item": true,
                "test_items": [
                    {
                        "name": "測試用飼料添加物A",
                        "source": "測試供應商",
                        "purity": "99.5%",
                        "storage": "常溫保存"
                    }
                ],
                "control_items": [
                    {
                        "name": "基礎飼料",
                        "source": "標準配方",
                        "purity": "N/A",
                        "storage": "常溫保存"
                    }
                ]
            },
            "design": {
                "anesthesia": {
                    "is_under_anesthesia": false,
                    "plan_type": "",
                    "premed_option": ""
                },
                "procedures": "1. 每日固定時間餵食，記錄採食量\n2. 每週固定時間測量體重\n3. 觀察動物健康狀況\n4. 記錄異常行為",
                "route_justifications": [],
                "blood_withdrawals": [],
                "imaging": [],
                "restraint": [],
                "pain": {
                    "category": "無痛或輕微不適"
                },
                "restrictions": {
                    "is_restricted": false,
                    "types": []
                },
                "endpoints": {
                    "experimental_endpoint": "實驗期滿或達到目標體重",
                    "humane_endpoint": "實驗過程中如果動物體重下降超過原體重的20%、食慾不振、身體虛弱、感染等情形，經獸醫師評估後提早結束實驗。"
                },
                "final_handling": {
                    "method": "人道安樂死後進行樣本採集",
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
                "content": "遵循動物保護法及實驗動物使用規範",
                "references": [
                    "動物保護法",
                    "實驗動物管理與使用指引"
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
                        "number": 30,
                        "age_min": 8,
                        "age_max": 12,
                        "age_unlimited": false,
                        "weight_min": 25,
                        "weight_max": 35,
                        "weight_unlimited": false,
                        "housing_location": "豬博士畜牧場"
                    }
                ],
                "total_animals": 30
            },
            "personnel": [
                {
                    "id": 1,
                    "name": "測試研究員A",
                    "position": "研究員",
                    "roles": ["b", "c"],
                    "roles_other_text": "",
                    "years_experience": 3,
                    "trainings": ["A"],
                    "training_certificates": [
                        {
                            "training_code": "A",
                            "certificate_no": "TEST-2024-001"
                        }
                    ]
                },
                {
                    "id": 2,
                    "name": "測試研究員B",
                    "position": "助理研究員",
                    "roles": ["d", "f"],
                    "roles_other_text": "",
                    "years_experience": 2,
                    "trainings": ["A"],
                    "training_certificates": [
                        {
                            "training_code": "A",
                            "certificate_no": "TEST-2024-002"
                        }
                    ]
                }
            ],
            "attachments": [],
            "signature": []
        }'::jsonb,
        '2025-02-01'::DATE,
        '2025-12-31'::DATE,
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
        '測試協議建立',
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
    
    RAISE NOTICE 'Test protocol created: % with IACUC No. PIG-115001', v_protocol_no;
END $$;