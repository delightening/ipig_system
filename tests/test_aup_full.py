"""
完整 AUP 審查流程測試

包括：
- 建立 8 個測試角色（PI, VET, IACUC_STAFF, IACUC_CHAIR, REVIEWER×3, REV_OTHER）
- 完整 14 步 AUP 審查流程
- 每步驟驗證 protocol status

用法：
    cd d:\\Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/test_aup_full.py
"""

import time
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL

# 測試帳號設定
AUP_TEST_USERS = {
    "IACUC_STAFF": {"email": "staff_int_test@example.com", "password": "password123", "display_name": "IACUC Staff (整合測試)", "role_codes": ["IACUC_STAFF"]},
    "REVIEWER1":   {"email": "rev1_int_test@example.com",  "password": "password123", "display_name": "Reviewer 1 (整合測試)", "role_codes": ["REVIEWER"]},
    "REVIEWER2":   {"email": "rev2_int_test@example.com",  "password": "password123", "display_name": "Reviewer 2 (整合測試)", "role_codes": ["REVIEWER"]},
    "REVIEWER3":   {"email": "rev3_int_test@example.com",  "password": "password123", "display_name": "Reviewer 3 (整合測試)", "role_codes": ["REVIEWER"]},
    "IACUC_CHAIR": {"email": "chair_int_test@example.com", "password": "password123", "display_name": "IACUC Chair (整合測試)", "role_codes": ["REVIEWER", "IACUC_CHAIR"]},
    "PI":          {"email": "pi_int_test@example.com",    "password": "password123", "display_name": "PI (整合測試)",         "role_codes": ["PI"]},
    "VET":         {"email": "vet_int_test@example.com",   "password": "password123", "display_name": "VET (整合測試)",        "role_codes": ["VET"]},
    "REV_OTHER":   {"email": "revother_int_test@example.com", "password": "password123", "display_name": "Reviewer Other (整合測試)", "role_codes": ["REVIEWER"]},
}


def run_aup_test() -> bool:
    """執行完整 AUP 測試，回傳是否全部通過"""
    t = BaseApiTester("AUP 完整流程測試")

    # ========================================
    # 前置作業
    # ========================================
    if not t.setup_test_users(AUP_TEST_USERS):
        return False
    if not t.login_all(AUP_TEST_USERS):
        return False

    protocol_id = None
    version_id = None

    def get_status():
        if not protocol_id:
            return "N/A"
        try:
            resp = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_id}", role="IACUC_STAFF")
            return resp.json().get("status", "Unknown")
        except Exception:
            return "Error"

    # ========================================
    # Step 1: PI 建立計畫書並提交
    # ========================================
    t.step("PI 建立計畫書並提交")
    full_content = {
        "basic": {
            "study_title": "AUP 整合測試計畫書 (完整流程)",
            "project_code": "INT-TEST-2026",
            "project_type": "Research",
            "project_category": "Medicine",
            "is_glp": True,
            "apply_study_number": f"AUP-INT-{int(time.time() * 1000) % 1000000}",
            "registration_authorities": ["FDA"],
            "pi_user_id": t.user_ids["PI"],
            "pi": {
                "name": "PI (整合測試)",
                "phone": "0912-345-678",
                "email": "pi_int_test@example.com",
                "address": "台北市南港區研究院路"
            },
            "sponsor": {
                "name": "整合測試研究基金會",
                "contact_person": "王測試",
                "contact_phone": "02-2789-0000",
                "contact_email": "sponsor@example.com"
            },
            "facility": {"id": "FAC-001", "title": "第一動物實驗中心"},
            "housing_location": "B1 無菌室"
        },
        "purpose": {
            "significance": "本研究旨在驗證 AUP 整合測試的完整流程。",
            "replacement": {
                "rationale": "目前無電腦模擬可替代。",
                "alt_search": {
                    "platforms": ["PubMed"],
                    "keywords": "AUP Integration Test",
                    "conclusion": "尚無相關自動化測試文獻。"
                }
            },
            "reduction": {"design": "使用統計學 Power Analysis 算出最小樣本數。"},
            "duplicate": {"experiment": False}
        },
        "items": {
            "use_test_item": True,
            "test_items": [{
                "name": "測試劑-A", "form": "液態", "purpose": "標記",
                "storage_conditions": "4度C", "is_sterile": True
            }]
        },
        "design": {
            "procedures": "1. 每日觀察 2. 每週秤重 3. 最後進行麻醉與採樣。",
            "anesthesia": {"is_under_anesthesia": True, "anesthesia_type": "survival_surgery"},
            "pain": {"category": "C", "management_plan": "使用止痛藥緩解不適感。"},
            "endpoints": {
                "experimental_endpoint": "計畫結束之動物處置方法。",
                "humane_endpoint": "若動物體重下降超過 20% 則進行安樂死。"
            }
        },
        "guidelines": {
            "content": "遵守實驗動物照顧及使用指引規範。",
            "references": [{"citation": "Guide for the Care and Use of Laboratory Animals", "url": "https://example.com"}]
        },
        "surgery": {
            "surgery_type": "無菌存活手術",
            "preop_preparation": "禁食 12 小時，皮膚除毛。",
            "surgery_description": "依照 SOP 進行腹腔切開。",
            "monitoring": "心跳、血氧、呼吸頻率。",
            "postop_care": "提供保溫設備與安靜環境。",
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
            "name": "PI (整合測試)", "position": "教授", "years_experience": "15",
            "roles": ["計畫主持人", "實驗操作"],
            "trainings": ["動物實驗倫理培訓", "外科手術培訓"]
        }]
    }

    ts = int(time.time())
    create_payload = {
        "title": f"AUP整合測試_{ts}",
        "working_content": full_content,
        "start_date": "2026-05-01",
        "end_date": "2027-04-01",
        "pi_user_id": t.user_ids["PI"]
    }
    resp = t._req("POST", f"{API_BASE_URL}/protocols", role="PI", json=create_payload)
    protocol_id = resp.json()["id"]
    t.record("建立計畫書", True, f"ID: {protocol_id[:8]}...")

    # 指派 Co-editor
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/co-editors", role="IACUC_STAFF",
           json={"user_id": t.user_ids["IACUC_STAFF"], "protocol_id": protocol_id})

    # PI 提交
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/submit", role="PI")
    status = get_status()
    t.record("PI 提交計畫書", status in ("submitted", "SUBMITTED"), f"status={status}")

    # ========================================
    # Step 2: Staff 行政預審 + 留言
    # ========================================
    t.step("STAFF 行政預審 + 留言")
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "PRE_REVIEW"})

    versions = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_id}/versions", role="IACUC_STAFF").json()
    version_id = versions[0]["id"]

    comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", role="IACUC_STAFF",
                      json={"protocol_version_id": version_id, "content": "請補充計畫書第三節的試驗設計細節。"}).json()
    comment_id = comment["id"]
    t.record("Staff 預審留言", True)

    # ========================================
    # Step 3: PI 回覆
    # ========================================
    t.step("PI 回覆預審意見")
    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", role="PI",
           json={"parent_comment_id": comment_id, "content": "已補充第三節試驗設計，包含統計方法與樣本數計算。"})
    t.record("PI 回覆預審意見", True)

    # ========================================
    # Step 4: Staff 要求修訂 → PI 修改重送
    # ========================================
    t.step("Staff 要求修訂 → PI 修改重送")
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "PRE_REVIEW_REVISION_REQUIRED", "remark": "請修正計畫書標題與內容"})

    t._req("PUT", f"{API_BASE_URL}/protocols/{protocol_id}", role="PI",
           json={"title": f"AUP整合測試_{ts}_rev1", "working_content": full_content})
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/submit", role="PI")
    status = get_status()
    t.record("PI 修訂後重送", status in ("resubmitted", "RESUBMITTED", "submitted", "SUBMITTED"), f"status={status}")

    # ========================================
    # Step 5: 醫療審查 (VET)
    # ========================================
    t.step("醫療審查 (VET)")
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "PRE_REVIEW"})
    t._req("POST", f"{API_BASE_URL}/reviews/comments/{comment_id}/resolve", role="IACUC_STAFF")

    vet_id = t.user_ids["VET"]
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "VET_REVIEW", "vet_id": vet_id})

    vet_comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", role="VET",
                          json={"protocol_version_id": version_id, "content": "藥物劑量偏高，建議調降至 5mg/kg。"}).json()
    vet_comment_id = vet_comment["id"]

    # PI 回覆 VET
    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", role="PI",
           json={"parent_comment_id": vet_comment_id, "content": "已依照 GLP 指引調降劑量至 5mg/kg。"})

    # Staff 退回修訂
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "VET_REVISION_REQUIRED", "remark": "依醫療意見修訂藥物劑量"})

    t._req("PUT", f"{API_BASE_URL}/protocols/{protocol_id}", role="PI",
           json={"title": f"AUP整合測試_{ts}_rev2", "working_content": full_content})
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/submit", role="PI")
    t.record("醫療審查完成", True)

    # ========================================
    # Step 6: 解決 VET 意見
    # ========================================
    t.step("解決 VET 審查意見")
    t._req("POST", f"{API_BASE_URL}/reviews/comments/{vet_comment_id}/resolve", role="VET")
    t.record("VET 意見已解決", True)

    # ========================================
    # Step 7-8: 指派委員 → 倫理審查
    # ========================================
    t.step("指派委員進入倫理審查")
    reviewers = [t.user_ids["REVIEWER1"], t.user_ids["REVIEWER2"], t.user_ids["REVIEWER3"]]
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "UNDER_REVIEW", "reviewer_ids": reviewers})
    status = get_status()
    t.record("進入倫理審查", status in ("under_review", "UNDER_REVIEW"), f"status={status}")

    # ========================================
    # Step 9: 3 名指派委員發表意見
    # ========================================
    t.step("3 名委員發表審查意見")
    version_id = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_id}/versions", role="IACUC_STAFF").json()[0]["id"]
    rev_comments = []
    reviewer_msgs = [
        "建議補充更多統計分析方法的說明，以確保樣本數的合理性。",
        "動物飼養環境條件需要更明確的描述，包含溫度與濕度控制。",
        "術後照護計畫需要包含 24 小時觀察紀錄表。"
    ]
    for i, role in enumerate(["REVIEWER1", "REVIEWER2", "REVIEWER3"]):
        c = t._req("POST", f"{API_BASE_URL}/reviews/comments", role=role,
                    json={"protocol_version_id": version_id, "content": reviewer_msgs[i]}).json()
        rev_comments.append((role, c["id"]))
    t.record("3 名委員留言", True)

    # ========================================
    # Step 10: 非指定委員留言
    # ========================================
    t.step("非指定委員 (REV_OTHER) 發表意見")
    other_comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", role="REV_OTHER",
                            json={"protocol_version_id": version_id, "content": "建議加入文獻引用以支持研究方法的選擇。"}).json()
    other_comment_id = other_comment["id"]
    t.record("非指定委員留言", True)

    # ========================================
    # Step 11: PI 回覆所有委員
    # ========================================
    t.step("PI 回覆所有審查委員")
    pi_replies = [
        "已補充詳細的統計分析方法，並附上 Power Analysis 計算結果。",
        "已更新飼養環境條件，溫度控制 22±2°C、濕度 50-60%。",
        "已新增術後 24 小時觀察紀錄表模板。"
    ]
    for i, (role, cid) in enumerate(rev_comments):
        t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", role="PI",
               json={"parent_comment_id": cid, "content": pi_replies[i]})

    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", role="PI",
           json={"parent_comment_id": other_comment_id, "content": "感謝建議，已加入相關文獻引用。"})
    t.record("PI 回覆所有意見", True)

    # ========================================
    # Step 12: Staff 要求修正
    # ========================================
    t.step("Staff 要求修正計畫書")
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "REVISION_REQUIRED"})
    t.record("Staff 要求修正", True)

    # ========================================
    # Step 13: 解決意見 → PI 重送
    # ========================================
    t.step("解決所有審查意見並重新提交")
    for role, cid in rev_comments:
        t._req("POST", f"{API_BASE_URL}/reviews/comments/{cid}/resolve", role=role)
    t._req("POST", f"{API_BASE_URL}/reviews/comments/{other_comment_id}/resolve", role="REV_OTHER")

    t._req("PUT", f"{API_BASE_URL}/protocols/{protocol_id}", role="PI",
           json={"title": f"AUP整合測試_{ts}_final", "working_content": full_content})
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/submit", role="PI")
    t.record("意見解決並重新提交", True)

    # ========================================
    # Step 14: 主委最終核定
    # ========================================
    t.step("主委最終核定")
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "UNDER_REVIEW", "reviewer_ids": reviewers})
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_CHAIR",
           json={"to_status": "APPROVED", "remark": "審核通過，核發許可證。"})
    status = get_status()
    t.record("主委核定通過", status in ("approved", "APPROVED"), f"status={status}")

    # ========================================
    # 彙總
    # ========================================
    print(f"\n{'=' * 60}")
    print(f"[完成] AUP 完整 14 步流程完成！Protocol ID: {protocol_id}")
    print(f"{'=' * 60}")
    return t.print_summary()


if __name__ == "__main__":
    try:
        success = run_aup_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n[CRITICAL ERROR] AUP 測試失敗: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
