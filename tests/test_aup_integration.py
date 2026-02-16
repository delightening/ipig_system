"""
AUP 全功能整合測試

涵蓋：
- Phase 1: 同時建立 2 個 AUP 案件（A / B 草稿）
- Phase 2: 案件 A 快速走完審查 → APPROVED
- Phase 3: 案件 B 退件修訂流程 → APPROVED
- Phase 4: 案件 A 的 Minor + Major Amendment
- Phase 5: 動物管理 — 建立動物 + 體重 / 觀察 / 疫苗
- Phase 6: 動物轉讓流程（A → B）
- Phase 7: 驗證資料隔離
- Phase 8: 附加驗證

使用方式：
    .venv\\Scripts\\python.exe tests/test_aup_integration.py
"""

import time
import sys
import os
from datetime import date, timedelta

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL
from test_fixtures import get_users_for_roles, AUP_INTEGRATION_ROLES

# ========================================
# 測試帳號設定（從 test_fixtures 統一取得）
# ========================================
USERS = get_users_for_roles(AUP_INTEGRATION_ROLES)


# ========================================
# 共用 Protocol 內容
# ========================================
def make_protocol_content(pi_name: str, pi_email: str) -> dict:
    """產生 protocol 的 working_content"""
    return {
        "basic": {
            "study_title": f"整合測試計畫 — {pi_name}",
            "project_code": f"INTEG-{int(time.time()) % 100000}",
            "project_type": "Research",
            "project_category": "Medicine",
            "is_glp": True,
            "apply_study_number": f"AUP-INTEG-{int(time.time() * 1000) % 1000000}",
            "registration_authorities": ["FDA"],
            "pi": {
                "name": pi_name,
                "phone": "0912-000-000",
                "email": pi_email,
                "address": "台北市南港區研究院路"
            },
            "sponsor": {
                "name": "整合測試基金",
                "contact_person": "王測試",
                "contact_phone": "02-0000-0000",
                "contact_email": "sponsor@example.com"
            },
            "facility": {"id": "FAC-001", "title": "第一動物實驗中心"},
            "housing_location": "B1 無菌室"
        },
        "purpose": {
            "significance": "驗證 AUP 整合流程。",
            "replacement": {
                "rationale": "無替代方案。",
                "alt_search": {"platforms": ["PubMed"], "keywords": "AUP Integration", "conclusion": "無"}
            },
            "reduction": {"design": "Power Analysis。"},
            "duplicate": {"experiment": False}
        },
        "items": {
            "use_test_item": True,
            "test_items": [{"name": "測試劑-X", "form": "液態", "purpose": "標記",
                            "storage_conditions": "4°C", "is_sterile": True}]
        },
        "design": {
            "procedures": "每日觀察、每週秤重。",
            "anesthesia": {"is_under_anesthesia": True, "anesthesia_type": "survival_surgery"},
            "pain": {"category": "C", "management_plan": "止痛藥。"},
            "endpoints": {
                "experimental_endpoint": "計畫結束之動物處置。",
                "humane_endpoint": "體重下降 > 20% 則安樂死。"
            }
        },
        "guidelines": {
            "content": "遵守實驗動物照顧及使用指引。",
            "references": [{"citation": "Guide for the Care and Use of Laboratory Animals", "url": "https://example.com"}]
        },
        "surgery": {
            "surgery_type": "無菌存活手術",
            "preop_preparation": "禁食 12 小時。",
            "surgery_description": "依照 SOP 進行。",
            "monitoring": "心跳、血氧。",
            "postop_care": "保溫。",
            "drugs": [{"drug_name": "抗生素", "dose": "10mg/kg", "route": "IM", "frequency": "QD", "purpose": "預防感染"}]
        },
        "animals": {
            "total_animals": 5,
            "animals": [{
                "species": "Pig", "strain": "L6", "sex": "MIXED",
                "number": 5, "age_min": "8", "age_max": "10", "age_unlimited": False,
                "weight_min": "20", "weight_max": "30", "weight_unlimited": False,
                "housing_location": "B1-02"
            }]
        },
        "personnel": [{
            "name": pi_name, "position": "教授", "years_experience": "10",
            "roles": ["計畫主持人"], "trainings": ["動物實驗倫理培訓"]
        }]
    }


# ========================================
# 快速審查通過 helper（14 步精簡版）
# ========================================
def approve_protocol(t: BaseApiTester, protocol_id: str, pi_role: str, label: str,
                     skip_pre_review: bool = False) -> bool:
    """
    將 protocol 從 submitted 推進到 APPROVED。
    skip_pre_review=True 時跳過 co-editor 和 PRE_REVIEW 步驟（已在該狀態時使用）。
    回傳是否成功。
    """
    def get_status():
        try:
            resp = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_id}", role="IACUC_STAFF")
            data = resp.json()
            if "protocol" in data:
                return data["protocol"].get("status", "?")
            return data.get("status", "?")
        except Exception:
            return "Error"

    if not skip_pre_review:
        # Step 0: 加入 co-editor（必要條件：進入預審前需指派試驗工作人員）
        t.sub_step(f"[{label}] 指派 co-editor")
        if "EXP_STAFF" in t.user_ids:
            t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/co-editors", role="IACUC_STAFF",
                   json={"user_id": t.user_ids["EXP_STAFF"], "protocol_id": protocol_id})

        # Step 1: Staff 預審
        t.sub_step(f"[{label}] Staff 預審")
        t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
               json={"to_status": "PRE_REVIEW"})

    versions = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_id}/versions", role="IACUC_STAFF").json()
    version_id = versions[0]["id"]

    # Step 2: VET 審查
    t.sub_step(f"[{label}] VET 審查")
    vet_id = t.user_ids["VET"]
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "VET_REVIEW", "vet_id": vet_id})

    vet_comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", role="VET",
                          json={"protocol_version_id": version_id, "content": f"[{label}] VET 審查無異議。"}).json()
    t._req("POST", f"{API_BASE_URL}/reviews/comments/{vet_comment['id']}/resolve", role="VET")

    # Step 3: 指派委員 → 倫理審查
    t.sub_step(f"[{label}] 指派委員 → 倫理審查")
    reviewers = [t.user_ids["REVIEWER1"], t.user_ids["REVIEWER2"], t.user_ids["REVIEWER3"]]
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "UNDER_REVIEW", "reviewer_ids": reviewers})

    # 每位委員留言
    version_id = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_id}/versions", role="IACUC_STAFF").json()[0]["id"]
    for role in ["REVIEWER1", "REVIEWER2", "REVIEWER3"]:
        t._req("POST", f"{API_BASE_URL}/reviews/comments", role=role,
               json={"protocol_version_id": version_id, "content": f"[{label}] 同意通過 — {role}"})

    # Step 4: 主委核定
    t.sub_step(f"[{label}] 主委核定")
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_CHAIR",
           json={"to_status": "APPROVED", "remark": f"[{label}] 審核通過。"})

    status = get_status()
    passed = status in ("approved", "APPROVED")
    t.record(f"[{label}] 審查通過", passed, f"status={status}")
    return passed


# ========================================
# 主測試函數
# ========================================
def run_aup_integration_test(ctx=None) -> bool:
    """執行 AUP 全功能整合測試"""
    t = BaseApiTester("AUP 全功能整合測試")

    # ---------- 前置作業 ----------
    if ctx:
        ctx.inject_into(t, AUP_INTEGRATION_ROLES)
        print(f"  ✓ 使用共享 Context，跳過登入（{len(AUP_INTEGRATION_ROLES)} 個角色）")
    else:
        if not t.setup_test_users(USERS):
            return False
        if not t.login_all(USERS):
            return False

    ts = int(time.time())

    # 儲存跨 Phase 的重要 ID
    protocol_a_id = None
    protocol_b_id = None
    iacuc_no_a = None
    iacuc_no_b = None
    animal_id = None
    transfer_id = None

    # ========================================
    # Phase 1：建立 2 個 AUP 案件
    # ========================================
    print(f"\n{'='*60}")
    print("[Phase 1] 建立 2 個 AUP 案件（A / B）")
    print(f"{'='*60}")

    t.step("Phase 1 — 建立案件 A (PI_A)")
    content_a = make_protocol_content("PI-A (整合測試)", "pia_integ@example.com")
    content_a["basic"]["pi_user_id"] = t.user_ids["PI_A"]
    resp_a = t._req("POST", f"{API_BASE_URL}/protocols", role="PI_A",
                     json={
                         "title": f"整合測試-A_{ts}",
                         "working_content": content_a,
                         "start_date": "2026-05-01",
                         "end_date": "2027-04-01",
                         "pi_user_id": t.user_ids["PI_A"]
                     })
    protocol_a_id = resp_a.json()["id"]
    t.record("建立案件 A", True, f"ID: {protocol_a_id[:8]}...")

    # PI_A 提交
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_a_id}/submit", role="PI_A")
    t.record("PI_A 提交案件 A", True)

    t.step("Phase 1 — 建立案件 B (PI_B)")
    content_b = make_protocol_content("PI-B (整合測試)", "pib_integ@example.com")
    content_b["basic"]["pi_user_id"] = t.user_ids["PI_B"]
    resp_b = t._req("POST", f"{API_BASE_URL}/protocols", role="PI_B",
                     json={
                         "title": f"整合測試-B_{ts}",
                         "working_content": content_b,
                         "start_date": "2026-06-01",
                         "end_date": "2027-05-01",
                         "pi_user_id": t.user_ids["PI_B"]
                     })
    protocol_b_id = resp_b.json()["id"]
    t.record("建立案件 B", True, f"ID: {protocol_b_id[:8]}...")

    # PI_B 提交
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_b_id}/submit", role="PI_B")
    t.record("PI_B 提交案件 B", True)

    # ========================================
    # Phase 2：案件 A 快速通過
    # ========================================
    print(f"\n{'='*60}")
    print("[Phase 2] 案件 A 快速通過")
    print(f"{'='*60}")

    t.step("Phase 2 — 案件 A 快速通過")
    approve_protocol(t, protocol_a_id, "PI_A", "案件A")

    # 取得 IACUC NO
    proto_a = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_a_id}", role="IACUC_STAFF").json()
    if "protocol" in proto_a:
        iacuc_no_a = proto_a["protocol"].get("iacuc_no")
    else:
        iacuc_no_a = proto_a.get("iacuc_no")
    t.record("取得案件 A IACUC No", iacuc_no_a is not None, f"iacuc_no={iacuc_no_a}")

    # ========================================
    # Phase 3：案件 B 退件修訂 → 最終通過
    # ========================================
    print(f"\n{'='*60}")
    print("[Phase 3] 案件 B 退件修訂流程")
    print(f"{'='*60}")

    t.step("Phase 3 — 案件 B 預審退件")
    # 指派 co-editor（使用 IACUC_STAFF 角色，需要 aup.review.assign 權限）
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_b_id}/co-editors", role="IACUC_STAFF",
           json={"user_id": t.user_ids["EXP_STAFF"], "protocol_id": protocol_b_id})
    # Staff 預審
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_b_id}/status", role="IACUC_STAFF",
           json={"to_status": "PRE_REVIEW"})

    versions_b = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_b_id}/versions", role="IACUC_STAFF").json()
    version_b_id = versions_b[0]["id"]

    # Staff 留預審意見
    pre_comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", role="IACUC_STAFF",
                          json={"protocol_version_id": version_b_id,
                                "content": "請補充統計方法。"}).json()
    pre_comment_id = pre_comment["id"]

    # 退件
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_b_id}/status", role="IACUC_STAFF",
           json={"to_status": "PRE_REVIEW_REVISION_REQUIRED", "remark": "請修正"})
    t.record("案件 B 預審退件", True)

    # PI_B 回覆 + 修訂重送
    t.step("Phase 3 — PI_B 修訂重送")
    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", role="PI_B",
           json={"parent_comment_id": pre_comment_id, "content": "已補充統計方法。"})
    t._req("PUT", f"{API_BASE_URL}/protocols/{protocol_b_id}", role="PI_B",
           json={"title": f"整合測試-B_{ts}_rev1", "working_content": content_b})
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_b_id}/submit", role="PI_B")
    t.record("PI_B 修訂重送", True)

    # 解決預審意見，然後快速通過
    t.step("Phase 3 — 案件 B 快速通過")
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_b_id}/status", role="IACUC_STAFF",
           json={"to_status": "PRE_REVIEW"})
    t._req("POST", f"{API_BASE_URL}/reviews/comments/{pre_comment_id}/resolve", role="IACUC_STAFF")

    approve_protocol(t, protocol_b_id, "PI_B", "案件B", skip_pre_review=True)

    # 取得 IACUC NO
    proto_b = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_b_id}", role="IACUC_STAFF").json()
    if "protocol" in proto_b:
        iacuc_no_b = proto_b["protocol"].get("iacuc_no")
    else:
        iacuc_no_b = proto_b.get("iacuc_no")
    t.record("取得案件 B IACUC No", iacuc_no_b is not None, f"iacuc_no={iacuc_no_b}")

    # ========================================
    # Phase 4：案件 A 的 Minor + Major Amendment
    # ========================================
    print(f"\n{'='*60}")
    print("[Phase 4] 案件 A Amendment")
    print(f"{'='*60}")

    # Minor Amendment
    t.step("Phase 4 — Minor Amendment")
    minor_resp = t._req("POST", f"{API_BASE_URL}/amendments", role="PI_A", json={
        "protocol_id": protocol_a_id,
        "title": f"小變更_{ts}",
        "description": "更改聯絡方式",
        "change_items": ["personnel_contact"],
        "changes_content": {"personnel": {"contact_phone": "0912-111-111"}}
    })
    minor_id = minor_resp.json()["id"]
    t._req("POST", f"{API_BASE_URL}/amendments/{minor_id}/submit", role="PI_A")
    classify_resp = t._req("POST", f"{API_BASE_URL}/amendments/{minor_id}/classify",
                            role="IACUC_STAFF", json={"amendment_type": "MINOR", "remark": "行政核准"})
    minor_status = classify_resp.json()["status"]
    t.record("Minor Amendment → ADMIN_APPROVED",
             minor_status in ("ADMIN_APPROVED", "admin_approved"),
             f"status={minor_status}")

    # Major Amendment
    t.step("Phase 4 — Major Amendment")
    major_resp = t._req("POST", f"{API_BASE_URL}/amendments", role="PI_A", json={
        "protocol_id": protocol_a_id,
        "title": f"重大變更_{ts}",
        "description": "增加動物數量",
        "change_items": ["animal_count", "design"],
        "changes_content": {
            "animals": {"total_animals": 10, "reason": "需要更多樣本"},
            "design": {"procedures": "每日觀察 + 週體重測量"}
        }
    })
    major_id = major_resp.json()["id"]
    t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/submit", role="PI_A")
    t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/classify",
           role="IACUC_STAFF", json={"amendment_type": "MAJOR", "remark": "需委員審查"})

    # 取得指派的委員
    assignments = t._req("GET", f"{API_BASE_URL}/amendments/{major_id}/assignments", role="IACUC_STAFF").json()
    t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/start-review", role="IACUC_STAFF")

    # 審查委員全部核准
    for a in assignments:
        reviewer_id = a["reviewer_id"]
        for role_name, uid in t.user_ids.items():
            if uid == reviewer_id:
                t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/decision",
                       role=role_name, json={"decision": "APPROVE", "comment": f"同意 — {role_name}"})
                break

    final_major = t._req("GET", f"{API_BASE_URL}/amendments/{major_id}", role="IACUC_STAFF").json()
    major_status = final_major["status"]
    t.record("Major Amendment → APPROVED",
             major_status in ("APPROVED", "approved"),
             f"status={major_status}")

    # ========================================
    # Phase 5：動物管理 — 建立動物 + 紀錄
    # ========================================
    print(f"\n{'='*60}")
    print("[Phase 5] 動物管理（建立 + 記錄）")
    print(f"{'='*60}")

    t.step("Phase 5 — 建立動物來源 + 動物")

    # 建立來源
    source_resp = t._req("POST", f"{API_BASE_URL}/animal-sources", role="EXP_STAFF",
                          json={
                              "code": f"SRC-INTEG-{ts % 10000}",
                              "name": "整合測試動物來源",
                              "address": "台南市善化區",
                              "contact": "陳先生",
                              "phone": "06-1234567"
                          })
    source_id = source_resp.json()["id"]
    t.record("建立動物來源", True, f"ID: {source_id[:8]}...")

    # 建立 1 隻動物（隸屬案件 A）
    animal_payload = {
        "ear_tag": f"{ts % 1000:03d}",
        "breed": "minipig",
        "gender": "male",
        "source_id": source_id,
        "entry_date": str(date.today() - timedelta(days=30)),
        "entry_weight": 22.5,
        "pen_location": "A-01",
        "iacuc_no": iacuc_no_a,
        "remark": "整合測試動物",
        "force_create": True,
    }
    animal_resp = t._req("POST", f"{API_BASE_URL}/animals", role="EXP_STAFF", json=animal_payload)
    animal_id = animal_resp.json()["id"]
    t.record("建立動物", True, f"ID: {animal_id[:8]}..., IACUC No={iacuc_no_a}")

    # 體重紀錄 — 2 筆
    t.step("Phase 5 — 體重紀錄")
    for i in range(2):
        w_date = str(date.today() - timedelta(days=(2 - i) * 7))
        w_val = 22.5 + (i + 1) * 1.5
        t._req("POST", f"{API_BASE_URL}/animals/{animal_id}/weights", role="EXP_STAFF",
               json={"measure_date": w_date, "weight": round(w_val, 1)})
    t.record("記錄體重 ×2", True)

    # 觀察紀錄 — 1 筆
    t.step("Phase 5 — 觀察紀錄")
    t._req("POST", f"{API_BASE_URL}/animals/{animal_id}/observations", role="EXP_STAFF",
           json={
               "event_date": str(date.today() - timedelta(days=5)),
               "record_type": "observation",
               "content": "動物精神良好，食慾正常。",
               "no_medication_needed": True,
               "remark": "整合測試觀察紀錄"
           })
    t.record("記錄觀察 ×1", True)

    # 疫苗紀錄 — 1 筆
    t.step("Phase 5 — 疫苗紀錄")
    t._req("POST", f"{API_BASE_URL}/animals/{animal_id}/vaccinations", role="EXP_STAFF",
           json={
               "vaccine_name": "豬瘟疫苗",
               "administered_date": str(date.today() - timedelta(days=3)),
               "dose": "2ml",
               "route": "IM",
               "remark": "整合測試疫苗"
           })
    t.record("記錄疫苗 ×1", True)

    # 驗證轉讓前紀錄數量
    t.step("Phase 5 — 驗證紀錄數量（轉讓前）")
    pre_weights = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/weights", role="EXP_STAFF").json()
    pre_obs = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/observations", role="EXP_STAFF").json()
    pre_vacc = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/vaccinations", role="EXP_STAFF").json()
    t.record("轉讓前體重紀錄", len(pre_weights) >= 2, f"{len(pre_weights)} 筆")
    t.record("轉讓前觀察紀錄", len(pre_obs) >= 1, f"{len(pre_obs)} 筆")
    t.record("轉讓前疫苗紀錄", len(pre_vacc) >= 1, f"{len(pre_vacc)} 筆")

    # ========================================
    # Phase 6：動物轉讓（A → B）
    # ========================================
    print(f"\n{'='*60}")
    print("[Phase 6] 動物轉讓 (A → B)")
    print(f"{'='*60}")

    t.step("Phase 6 — 發起轉讓")
    # 先將動物狀態依序轉換：unassigned → in_experiment → completed
    # API 強制狀態機順序，不可跳過；in_experiment 需附帶 iacuc_no
    t._req("PUT", f"{API_BASE_URL}/animals/{animal_id}", role="EXP_STAFF",
           json={
               "status": "in_experiment",
               "iacuc_no": iacuc_no_a,
               "experiment_date": str(date.today() - timedelta(days=10))
           })
    t._req("PUT", f"{API_BASE_URL}/animals/{animal_id}", role="EXP_STAFF",
           json={"status": "completed"})
    transfer_resp = t._req("POST", f"{API_BASE_URL}/animals/{animal_id}/transfers", role="EXP_STAFF",
                            json={"reason": "計畫變更，需轉至新 PI。"})
    transfer_data = transfer_resp.json()
    transfer_id = transfer_data["id"]
    t.record("發起轉讓", True, f"transfer_id={transfer_id[:8]}...")

    # 獸醫評估
    t.step("Phase 6 — 獸醫評估")
    t._req("POST", f"{API_BASE_URL}/transfers/{transfer_id}/vet-evaluate", role="VET",
           json={
               "health_status": "動物健康狀況良好，適合轉讓。",
               "is_fit_for_transfer": True,
               "conditions": None
           })
    t.record("獸醫評估通過", True)

    # 指定新計劃（案件 B）
    t.step("Phase 6 — 指定新計劃")
    t._req("PUT", f"{API_BASE_URL}/transfers/{transfer_id}/assign-plan", role="EXP_STAFF",
           json={"to_iacuc_no": iacuc_no_b})
    t.record("指定新計劃", True, f"to_iacuc_no={iacuc_no_b}")

    # PI 同意
    t.step("Phase 6 — PI 同意轉讓")
    t._req("POST", f"{API_BASE_URL}/transfers/{transfer_id}/approve", role="EXP_STAFF")
    t.record("PI_A 同意轉讓", True)

    # 完成轉讓
    t.step("Phase 6 — 完成轉讓")
    complete_resp = t._req("POST", f"{API_BASE_URL}/transfers/{transfer_id}/complete", role="EXP_STAFF")
    complete_data = complete_resp.json()
    t.record("轉讓完成", complete_data.get("status") in ("completed", "COMPLETED"),
             f"status={complete_data.get('status')}")

    # 驗證動物的 iacuc_no 更新
    animal_after = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}", role="IACUC_STAFF").json()
    new_iacuc = animal_after.get("iacuc_no")
    t.record("動物 IACUC No 更新", new_iacuc == iacuc_no_b,
             f"iacuc_no: {iacuc_no_a} → {new_iacuc}")

    # 加入轉讓後的新紀錄（應對新 PI 可見）
    t.step("Phase 6 — 轉讓後新增紀錄")
    time.sleep(1)  # 確保 created_at 在 completed_at 之後

    t._req("POST", f"{API_BASE_URL}/animals/{animal_id}/weights", role="EXP_STAFF",
           json={"measure_date": str(date.today()), "weight": 28.0})
    t._req("POST", f"{API_BASE_URL}/animals/{animal_id}/observations", role="EXP_STAFF",
           json={
               "event_date": str(date.today()),
               "record_type": "observation",
               "content": "轉讓後第一筆觀察紀錄。",
               "no_medication_needed": True,
               "remark": "轉讓後觀察"
           })
    t.record("轉讓後新增紀錄", True, "體重 ×1 + 觀察 ×1")

    # ========================================
    # Phase 7：驗證資料隔離
    # ========================================
    print(f"\n{'='*60}")
    print("[Phase 7] 驗證資料隔離")
    print(f"{'='*60}")

    t.step("Phase 7 — 取得 data-boundary")

    # PI_B 取得 data boundary（應有 boundary 值）
    boundary_resp_b = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/data-boundary", role="PI_B")
    boundary_b = boundary_resp_b.json()
    has_boundary = boundary_b.get("boundary") is not None
    t.record("PI_B 有 data boundary", has_boundary,
             f"boundary={boundary_b.get('boundary')}")

    # IACUC_STAFF 取得 data boundary（特權角色，應為 null）
    boundary_resp_staff = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/data-boundary", role="IACUC_STAFF")
    boundary_staff = boundary_resp_staff.json()
    t.record("IACUC_STAFF 無 boundary (特權)", boundary_staff.get("boundary") is None,
             f"boundary={boundary_staff.get('boundary')}")

    # VET 取得 data boundary（特權角色，應為 null）
    boundary_resp_vet = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/data-boundary", role="VET")
    boundary_vet = boundary_resp_vet.json()
    t.record("VET 無 boundary (特權)", boundary_vet.get("boundary") is None,
             f"boundary={boundary_vet.get('boundary')}")

    # 帶 after 參數查詢 — PI_B 只應看到轉讓後的紀錄
    t.step("Phase 7 — 驗證帶 after 參數的查詢結果")
    boundary_ts = boundary_b.get("boundary", "")
    after_param = f"?after={boundary_ts}" if boundary_ts else ""

    # 體重：全部 3 筆（2 筆轉讓前 + 1 筆轉讓後），加 after 應只有 1 筆
    weights_all = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/weights", role="IACUC_STAFF").json()
    weights_filtered = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/weights{after_param}", role="PI_B").json()
    t.record("體重-全部 (STAFF)",
             len(weights_all) >= 3,
             f"全部 {len(weights_all)} 筆")
    t.record("體重-過濾後 (PI_B, after)",
             len(weights_filtered) == 1,
             f"過濾後 {len(weights_filtered)} 筆 (期望 1)")

    # 觀察：全部 2 筆（1 筆轉讓前 + 1 筆轉讓後），加 after 應只有 1 筆
    obs_all = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/observations", role="IACUC_STAFF").json()
    obs_filtered = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/observations{after_param}", role="PI_B").json()
    t.record("觀察-全部 (STAFF)",
             len(obs_all) >= 2,
             f"全部 {len(obs_all)} 筆")
    t.record("觀察-過濾後 (PI_B, after)",
             len(obs_filtered) == 1,
             f"過濾後 {len(obs_filtered)} 筆 (期望 1)")

    # 疫苗：全部 1 筆（轉讓前），加 after 應 0 筆
    vacc_all = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/vaccinations", role="IACUC_STAFF").json()
    vacc_filtered = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/vaccinations{after_param}", role="PI_B").json()
    t.record("疫苗-全部 (STAFF)",
             len(vacc_all) >= 1,
             f"全部 {len(vacc_all)} 筆")
    t.record("疫苗-過濾後 (PI_B, after)",
             len(vacc_filtered) == 0,
             f"過濾後 {len(vacc_filtered)} 筆 (期望 0)")

    # ========================================
    # Phase 8：附加驗證
    # ========================================
    print(f"\n{'='*60}")
    print("[Phase 8] 附加驗證")
    print(f"{'='*60}")

    t.step("Phase 8 — 驗證轉讓歷程")
    transfers = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/transfers", role="IACUC_STAFF").json()
    t.record("轉讓歷程可查", len(transfers) >= 1, f"共 {len(transfers)} 筆轉讓記錄")

    # 驗證獸醫評估
    vet_eval = t._req("GET", f"{API_BASE_URL}/transfers/{transfer_id}/vet-evaluation", role="IACUC_STAFF").json()
    t.record("獸醫評估可查", vet_eval.get("is_fit_for_transfer") is True,
             f"fit={vet_eval.get('is_fit_for_transfer')}")

    # 驗證案件 A 的 amendment 列表
    t.step("Phase 8 — 驗證 Amendment 列表")
    proto_amendments = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_a_id}/amendments", role="IACUC_STAFF").json()
    t.record("案件 A amendment 數量",
             len(proto_amendments) >= 2,
             f"共 {len(proto_amendments)} 個 amendment")

    # 驗證無 after 的 STAFF 可看全部資料
    t.step("Phase 8 — 特權角色無隔離驗證")
    staff_weights = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/weights", role="IACUC_STAFF").json()
    staff_obs = t._req("GET", f"{API_BASE_URL}/animals/{animal_id}/observations", role="IACUC_STAFF").json()
    t.record("STAFF 體重全部可見 (無 after)", len(staff_weights) >= 3, f"{len(staff_weights)} 筆")
    t.record("STAFF 觀察全部可見 (無 after)", len(staff_obs) >= 2, f"{len(staff_obs)} 筆")

    # ========================================
    # 彙總
    # ========================================
    print(f"\n{'='*60}")
    print(f"[完成] AUP 全功能整合測試結束！")
    print(f"  Protocol A: {protocol_a_id} (IACUC: {iacuc_no_a})")
    print(f"  Protocol B: {protocol_b_id} (IACUC: {iacuc_no_b})")
    print(f"  Animal:     {animal_id}")
    print(f"  Transfer:   {transfer_id}")
    print(f"{'='*60}")
    return t.print_summary()


if __name__ == "__main__":
    try:
        success = run_aup_integration_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n[CRITICAL ERROR] AUP 整合測試失敗: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
