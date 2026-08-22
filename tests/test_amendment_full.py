# -*- coding: utf-8 -*-
"""
完整 Amendment（變更申請）流程測試

測試流程：
1. 建立 AUP 計畫並走完核准流程（簡化版）
2. PI 建立 Amendment
3. PI 提交 Amendment
4. IACUC_STAFF 分類為 Minor → 自動 ADMIN_APPROVED
5. PI 建立第二個 Amendment（Major 路徑）
6. PI 提交 → IACUC_STAFF 分類為 Major → CLASSIFIED
7. IACUC_STAFF 開始審查 → UNDER_REVIEW
8. 審查委員記錄決議（全部核准） → 最終 APPROVED
9. 驗證版本歷程與狀態歷程
10. 驗證 Protocol amendments 列表

用法：
    cd d:\\Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/test_amendment_full.py
"""

import time
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL
from test_fixtures import get_users_for_roles, AMENDMENT_ROLES

# 測試帳號設定（從 test_fixtures 統一取得，與 AUP 共用）
AMENDMENT_TEST_USERS = get_users_for_roles(AMENDMENT_ROLES)


def create_approved_protocol(t) -> str:
    """建立並核准一個 AUP 計畫書（簡化流程），回傳 protocol_id"""
    ts = int(time.time())

    # 最小可提交的內容
    content = {
        "basic": {
            "study_title": f"Amendment 測試用計畫書 {ts}",
            "project_code": f"AMD-TEST-{ts % 10000}",
            "project_type": "Research",
            "project_category": "Medicine",
            "is_glp": True,
            "apply_study_number": f"AMD-{ts % 1000000}",
            "registration_authorities": ["FDA"],
            "pi_user_id": t.user_ids["PI"],
            "pi": {"name": "PI", "phone": "0912-000-000", "email": "pi@test.com", "address": "台北"},
            "sponsor": {"name": "測試研究贊助商", "contact_person": "自測員", "contact_phone": "02-0000-0000", "contact_email": "s@test.com"},
            "facility": {"id": "FAC-001", "title": "第一動物實驗中心"},
            "housing_location": "B1"
        },
        "purpose": {
            "significance": "測試用途",
            "replacement": {"rationale": "無替代方案", "alt_search": {"platforms": ["PubMed"], "keywords": "test", "conclusion": "無"}},
            "reduction": {"design": "最小樣本"},
            "duplicate": {"experiment": False}
        },
        "items": {"use_test_item": False},
        "design": {
            "procedures": "每日觀察",
            "anesthesia": {"is_under_anesthesia": False},
            "pain": {"category": "B", "management_plan": "無"},
            "endpoints": {"experimental_endpoint": "計畫結束", "humane_endpoint": "體重下降 20%"}
        },
        "guidelines": {"content": "依循規範"},
        "animals": {
            "total_animals": 2,
            "animals": [{"species": "Pig", "strain": "L6", "sex": "MIXED", "number": 2,
                         "age_min": "8", "age_max": "10", "age_unlimited": False,
                         "weight_min": "20", "weight_max": "30", "weight_unlimited": False,
                         "housing_location": "B1"}]
        },
        "personnel": [{"name": "PI", "position": "教授", "years_experience": "10",
                       "roles": ["計畫主持人"], "trainings": ["動物實驗倫理研習"]}]
    }

    # 1. PI 建立並提交
    create_resp = t._req("POST", f"{API_BASE_URL}/protocols", role="PI", json={
        "title": f"Amendment測試計畫_{ts}",
        "working_content": content,
        "start_date": "2026-05-01",
        "end_date": "2027-04-01",
        "pi_user_id": t.user_ids["PI"]
    })
    protocol_id = create_resp.json()["id"]

    # 指派 Co-editor
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/co-editors", role="IACUC_STAFF",
           json={"user_id": t.user_ids["IACUC_STAFF"], "protocol_id": protocol_id})

    # PI 提交
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/submit", role="PI")

    # Staff 預審
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "PRE_REVIEW"})

    # 進入倫理審查
    reviewers = [t.user_ids["REVIEWER1"], t.user_ids["REVIEWER2"], t.user_ids["REVIEWER3"]]
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_STAFF",
           json={"to_status": "UNDER_REVIEW", "reviewer_ids": reviewers})

    # 3 位委員發言
    versions = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_id}/versions", role="IACUC_STAFF").json()
    version_id = versions[0]["id"]
    for role in ["REVIEWER1", "REVIEWER2", "REVIEWER3"]:
        t._req("POST", f"{API_BASE_URL}/reviews/comments", role=role,
               json={"protocol_version_id": version_id, "content": f"OK from {role}"})

    # 主委核准
    t._req("POST", f"{API_BASE_URL}/protocols/{protocol_id}/status", role="IACUC_CHAIR",
           json={"to_status": "APPROVED", "remark": "核准"})

    return protocol_id


def run_amendment_test(ctx=None, protocol_id=None) -> bool:
    """執行完整 Amendment 測試，回傳是否全部通過

    Args:
        ctx: 共享 Context（有的話跳過登入）
        protocol_id: 已核准的 AUP protocol_id（有的話跳過建立 AUP）
    """
    t = BaseApiTester("Amendment 完整流程測試")

    # ========================================
    # 前置作業：帳號建立＆登入（有 ctx 時跳過）
    # ========================================
    if ctx:
        ctx.inject_into(t, AMENDMENT_ROLES)
        print(f"  ✓ 使用共享 Context，跳過登入（{len(AMENDMENT_ROLES)} 個角色）")
    else:
        if not t.setup_test_users(AMENDMENT_TEST_USERS):
            return False
        if not t.login_all(AMENDMENT_TEST_USERS):
            return False

    # ========================================
    # Step 1: 建立已核准的 AUP 計畫書
    # ========================================
    if protocol_id:
        t.step("使用外部已核准的 AUP 計畫書")
        t.record("複用已核准計畫書", True, f"Protocol ID: {protocol_id[:8]}...（跳過重建）")
    else:
        t.step("建立已核准的 AUP 計畫書")
        try:
            protocol_id = create_approved_protocol(t)
            t.record("建立已核准計畫書", True, f"Protocol ID: {protocol_id[:8]}...")
        except Exception as e:
            t.record("建立已核准計畫書", False, str(e))
            return t.print_summary()

    # ========================================
    # Step 2: PI 建立 Amendment (Minor 路徑)
    # ========================================
    t.step("PI 建立 Amendment（Minor 路徑）")
    ts = int(time.time())
    minor_resp = t._req("POST", f"{API_BASE_URL}/amendments", role="PI", json={
        "protocol_id": protocol_id,
        "title": f"小變更測試_{ts}",
        "description": "修改實驗人員聯絡資訊",
        "change_items": ["personnel_contact"],
        "changes_content": {"personnel": {"contact_phone": "0912-111-111"}}
    })
    minor_amendment = minor_resp.json()
    minor_id = minor_amendment["id"]
    t.record("建立 Minor Amendment", True, f"ID: {minor_id[:8]}..., status={minor_amendment['status']}")

    # ========================================
    # Step 3: PI 提交 Minor Amendment
    # ========================================
    t.step("PI 提交 Minor Amendment")
    submit_resp = t._req("POST", f"{API_BASE_URL}/amendments/{minor_id}/submit", role="PI")
    minor_status = submit_resp.json()["status"]
    t.record("提交 Minor Amendment", minor_status in ("SUBMITTED", "submitted"),
             f"status={minor_status}")

    # ========================================
    # Step 4: IACUC_STAFF 分類為 Minor → 自動 ADMIN_APPROVED
    # ========================================
    t.step("IACUC_STAFF 分類為 Minor（自動行政核准）")
    classify_resp = t._req("POST", f"{API_BASE_URL}/amendments/{minor_id}/classify",
                            role="IACUC_STAFF", json={
                                "amendment_type": "MINOR",
                                "remark": "小變更，行政核准"
                            })
    minor_final_status = classify_resp.json()["status"]
    t.record("Minor 分類 → ADMIN_APPROVED",
             minor_final_status in ("ADMIN_APPROVED", "admin_approved"),
             f"status={minor_final_status}")

    # ========================================
    # Step 5: PI 建立 Amendment (Major 路徑)
    # ========================================
    t.step("PI 建立 Amendment（Major 路徑）")
    major_resp = t._req("POST", f"{API_BASE_URL}/amendments", role="PI", json={
        "protocol_id": protocol_id,
        "title": f"重大變更測試_{ts}",
        "description": "增加實驗動物數量",
        "change_items": ["animal_count", "design"],
        "changes_content": {
            "animals": {"total_animals": 10, "reason": "統計分析需要更多樣本"},
            "design": {"procedures": "每日觀察 + 血液檢測"}
        }
    })
    major_amendment = major_resp.json()
    major_id = major_amendment["id"]
    t.record("建立 Major Amendment", True, f"ID: {major_id[:8]}...")

    # ========================================
    # Step 6: PI 提交 → IACUC_STAFF 分類為 Major
    # ========================================
    t.step("PI 提交 → IACUC_STAFF 分類為 Major")
    t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/submit", role="PI")

    classify_major_resp = t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/classify",
                                  role="IACUC_STAFF", json={
                                      "amendment_type": "MAJOR",
                                      "remark": "需要正式審查"
                                  })
    major_classified_status = classify_major_resp.json()["status"]
    t.record("Major 分類 → CLASSIFIED",
             major_classified_status in ("CLASSIFIED", "classified"),
             f"status={major_classified_status}")

    # ========================================
    # Step 7: 驗證審查委員自動指派（後端自動複製）
    # ========================================
    t.step("驗證審查委員自動指派")
    assignments_resp = t._req("GET", f"{API_BASE_URL}/amendments/{major_id}/assignments",
                               role="IACUC_STAFF")
    assignments = assignments_resp.json()
    t.record("審查委員自動指派", len(assignments) >= 2,
             f"共 {len(assignments)} 位審查員")

    # ========================================
    # Step 8: IACUC_STAFF 開始審查 → UNDER_REVIEW
    # ========================================
    t.step("IACUC_STAFF 開始審查")
    review_resp = t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/start-review",
                          role="IACUC_STAFF")
    review_status = review_resp.json()["status"]
    t.record("開始審查 → UNDER_REVIEW",
             review_status in ("UNDER_REVIEW", "under_review"),
             f"status={review_status}")

    # ========================================
    # Step 9: 審查委員記錄決議（全部核准 → 最終 APPROVED）
    # ========================================
    t.step("審查委員記錄決議（全部核准）")

    # 找出已指派的審查委員 ID
    reviewer_roles = []
    for a in assignments:
        reviewer_id = a["reviewer_id"]
        # 找出對應的角色名稱
        for role_name, uid in t.user_ids.items():
            if uid == reviewer_id:
                reviewer_roles.append(role_name)
                break

    decision_count = 0
    for role_name in reviewer_roles:
        try:
            t._req("POST", f"{API_BASE_URL}/amendments/{major_id}/decision",
                    role=role_name, json={
                        "decision": "APPROVE",
                        "comment": f"同意變更 — by {role_name}"
                    })
            decision_count += 1
        except Exception as e:
            print(f"    ⚠ {role_name} 決議失敗: {e}")

    t.record("審查委員全部核准", decision_count == len(reviewer_roles),
             f"{decision_count}/{len(reviewer_roles)} 通過")

    # 驗證狀態更新為 APPROVED
    final_resp = t._req("GET", f"{API_BASE_URL}/amendments/{major_id}", role="IACUC_STAFF")
    final_status = final_resp.json()["status"]
    t.record("狀態更新 → APPROVED",
             final_status in ("APPROVED", "approved"),
             f"status={final_status}")

    # ========================================
    # Step 10: 驗證版本歷程
    # ========================================
    t.step("驗證版本歷程")
    versions_resp = t._req("GET", f"{API_BASE_URL}/amendments/{major_id}/versions",
                            role="IACUC_STAFF")
    versions = versions_resp.json()
    t.record("Major 版本歷程", len(versions) >= 1, f"共 {len(versions)} 個版本")

    # ========================================
    # Step 11: 驗證狀態歷程
    # ========================================
    t.step("驗證狀態歷程")
    history_resp = t._req("GET", f"{API_BASE_URL}/amendments/{major_id}/history",
                           role="IACUC_STAFF")
    history = history_resp.json()
    # 預期歷程：DRAFT→SUBMITTED→CLASSIFIED→UNDER_REVIEW→APPROVED
    history_statuses = [h.get("to_status", "") for h in history]
    t.record("狀態歷程正確",
             len(history) >= 4,
             f"共 {len(history)} 筆 {' → '.join(history_statuses[::-1])}")

    # ========================================
    # Step 12: 驗證 Protocol 的 amendments 列表
    # ========================================
    t.step("驗證 Protocol amendments 列表")
    proto_amendments_resp = t._req("GET", f"{API_BASE_URL}/protocols/{protocol_id}/amendments",
                                     role="IACUC_STAFF")
    proto_amendments = proto_amendments_resp.json()
    t.record("Protocol amendments 列表",
             len(proto_amendments) >= 2,
             f"共 {len(proto_amendments)} 個 amendment")

    # ========================================
    # Step 13: 驗證 Amendment 列表查詢（含篩選）
    # ========================================
    t.step("驗證 Amendment 列表查詢")
    all_resp = t._req("GET", f"{API_BASE_URL}/amendments", role="IACUC_STAFF")
    all_amendments = all_resp.json()
    t.record("Amendment 列表查詢", len(all_amendments) >= 2,
             f"共 {len(all_amendments)} 筆")

    # 依狀態篩選
    approved_resp = t._req("GET", f"{API_BASE_URL}/amendments?status=APPROVED",
                            role="IACUC_STAFF")
    approved_amendments = approved_resp.json()
    t.record("依狀態篩選（APPROVED）", len(approved_amendments) >= 1,
             f"共 {len(approved_amendments)} 筆")

    # ========================================
    # Step 14: 驗證待處理數量 API
    # ========================================
    t.step("驗證待處理數量 API")
    pending_resp = t._req("GET", f"{API_BASE_URL}/amendments/pending-count",
                           role="IACUC_STAFF")
    pending_data = pending_resp.json()
    t.record("待處理數量 API", "count" in pending_data,
             f"pending count = {pending_data.get('count', 'N/A')}")

    # ========================================
    # 彙總
    # ========================================
    print(f"\n{'=' * 60}")
    print(f"[完成] Amendment 完整流程完成！")
    print(f"  Protocol ID: {protocol_id}")
    print(f"  Minor Amendment ID: {minor_id} (ADMIN_APPROVED)")
    print(f"  Major Amendment ID: {major_id} (APPROVED)")
    print(f"{'=' * 60}")
    return t.print_summary()


if __name__ == "__main__":
    try:
        success = run_amendment_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n[CRITICAL ERROR] Amendment 測試失敗: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
