-- ============================================
-- 遷移 028: 創建第二個測試協議（草稿狀態）
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
        '草稿測試協議 - 豬隻營養代謝研究',
        'DRAFT',
        v_user_id,
        '{
            "basic": {
                "is_glp": false,
                "registration_authorities": [],
                "study_title": "草稿測試協議 - 豬隻營養代謝研究",
                "apply_study_number": "",
                "start_date": "2025-04-01",
                "end_date": "2025-10-31",
                "project_type": "研究",
                "project_category": "營養研究",
                "test_item_type": "營養補充品",
                "tech_categories": ["代謝評估", "營養分析"],
                "funding_sources": ["公司內部預算"],
                "pi": {
                    "name": "營養測試主持人",
                    "phone": "0934-567-890",
                    "email": "nutrition.pi@test.com",
                    "address": "新竹市東區測試街200號"
                },
                "sponsor": {
                    "name": "營養研究委託機構",
                    "contact_person": "營養聯絡人",
                    "contact_phone": "04-1122-3344",
                    "contact_email": "nutrition.sponsor@test.com"
                },
                "sd": {
                    "name": "營養專案主持人",
                    "email": "nutrition.sd@test.com"
                },
                "facility": {
                    "title": "豬博士動物科技股份有限公司",
                    "address": "苗栗縣後龍鎮外埔里外埔6-15號"
                },
                "housing_location": "苗栗縣後龍鎮外埔里外埔6-15號"
            },
            "purpose": {
                "significance": "本研究旨在評估新型營養補充品對豬隻生長代謝的影響，包括營養吸收率、代謝指標變化等。",
                "replacement": {
                    "rationale": "營養代謝研究需要完整的消化系統和代謝途徑，無法以體外模型完全取代。",
                    "alt_search": {
                        "platforms": ["PubMed", "Web of Science"],
                        "keywords": "porcine nutrition metabolism feed efficiency",
                        "conclusion": "體外模型無法模擬完整的消化代謝過程"
                    }
                },
                "reduction": {
                    "design": "採用完全隨機設計，分為高劑量組、低劑量組及對照組，每組12頭豬隻，足以進行統計分析。",
                    "grouping_plan": [
                        {
                            "group_name": "高劑量組",
                            "n": 12,
                            "treatment": "基礎飼料 + 高劑量營養補充品",
                            "timepoints": "每週測量體重、採集糞便樣本"
                        },
                        {
                            "group_name": "低劑量組",
                            "n": 12,
                            "treatment": "基礎飼料 + 低劑量營養補充品",
                            "timepoints": "每週測量體重、採集糞便樣本"
                        },
                        {
                            "group_name": "對照組",
                            "n": 12,
                            "treatment": "基礎飼料",
                            "timepoints": "每週測量體重、採集糞便樣本"
                        }
                    ]
                },
                "duplicate": {
                    "experiment": false,
                    "justification": "本實驗為探索性研究，待結果確認後再規劃重複試驗"
                }
            },
            "items": {
                "use_test_item": true,
                "test_items": [
                    {
                        "name": "營養補充品C",
                        "source": "測試供應商",
                        "purity": "95%",
                        "storage": "乾燥陰涼處保存"
                    }
                ],
                "control_items": [
                    {
                        "name": "標準基礎飼料",
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
                "procedures": "1. 每日固定時間餵食，記錄採食量\n2. 每週固定時間測量體重\n3. 每週採集糞便樣本進行營養分析\n4. 定期採集血液樣本檢測代謝指標\n5. 觀察動物健康狀況和行為表現",
                "route_justifications": [],
                "blood_withdrawals": [
                    {
                        "volume_ml": 3,
                        "frequency": "每2週一次，共6次",
                        "justification": "代謝指標檢測所需",
                        "total_volume_per_animal": 18
                    }
                ],
                "imaging": [],
                "restraint": [
                    {
                        "method": "徒手保定",
                        "duration": "約3-5分鐘",
                        "frequency": "採血及體重測量時",
                        "justification": "進行採血及測量所需"
                    }
                ],
                "pain": {
                    "category": "無痛或輕微不適"
                },
                "restrictions": {
                    "is_restricted": false,
                    "types": []
                },
                "endpoints": {
                    "experimental_endpoint": "完成12週營養代謝評估",
                    "humane_endpoint": "如動物出現持續食慾不振、體重下降超過原體重的20%、腹瀉超過72小時等異常狀況，經獸醫師評估後提早結束實驗。"
                },
                "final_handling": {
                    "method": "人道安樂死後進行組織採樣分析",
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
                "content": "遵循動物保護法及實驗動物使用規範，參考NRC豬隻營養需求標準",
                "references": [
                    "動物保護法",
                    "實驗動物管理與使用指引",
                    "NRC Nutrient Requirements of Swine (2012)"
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
                        "number": 36,
                        "age_min": 6,
                        "age_max": 10,
                        "age_unlimited": false,
                        "weight_min": 20,
                        "weight_max": 30,
                        "weight_unlimited": false,
                        "housing_location": "豬博士畜牧場"
                    }
                ],
                "total_animals": 36
            },
            "personnel": [
                {
                    "id": 1,
                    "name": "營養研究員X",
                    "position": "研究員",
                    "roles": ["b", "c", "d"],
                    "roles_other_text": "",
                    "years_experience": 4,
                    "trainings": ["A"],
                    "training_certificates": [
                        {
                            "training_code": "A",
                            "certificate_no": "NUTR-2024-001"
                        }
                    ]
                },
                {
                    "id": 2,
                    "name": "營養研究員Y",
                    "position": "助理研究員",
                    "roles": ["f", "g"],
                    "roles_other_text": "",
                    "years_experience": 1,
                    "trainings": ["A"],
                    "training_certificates": [
                        {
                            "training_code": "A",
                            "certificate_no": "NUTR-2024-002"
                        }
                    ]
                }
            ],
            "attachments": [],
            "signature": []
        }'::jsonb,
        '2025-04-01'::DATE,
        '2025-10-31'::DATE,
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
        '第二個草稿測試協議建立',
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
    
    RAISE NOTICE 'Second draft test protocol created: % (status: DRAFT, no IACUC number)', v_protocol_no;
END $$;