"""Sample payloads for /api/preview/{template_id} — taken from real source docx."""

SAMPLES: dict[str, dict] = {
    # --------------------------------------------------------------------
    "pig_approval": {
        "approval_no": "PIG-000001",
        "pi_name": "PI 1",
        "sponsor": "機構1",
        "study_title_zh": "動物實驗A",
        "study_title_en": "Animal Study A",
        "housing_location": "豬博士動物科技股份有限公司(豬博士畜牧場)",
        "animals": [
            {
                "species": "豬/迷你豬",
                "amount": "7頭",
                "period_from": "2026/02/28",
                "period_to": "2026/12/31",
            }
        ],
        "approval_date": "2026/02/28",
        "chair_name": "",
        "export_date": "2026-05-15",
    },
    # --------------------------------------------------------------------
    "review_result": {
        "initial": {
            "approved": False,
            "revise_required": True,
            "rejected": False,
            "revision_opinions": "請補充手術操作之術中動物觀察細節（參見委員一意見）。",
            "reviewer_signature": "",
            "reviewer_date": "2026/02/09",
            "convener_signature": "",
            "convener_date": "2026/02/09",
        },
        "final": {
            "approved": True,
            "revise_required": False,
            "rejected": False,
            "revision_opinions": "",
            "reviewer_signature": "",
            "reviewer_date": "2026/02/28",
            "convener_signature": "",
            "convener_date": "2026/02/28",
        },
        "export_date": "2026-05-15",
    },
    # --------------------------------------------------------------------
    "review_reply": {
        "application_no": "APIG-000001",
        "study_title": "動物實驗A",
        "pi_name": "PI 1",
        "secretary_items": [
            {
                "item_no": "p.3",
                "opinion": "為避免後續修正流程，請確認計畫期限至 2026/12/31 時間是否足夠完成實驗",
                "reply": "足夠",
            },
            {
                "item_no": "6.4",
                "opinion": "請說明切創大小",
                "reply": "切創大小約 10 cm",
            },
            {
                "item_no": "6.6",
                "opinion": "如術後出現血栓、假性動脈瘤、破裂、感染、遠端缺血，該如何處理?",
                "reply": "假性動脈瘤可以手術改善，其他術後不良情形，會影響豬隻活動，故將豬隻進行犧牲",
            },
        ],
        "vet_review": {
            "basic_info": {"status": "V", "opinion": "", "reply": ""},
            "research_purpose": {"status": "V", "opinion": "", "reply": ""},
            "necessity": {"status": "V", "opinion": "", "reply": ""},
            "experiment_design": {"status": "V", "opinion": "", "reply": ""},
            "endpoint": {"status": "V", "opinion": "", "reply": ""},
            "end_handling": {"status": "V", "opinion": "", "reply": ""},
            "hazardous": {"status": "-", "opinion": "", "reply": ""},
            "anesthesia": {"status": "V", "opinion": "", "reply": ""},
            "surgery": {"status": "X", "opinion": "如委員一回覆", "reply": ""},
            "postop_care": {"status": "V", "opinion": "", "reply": ""},
            "animal_info": {"status": "V", "opinion": "", "reply": ""},
            "personnel": {"status": "V", "opinion": "", "reply": ""},
            "signature_date": "2026.2.9",
        },
        "committee_1": [
            {
                "item_no": "6.4",
                "opinion_1st": "計畫書上只有一側股動脈的手術流程，另一側的手術流程也需在計畫書上說明",
                "reply_1st": "僅左側股動脈進行手術，右側對照組血管為未進行手術的豬隻股動脈。",
                "opinion_2nd": "我沒有其他意見，謝謝",
                "reply_2nd": "",
            }
        ],
        "committee_2": [
            {
                "item_no": "",
                "opinion_1st": "請說明術後需觀察項目，若出現血循異常，豬博士同仁應注意什麼異常狀況？",
                "reply_1st": (
                    "觀察豬隻的食慾與活動力，後肢是否跛行或腫脹，影響其站立、行走或抬腳。"
                    "觀察豬的後肢末梢組織是否因缺血而變色或壞死。"
                    "縫合傷口是否發炎或潰爛難以癒合。"
                    "輕微按壓後肢傷口及其周邊區域來評估脈搏或體溫是否異常。"
                ),
                "opinion_2nd": "我無其他意見",
                "reply_2nd": "",
            }
        ],
        "committee_3": [
            {"item_no": "", "opinion_1st": "本案無意見", "reply_1st": "", "opinion_2nd": "", "reply_2nd": ""}
        ],
        "committee_4": [
            {
                "item_no": "6.4",
                "opinion_1st": (
                    "一、兔去細胞瓣膜之異種移植手術「反式植入經超臨界二氧化碳去細胞之兔瓣膜移植物，"
                    "以 7-0 PROLENE 縫線連續縫合方式端對端吻合。」請問是指左側股動脈的 ScCO₂ "
                    "去細胞兔瓣、ePTFE 5 mm 與 SDS 去細胞兔瓣等植入操作？右側股動脈對照組血管會有何種處置？"
                ),
                "reply_1st": (
                    "指的是左側股動脈的 ScCO₂ 去細胞兔瓣、ePTFE 5 mm 與 SDS 去細胞兔瓣等植入操作；"
                    "右側對照組血管為未進行手術的豬隻股動脈。"
                ),
                "opinion_2nd": "本案無其他意見",
                "reply_2nd": "",
            }
        ],
        "export_date": "2026-05-15",
    },
    # --------------------------------------------------------------------
    "aup_protocol": {
        "cover": {
            "study_title_zh": "動物實驗A",
            "study_title_en": "Animal Study A",
            "sponsor": "機構1",
            "testing_facility": "豬博士動物科技股份有限公司",
        },
        "pi": {
            "name": "PI 1",
            "title": "教授",
            "phone": "02-1234-5678",
            "email": "pi@example.org",
            "address": "某市某區",
            "sponsor_name": "機構1",
            "contact_person": "聯絡人1",
            "contact_phone": "02-1234-5678",
            "contact_email": "assistant@example.org",
        },
        "sd": {"name": "豬博士專案經理", "email": "sd@drpig.example"},
        "facility": {
            "title": "豬博士畜牧場",
            "address": "彰化縣芳苑鄉",
        },
        "protocol": {
            "iacuc_apply_no": "APIG-000001",
            "iacuc_approval_no": "PIG-000001",
            "apply_date": "2026/02/01",
            "approval_date": "2026/02/28",
            "valid_from": "2026/02/28",
            "valid_to": "2026/12/31",
            "is_glp": False,
        },
        "sections": {
            "abstract": "本計畫探討動物實驗A的耐久性與耐血栓表現……（摘要省略）",
            "alternatives_search": "已查 ALTBIB / DB-ALM / TAAT；關鍵字「porcine valve xenograft」、「decellularized heart valve」確認無可行替代方案。",
            "alternatives_databases": ["ALTBIB", "DB-ALM", "TAAT", "EURL ECVAM", "JHU CAAT", "NC3Rs EDA", "NC3Rs Refinement DB"],
            "alternatives_keywords": "porcine valve xenograft、decellularized heart valve",
            "alternatives_conclusion": "未發現可行非動物性替代方案，本試驗具不可替代性。",
            "duplicate_experiment": "否",
            "duplicate_status": "no",
            "duplicate_prev_iacuc": "",
            "duplicate_n_a_basis": "",
            "duplicate_yes_note": "",
            "reduction_rationale": "依 power analysis n=7 為偵測 effect size 0.8 所需最小樣本。",
            "special_care": "術後 7 日加溫保暖、軟食。",
            "individual_housing": "否",
            "reuse_after_study": "否，全部結案安樂死。",
            "refinement_measures": "提供環境豐富化（玩具球、鏈條）；麻醉前 IM 鎮靜；術後止痛 7 日。",
            "live_animal_necessity": "瓣膜在體血流動力學無法以 in-vitro 或 in-silico 模型完整重現。",
            "test_article_explanation": "ScCO₂ 去細胞兔瓣膜 + ePTFE 5mm + SDS 去細胞兔瓣膜，與未處理對照組比較。",
            "pain_distress_mitigation": "麻醉前 IM 鎮靜；術後 SID 止痛 7 日；術後傷口護理。",
            "endpoint_humane": (
                "實驗終點：植入後 12 週影像檢查。\n人道終點：體重下降 >20%、無法進食、傷口感染惡化、跛行 >7 日。"
            ),
            "carcass_disposal": "委由金海龍生物科技股份有限公司化製處理（化製廠管編 P6001213）。",
            "non_pharma_grade": "否",
            "hazardous_disposal": "不適用",
            "hazardous_application": "不適用",
            "hazardous_protection": "不適用",
            "references": "1. ATS Guidelines 2023…\n2. NIH RePORTER…",
            "pre_surgery_prep": (
                "1. 術前禁食至少 12 小時，不禁水。\n"
                "2. 畜舒坦 3-5 mg/kg + Atropine 0.03-0.05 mg/kg IM 鎮靜。\n"
                "3. Zoletil-50 4.4 mg/kg IM 誘導麻醉，氣管插管後 Isoflurane 0.5-2% 維持。\n"
                "4. 術前 Cefazolin 15 mg/kg + meloxicam 0.4 mg/kg。"
            ),
            "surgery_description": (
                "切創長度約 10 cm。左側股動脈分離 → 反式植入兔瓣膜，7-0 PROLENE 端對端吻合。"
            ),
            "intraop_monitoring": "心跳 / 呼吸 / 體溫每 30 分鐘記錄。",
            "postop_impact": "術後可能出現後肢跛行 / 缺血變色 / 傷口感染；獸醫師逐日評估。",
            "multiple_surgery": "否",
            "postop_care": "術後 7 日 SID 止痛 + 抗生素，異常情形通報獸醫師。",
            "surgery_endpoint": "植入後 12 週影像追蹤後安樂死。",
            "personnel_other_role": "",
        },
        "drugs": [
            {"name": "Atropine", "dose": "1 mg/mL", "route": "IM", "frequency": "麻醉誘導前 1 次", "purpose": "麻醉誘導"},
            {"name": "畜舒坦 (Azaperonum)", "dose": "3-5 mg/kg", "route": "IM", "frequency": "麻醉誘導前 1 次", "purpose": "麻醉誘導"},
            {"name": "Zoletil-50", "dose": "4.4 mg/kg", "route": "IM", "frequency": "麻醉誘導前 1 次", "purpose": "麻醉誘導"},
            {"name": "Cefazolin", "dose": "15-30 mg/kg", "route": "IM", "frequency": "術前 1 次；術後每日 1 次", "purpose": "抗生素"},
            {"name": "Meloxicam", "dose": "0.1-0.4 mg/kg", "route": "IM", "frequency": "術前 1 次；術後每日 1 次", "purpose": "止痛"},
            {"name": "Isoflurane", "dose": "0.5-2%", "route": "吸入", "frequency": "術中持續吸入", "purpose": "麻醉維持"},
        ],
        "animals": [
            {
                "species_breed": "豬/迷你豬",
                "sex_count": "公 4 頭 / 母 3 頭",
                "age": "6-8 月齡",
                "weight": "30-40 kg",
                "source": "豬博士畜牧場",
                "housing": "豬博士畜牧場",
            }
        ],
        "personnel": [
            {"seq": "1", "name": "使用者A", "position": "獸醫師", "roles": "b, c, d, f, g, h", "years": "7", "trainings": "A. IACUC 訓練班 證號 IACUC-2021-A001, C. 輻射安全訓練班 證號 RAD-2022-038, C. 輻射安全訓練班 證號 RAD-2024-112"},
            {"seq": "2", "name": "使用者B", "position": "獸醫師", "roles": "b, c, d, f, g, h", "years": "7", "trainings": "A. IACUC 訓練班 證號 IACUC-2020-B017, C. 輻射安全訓練班 證號 RAD-2021-005, A. IACUC 訓練班 證號 IACUC-2023-C042, C. 輻射安全訓練班 證號 RAD-2025-031"},
            {"seq": "3", "name": "使用者C", "position": "技術員", "roles": "b, c, d, f, g, h", "years": "5", "trainings": "A. IACUC 訓練班 證號 IACUC-2022-D008, C. 輻射安全訓練班 證號 RAD-2023-052, C. 輻射安全訓練班 證號 RAD-2025-104"},
            {"seq": "4", "name": "使用者D", "position": "技術員", "roles": "b, c, d, f, g, h", "years": "6", "trainings": "A. IACUC 訓練班 證號 IACUC-2021-E021, A. IACUC 訓練班 證號 IACUC-2024-F063, C. 輻射安全訓練班 證號 RAD-2022-019, C. 輻射安全訓練班 證號 RAD-2025-128"},
            {"seq": "5", "name": "使用者E", "position": "技術員", "roles": "b, c, d, f, g, h", "years": "2", "trainings": "A. IACUC 訓練班 證號 IACUC-2024-G099, C. 輻射安全訓練班 證號 RAD-2024-077, C. 輻射安全訓練班 證號 RAD-2025-156"},
        ],
        "export_date": "2026-05-15",
        "anesthesia": {
            "survival_surgery": True,
            "non_survival_surgery": False,
            "isoflurane_only": False,
            "azaperonum_atropine_isoflurane": True,
            "none": False,
        },
        "pain": {
            "cat_b": {"breeding_only": False, "other": False, "other_text": ""},
            "cat_c": {"handling_weighing": True, "injection_oral": True, "marking": False, "routine_husbandry": False, "full_anesthesia": True, "avma_euthanasia": True, "other": False, "other_text": ""},
            "cat_d": {"survival_surgery_under_anesthesia": True, "pain_with_analgesia": True, "transport_with_sedative": False, "intubation": True, "non_survival_surgery_under_anesthesia": False, "sub_lethal_chemical": False, "catheter_implant": False, "bleeding_perfusion": False, "food_water_restriction": False, "induced_pathology": False, "chemical_damage": False, "eye_skin_irritation": False, "other": False, "other_text": ""},
            "cat_e": {"chemical_severe_damage": False, "paralytic_no_anesthesia": False, "burns_skin_trauma": False, "induced_disease": False, "near_pain_threshold": False, "chronic_pain_disease": False, "extreme_food_water_restriction": False, "extreme_environment": False, "lethal_procedures": False, "pain_distress_research": False, "other": False, "other_text": ""},
        },
        "surgery_type": {"survival": True, "non_survival": False},
        "aseptic": {
            "surgical_site_disinfection": True,
            "instrument_disinfection": True,
            "sterile_gowns_gloves": True,
            "sterile_drapes": True,
            "surgical_hand_disinfection": True,
        },
        "end_handling": {
            "eutha_kcl": True,
            "eutha_electrocution": False,
            "eutha_other": False,
            "eutha_other_text": "",
            "transfer": False,
            "transfer_recipient_name": "",
            "transfer_recipient_org": "",
            "transfer_project": "",
            "other": False,
            "other_text": "",
        },
    },
    # --------------------------------------------------------------------
    "vet_patrol": {
        "header": {
            "inspector_name": "",
            # patrol_date / period 由 server 自動填（_auto_fill_vet_patrol）
            "period": "PM",
        },
        # 對齊 reference PDF — 真實 pen 配置（多行 G 區 group cells）
        "pens": {
            # D 區
            "D17": {"ear_tags": "627", "status": "○"},
            "D14": {"ear_tags": "004", "status": "○"},
            "D12": {"ear_tags": "654", "status": "○"},
            "D11": {"ear_tags": "661", "status": "○"},
            "D10": {"ear_tags": "191", "status": "○"},
            "D09": {"ear_tags": "797", "status": "○"},
            "D08": {"ear_tags": "807", "status": "○"},
            "D07": {"ear_tags": "760", "status": "○"},
            "D06": {"ear_tags": "802", "status": "○"},
            "D05": {"ear_tags": "651", "status": "○"},
            "D04": {"ear_tags": "656", "status": "○"},
            "D03": {"ear_tags": "652", "status": "○"},
            "D02": {"ear_tags": "695", "status": "○"},
            "D01": {"ear_tags": "806", "status": "○"},
            "D31": {"ear_tags": "700", "status": "○"},
            "D29": {"ear_tags": "786", "status": "●"},
            "D28": {"ear_tags": "785", "status": "●"},
            "D27": {"ear_tags": "784", "status": "●"},
            "D26": {"ear_tags": "744", "status": "●"},
            "D25": {"ear_tags": "727", "status": "●"},
            "D24": {"ear_tags": "640", "status": "●"},
            "D23": {"ear_tags": "726", "status": "●"},
            "D22": {"ear_tags": "725", "status": "●"},
            "D21": {"ear_tags": "763", "status": "○"},
            "D19": {"ear_tags": "663", "status": "○"},
            # E 區
            "E23": {"ear_tags": "245", "status": "○"},
            "E22": {"ear_tags": "243", "status": "○"},
            "E19": {"ear_tags": "662", "status": "○"},
            "E18": {"ear_tags": "755", "status": "○"},
            "E17": {"ear_tags": "751", "status": "○"},
            "E14": {"ear_tags": "006", "status": "○"},
            "E13": {"ear_tags": "677", "status": "●"},
            "E12": {"ear_tags": "699", "status": "●"},
            "E11": {"ear_tags": "698", "status": "●"},
            "E10": {"ear_tags": "683", "status": "●"},
            "E09": {"ear_tags": "686", "status": "●"},
            "E08": {"ear_tags": "692", "status": "●"},
            "E07": {"ear_tags": "697", "status": "●"},
            "E06": {"ear_tags": "680", "status": "●"},
            "E04": {"ear_tags": "616", "status": "●"},
            "E03": {"ear_tags": "609", "status": "●"},
            "E02": {"ear_tags": "619", "status": "●"},
            "E01": {"ear_tags": "617", "status": "●"},
            # C 區
            "C10": {"ear_tags": "678", "status": "○"},
            "C09": {"ear_tags": "649", "status": "○"},
            "C08": {"ear_tags": "635", "status": "○"},
            "C07": {"ear_tags": "638", "status": "○"},
            "C06": {"ear_tags": "473", "status": "○"},
            "C05": {"ear_tags": "766", "status": "○"},
            "C04": {"ear_tags": "453", "status": "○"},
            "C03": {"ear_tags": "472", "status": "○"},
            "C02": {"ear_tags": "771", "status": "○"},
            "C01": {"ear_tags": "598", "status": "○"},
            "C20": {"ear_tags": "577", "status": "○"},
            "C19": {"ear_tags": "414", "status": "○"},
            "C18": {"ear_tags": "814.815", "status": "○"},
            "C17": {"ear_tags": "817.816", "status": "○"},
            "C16": {"ear_tags": "653", "status": "○"},
            "C15": {"ear_tags": "704", "status": "○"},
            "C14": {"ear_tags": "655", "status": "○"},
            "C13": {"ear_tags": "410", "status": "○"},
            # A 區
            "A08": {"ear_tags": "003", "status": "○"},
            "A06": {"ear_tags": "246", "status": "○"},
            "A05": {"ear_tags": "009", "status": "○"},
            "A04": {"ear_tags": "008", "status": "○"},
            "A03": {"ear_tags": "002", "status": "○"},
            "A20": {"ear_tags": "613", "status": "○"},
            "A19": {"ear_tags": "664", "status": "●"},
            "A18": {"ear_tags": "643", "status": "○"},
            "A17": {"ear_tags": "819.823", "status": "○"},
            "A16": {"ear_tags": "820.821.822", "status": "○"},
            "A15": {"ear_tags": "818.824", "status": "○"},
            "A14": {"ear_tags": "623", "status": "○"},
            "A13": {"ear_tags": "592", "status": "●"},
            "A12": {"ear_tags": "639", "status": "●"},
            # B 區
            "B10": {"ear_tags": "", "status": "●"},
            "B08": {"ear_tags": "", "status": "●"},
            "B07": {"ear_tags": "642", "status": "●"},
            "B06": {"ear_tags": "633", "status": "●"},
            "B04": {"ear_tags": "", "status": "●"},
            "B03": {"ear_tags": "622", "status": "●"},
            "B02": {"ear_tags": "621", "status": "●"},
            # G 區（多行 group cells — 測試高度鎖死 + line-height 收斂）
            "G06": {"ear_tags": "249.250.251.252.253", "status": "○"},
            "G05": {"ear_tags": "719.739.773.705", "status": "○"},
            "G04": {"ear_tags": "759.740.007.005", "status": "○"},
            "G03": {"ear_tags": "674.681.687.688.689.690.691.696", "status": "○"},
        },
        "export_date": "",
    },
    # --------------------------------------------------------------------
    "audit_log": {
        "meta": {
            "system_name": "豬博士動物實驗管理系統",
            "period_from": "2026-05-01 00:00",
            "period_to": "2026-05-15 23:59",
            "exported_by": "陳獸醫",
            "export_time": "2026-05-16 09:00:00",
        },
        "summary": {
            "total_count": 142,
            "user_count": 6,
            "failure_count": 3,
            "admin_count": 12,
        },
        "entries": [
            {
                "timestamp": "2026-05-15 14:23:11",
                "user": "陳獸醫",
                "action": "CREATE_OBSERVATION",
                "resource": "animals/A-003/observations",
                "ip": "10.0.0.42",
                "change_summary": "新增觀察：食慾正常",
            },
            {
                "timestamp": "2026-05-15 09:11:02",
                "user": "Anonymous",
                "action": "LOGIN_FAILED",
                "resource": "auth/login",
                "ip": "203.0.113.7",
                "change_summary": "帳號鎖定 (5 次失敗)",
            },
            {
                "timestamp": "2026-05-14 16:45:50",
                "user": "SYSTEM",
                "action": "SCHEDULED_BACKUP",
                "resource": "system/backup",
                "ip": "-",
                "change_summary": "每日備份完成 (412 MB)",
            },
        ],
        "signature": {
            "admin_name": "PI 1",
            "admin_signature": "簽章：__________ 日期：2026-05-16",
        },
    },
    # --------------------------------------------------------------------
    "blood_test": {
        "animal_ear_tag": "627",
        "animal_iacuc_no": "PIG-000001",
        "export_date": "2026-05-15",
        "exporter_name": "陳獸醫",
        "items": [
            {
                "test_date": "2026-05-01",
                "item_name": "WBC",
                "result_value": "8.2 10^3/uL",
                "reference_range": "5.0 - 21.0",
                "is_abnormal": False,
                "abnormal_mark": "",
                "created_by_name": "陳獸醫",
            },
            {
                "test_date": "2026-05-01",
                "item_name": "RBC",
                "result_value": "5.4 10^6/uL",
                "reference_range": "5.0 - 8.0",
                "is_abnormal": False,
                "abnormal_mark": "",
                "created_by_name": "陳獸醫",
            },
            {
                "test_date": "2026-05-01",
                "item_name": "HGB",
                "result_value": "9.8 g/dL",
                "reference_range": "10.0 - 16.0",
                "is_abnormal": True,
                "abnormal_mark": "✓",
                "created_by_name": "陳獸醫",
            },
        ],
    },
    # --------------------------------------------------------------------
    "medical_record": {
        "animal": {
            "ear_tag": "627",
            "iacuc_no": "PIG-000001",
            "breed_zh": "迷你豬",
            "gender_zh": "公",
            "birth_date": "2025-08-15",
            "entry_date": "2026-02-28",
            "source_name": "豬博士畜牧場",
            "entry_weight": "32.0",
        },
        "vaccinations": [
            {"administered_date": "2026-03-05", "vaccine": "豬瘟疫苗", "deworming_dose": "無"},
            {"administered_date": "2026-03-12", "vaccine": "無", "deworming_dose": "Ivermectin 0.3 mg/kg"},
        ],
        "events": [
            {"event_date": "2026-03-01", "weight": "32.0", "content": ""},
            {"event_date": "2026-03-15", "weight": "34.5", "content": ""},
            {"event_date": "2026-04-02", "weight": "", "content": "左眼(手術) — 植入試驗裝置"},
            {"event_date": "2026-04-05", "weight": "", "content": "術後觀察：食慾正常，活動力佳", "medications": "[抗生素] Amoxicillin 5 mg/kg（IM 肌肉注射）"},
            {"event_date": "2026-04-15", "weight": "37.0", "content": ""},
        ],
        "export_date": "2026-05-15",
    },
    # --------------------------------------------------------------------
    "surgery": {
        "animal": {
            "ear_tag": "627",
            "iacuc_no": "PIG-000001",
            "breed_zh": "迷你豬",
            "gender_zh": "公",
            "birth_date": "2025-08-15",
        },
        "surgery": {
            "surgery_date": "2026-04-02",
            "surgery_site": "左股動脈",
            "body_weight": "35.0 Kg",
            "pre_op_observation": "精神狀況佳，無異常。",
            "induction_anesthesia": "畜舒坦 4 mg/kg IM\nAtropine 0.04 mg/kg IM\nZoletil-50 4.4 mg/kg IM",
            "pre_op_medication": "Cefazolin 15 mg/kg IM\nMeloxicam 0.4 mg/kg IM",
            "posture": "右側臥",
            "gas_anesthesia": "Isoflurane 1.5% / O2 2 L/min",
            "anesthesia_observation": "誘導順利，無嗆咳。",
        },
        "recorded_by": "使用者A",
        "vital_signs": [
            {"respiration_method": "IPPV", "time": "09:00", "heart_rate": "92", "respiration_rate": "16", "temperature": "37.8", "spo2": "98"},
            {"respiration_method": "IPPV", "time": "09:30", "heart_rate": "88", "respiration_rate": "14", "temperature": "37.6", "spo2": "99"},
            {"respiration_method": "IPPV", "time": "10:00", "heart_rate": "90", "respiration_rate": "15", "temperature": "37.5", "spo2": "98"},
        ],
        "post_op": {
            "reflex_recovery": "10:45 開始有反射，11:00 完全恢復",
            "spontaneous_respiration_rate": "18 次/分鐘",
            "post_op_medication": "Cefazolin SID × 5 日 + Meloxicam SID × 5 日",
            "remark": "傷口縫合整齊，無滲血。",
        },
        "pain_assessments": [
            {"record_date": "2026-04-02", "post_op_day": "0", "time_period": "下午", "fasted": "是", "standing": "1", "walking": "1", "wound_site": "0", "wound_condition": "0", "behavior": "1", "appetite": "1", "defecation": "1", "urination": "1", "pain_score": "2", "total_score": "9", "medication_or_remark": "Meloxicam 0.4 mg/kg IM"},
            {"record_date": "2026-04-03", "post_op_day": "1", "time_period": "上午", "fasted": "否", "standing": "1", "walking": "1", "wound_site": "0", "wound_condition": "0", "behavior": "0", "appetite": "0", "defecation": "0", "urination": "0", "pain_score": "1", "total_score": "3", "medication_or_remark": "食慾恢復正常"},
        ],
        "export_date": "2026-05-15",
    },
    # --------------------------------------------------------------------
    "warehouse": {
        "warehouse": {
            "code": "WH-MAIN",
            "name": "主倉",
            "address": "彰化縣芳苑鄉",
        },
        "summary": {
            "total_locations": 24,
            "active_locations": 18,
            "total_capacity": 480,
            "total_current_count": 312,
            "total_inventory_items": 47,
        },
        "inventory_rows": [
            {"location_code": "A-01", "location_name": "藥品區 A1", "product_name": "Cefazolin", "spec": "1g/vial", "batch_no": "CFZ-2025-08", "quantity": "24", "unit": "vial", "expiry_date": "2027-08-31"},
            {"location_code": "A-02", "location_name": "藥品區 A2", "product_name": "Meloxicam", "spec": "20mg/mL 50mL", "batch_no": "MLX-2025-11", "quantity": "8", "unit": "bottle", "expiry_date": "2027-11-30"},
            {"location_code": "B-05", "location_name": "麻醉藥區 B5", "product_name": "Isoflurane", "spec": "250 mL", "batch_no": "ISO-2026-02", "quantity": "12", "unit": "bottle", "expiry_date": "2028-02-28"},
            {"location_code": "C-12", "location_name": "耗材區 C12", "product_name": "PROLENE 7-0", "spec": "75cm", "batch_no": "PRL-2026-03", "quantity": "30", "unit": "pack", "expiry_date": "—"},
        ],
        "generated_at": "2026-05-15 10:00",
        "exporter_name": "陳獸醫",
    },
    # --------------------------------------------------------------------
    "vet_patrol_report": {
        "vet_name": "陳獸醫",
        "companion": "助理王",
        "patrol_date": "2026-05-10",
        "patrol_date_display": "2026年05月10日",
        "categories": [
            {
                "label": "豬隻狀況",
                "observation": "#001 食慾正常\n#012 後肢輕微跛行，建議追蹤",
                "suggestion": "持續觀察 3 日；若惡化加投止痛。",
                "follow_up": "",
                "photos": [],
            },
            {
                "label": "環境清潔",
                "observation": "A 區走道有少量積水，已通知清潔。",
                "suggestion": "",
                "follow_up": "",
                "photos": [],
            },
        ],
        "report_photos": [],
    },
}
