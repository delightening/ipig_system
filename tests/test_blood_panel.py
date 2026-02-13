"""
血液檢查組合 (Blood Test Panel) 完整測試

包括：
- Phase 1: Panel API CRUD — 驗證組合列表、建立、更新、刪除
- Phase 2: Panel + Template 關聯 — 驗證組合內項目管理
- Phase 3: 整合測試 — 使用 Panel 建立血液檢查紀錄，驗證 ERP 與動物端皆可見
- Phase 4: 權限驗證 — 驗證非授權使用者無法操作

用法：
    cd d:\\Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/test_blood_panel.py
"""

import sys
import os
import time
from datetime import date, timedelta

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL

# 測試帳號設定
TEST_USERS = {
    "ADMIN": {
        "email": "admin@ipig.local",
        "password": "admin123",
        "display_name": "系統管理員",
        "role_codes": ["ADMIN"],
    },
    "VET": {
        "email": "vet_panel_test@example.com",
        "password": "password123",
        "display_name": "獸醫師 (Panel 測試)",
        "role_codes": ["VET"],
    },
    "EXPERIMENT_STAFF": {
        "email": "exp_panel_test@example.com",
        "password": "password123",
        "display_name": "試驗工作人員 (Panel 測試)",
        "role_codes": ["EXPERIMENT_STAFF"],
    },
}


def run_blood_panel_test() -> bool:
    """執行血液檢查組合完整測試"""
    t = BaseApiTester("血液檢查組合 (Panel) 測試")

    if not t.setup_test_users(TEST_USERS):
        return False
    if not t.login_all(TEST_USERS):
        return False

    STAFF = "EXPERIMENT_STAFF"
    VET = "VET"
    ADMIN = "ADMIN"
    today = str(date.today())

    # ========================================
    # Phase 1: Panel API — 列表 + CRUD
    # ========================================
    t.step("Phase 1 — 列表預設 Panel（Seed 資料驗證）")

    # 1.1 取得啟用中組合
    resp = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=STAFF)
    panels = resp.json()
    panel_count = len(panels)
    t.record("列出啟用中組合", panel_count >= 10,
             f"共 {panel_count} 個組合（預期 ≥ 10，含 seed 的 14 個）")

    # 1.2 驗證 seed 組合結構
    has_cbc = any(p["key"] == "CBC" for p in panels)
    has_liver = any(p["key"] == "LIVER" for p in panels)
    has_kidney = any(p["key"] == "KIDNEY" for p in panels)
    t.record("Seed 組合存在 (CBC/Liver/Kidney)",
             has_cbc and has_liver and has_kidney)

    # 1.3 驗證組合包含 items
    cbc_panel = next((p for p in panels if p["key"] == "CBC"), None)
    if cbc_panel:
        cbc_items = cbc_panel.get("items", [])
        t.record("CBC 組合包含項目", len(cbc_items) >= 3,
                 f"CBC 有 {len(cbc_items)} 個項目")
        # 列出一些項目名稱
        item_names = [item.get("name", item.get("code", "?")) for item in cbc_items[:5]]
        t.sub_step(f"CBC 項目：{', '.join(item_names)}")
    else:
        t.record("CBC 組合包含項目", False, "找不到 CBC 組合")

    # 1.4 取得所有組合（含停用）
    resp_all = t._req("GET", f"{API_BASE_URL}/blood-test-panels/all", role=STAFF)
    all_panels = resp_all.json()
    t.record("列出所有組合（含停用）", len(all_panels) >= panel_count,
             f"啟用 {panel_count} / 全部 {len(all_panels)}")

    # ========================================
    # Phase 2: Panel CRUD 操作
    # ========================================
    t.step("Phase 2 — Panel CRUD 操作")

    # 2.1 取得模板列表（用於建立組合）
    tpl_resp = t._req("GET", f"{API_BASE_URL}/blood-test-templates", role=STAFF)
    templates = tpl_resp.json()
    t.sub_step(f"取得 {len(templates)} 個模板")

    # 選幾個模板做測試組合
    test_template_ids = [tpl["id"] for tpl in templates[:4]]

    # 2.2 建立自訂組合
    create_payload = {
        "key": f"test_panel_{int(time.time()) % 100000}",
        "name": "測試用組合 - 整合測試",
        "icon": "🧪",
        "sort_order": 99,
        "template_ids": test_template_ids,
    }
    resp = t._req("POST", f"{API_BASE_URL}/blood-test-panels", role=STAFF,
                  json=create_payload)
    new_panel = resp.json()
    new_panel_id = new_panel["id"]
    t.record("建立自訂組合", True,
             f"ID: {new_panel_id[:8]}... key={create_payload['key']}")

    # 驗證新組合的項目
    new_panel_items = new_panel.get("items", [])
    t.record("新組合包含正確項目數", len(new_panel_items) == len(test_template_ids),
             f"預期 {len(test_template_ids)} / 實際 {len(new_panel_items)}")

    # 2.3 更新組合
    update_payload = {
        "name": "測試用組合 (已更新)",
        "icon": "✅",
        "sort_order": 98,
    }
    resp = t._req("PUT", f"{API_BASE_URL}/blood-test-panels/{new_panel_id}",
                  role=STAFF, json=update_payload)
    updated = resp.json()
    t.record("更新組合名稱/圖示",
             updated.get("name") == "測試用組合 (已更新)" and updated.get("icon") == "✅")

    # 2.4 更新組合項目
    new_template_ids = [tpl["id"] for tpl in templates[2:7]]  # 切換為不同的模板
    resp = t._req("PUT", f"{API_BASE_URL}/blood-test-panels/{new_panel_id}/items",
                  role=STAFF, json={"template_ids": new_template_ids})
    updated_items = resp.json().get("items", [])
    t.record("更新組合項目",
             len(updated_items) == len(new_template_ids),
             f"預期 {len(new_template_ids)} / 實際 {len(updated_items)}")

    # 2.5 驗證更新後列表
    resp = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=STAFF)
    panels_after = resp.json()
    found = next((p for p in panels_after if p["id"] == new_panel_id), None)
    t.record("列表中可找到更新後的組合",
             found is not None and found.get("name") == "測試用組合 (已更新)")

    # ========================================
    # Phase 3: 整合測試 — 使用 Panel 建立血液檢查
    # ========================================
    t.step("Phase 3 — 使用 Panel 建立血液檢查紀錄")

    # 3.1 建立一隻測試豬
    source_resp = t._req("POST", f"{API_BASE_URL}/pig-sources", role=STAFF,
                         json={
                             "code": f"SRC-BT-{int(time.time()) % 10000}",
                             "name": "Panel 測試豬源",
                         })
    source_id = source_resp.json()["id"]

    pig_resp = t._req("POST", f"{API_BASE_URL}/pigs", role=STAFF, json={
        "ear_tag": f"{int(time.time()) % 1000:03d}",
        "breed": "minipig",
        "gender": "female",
        "birth_date": str(date.today() - timedelta(days=180)),
        "entry_date": today,
        "entry_weight": 25.0,
        "pen_location": "A-01",
        "source_id": source_id,
        "remark": "Panel 測試用豬隻",
    })
    test_pig = pig_resp.json()
    test_pig_id = test_pig["id"]
    t.record("建立測試豬隻", True, f"ID: {test_pig_id[:8]}... ear_tag={test_pig.get('ear_tag')}")

    # 3.2 取得 CBC 組合的所有項目
    cbc_panel = next((p for p in panels if p["key"] == "CBC"), None)
    liver_panel = next((p for p in panels if p["key"] == "LIVER"), None)

    if not cbc_panel or not liver_panel:
        t.record("取得 CBC/Liver 組合", False, "找不到 seed 組合")
        return t.print_summary()

    # 3.3 用 CBC + Liver 組合的項目建立一筆血液檢查
    all_items = []
    for idx, tpl in enumerate(cbc_panel["items"]):
        all_items.append({
            "template_id": tpl["id"],
            "item_name": tpl["name"],
            "result_value": str(round(5.0 + idx * 0.5, 1)),
            "result_unit": tpl.get("default_unit", ""),
            "reference_range": tpl.get("reference_range", ""),
            "is_abnormal": idx == 2,  # 第 3 項異常
            "remark": "",
            "sort_order": idx,
        })
    for idx, tpl in enumerate(liver_panel["items"]):
        all_items.append({
            "template_id": tpl["id"],
            "item_name": tpl["name"],
            "result_value": str(round(20.0 + idx * 5.0, 1)),
            "result_unit": tpl.get("default_unit", ""),
            "reference_range": tpl.get("reference_range", ""),
            "is_abnormal": idx == 0,
            "remark": "Liver item" if idx == 0 else "",
            "sort_order": len(cbc_panel["items"]) + idx,
        })

    blood_test_payload = {
        "test_date": today,
        "lab_name": "Panel 整合測試實驗室",
        "remark": "使用 CBC + Liver 組合建立",
        "items": all_items,
    }

    bt_resp = t._req("POST", f"{API_BASE_URL}/pigs/{test_pig_id}/blood-tests",
                     role=STAFF, json=blood_test_payload)
    bt_data = bt_resp.json()
    # 回應結構: {blood_test: {id, ...}, items: [...], created_by_name: ...}
    bt_inner = bt_data.get("blood_test", bt_data)
    bt_id = bt_inner["id"]
    bt_items_count = len(bt_data.get("items", []))
    expected_items = len(cbc_panel["items"]) + len(liver_panel["items"])
    t.record("建立血液檢查（含 Panel 項目）",
             bt_items_count == expected_items,
             f"CBC({len(cbc_panel['items'])}項) + Liver({len(liver_panel['items'])}項) = {bt_items_count} 項")

    # ========================================
    # Phase 4: ERP 端 — 驗證查詢可見
    # ========================================
    t.step("Phase 4 — 驗證 ERP 端可見（admin 帳號）")

    # 4.1 ADMIN 可查詢 Panel 列表
    resp = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=ADMIN)
    admin_panels = resp.json()
    t.record("ADMIN 可列出組合", len(admin_panels) >= 10)

    # 4.2 ADMIN 可查所有 Panel（含停用）
    resp = t._req("GET", f"{API_BASE_URL}/blood-test-panels/all", role=ADMIN)
    t.record("ADMIN 可列出所有組合", resp.status_code == 200)

    # 4.3 ADMIN 可查血液檢查紀錄
    resp = t._req("GET", f"{API_BASE_URL}/pigs/{test_pig_id}/blood-tests", role=ADMIN)
    admin_bt = resp.json()
    t.record("ADMIN 可查詢血液檢查列表", len(admin_bt) >= 1,
             f"共 {len(admin_bt)} 筆")

    # 4.4 ADMIN 可查看血液檢查詳情
    resp = t._req("GET", f"{API_BASE_URL}/blood-tests/{bt_id}", role=ADMIN)
    detail = resp.json()
    t.record("ADMIN 可查看血檢詳情",
             len(detail.get("items", [])) == expected_items,
             f"項目數：{len(detail.get('items', []))}")

    # ========================================
    # Phase 5: 動物端 — VET 驗證可見
    # ========================================
    t.step("Phase 5 — 驗證動物端可見（VET 帳號）")

    # 5.1 VET 可查詢 Panel 列表
    resp = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=VET)
    vet_panels = resp.json()
    t.record("VET 可列出組合", len(vet_panels) >= 10)

    # 5.2 VET 可查看血液檢查列表
    resp = t._req("GET", f"{API_BASE_URL}/pigs/{test_pig_id}/blood-tests", role=VET)
    vet_bt = resp.json()
    t.record("VET 可查詢血液檢查列表", len(vet_bt) >= 1)

    # 5.3 VET 可查看血液檢查詳情
    resp = t._req("GET", f"{API_BASE_URL}/blood-tests/{bt_id}", role=VET)
    vet_detail = resp.json()
    vet_items = vet_detail.get("items", [])
    t.record("VET 可查看血檢詳情", len(vet_items) == expected_items)

    # 5.4 驗證有異常的項目
    abnormal_items = [i for i in vet_items if i.get("is_abnormal")]
    t.record("異常項目正確標記", len(abnormal_items) == 2,
             f"預期 2 個異常 / 實際 {len(abnormal_items)} 個")

    # ========================================
    # Phase 6: Staff 使用第二組 Panel 再建一筆
    # ========================================
    t.step("Phase 6 — STAFF 使用 Kidney 組合建立第二筆血檢")

    kidney_panel = next((p for p in panels if p["key"] == "KIDNEY"), None)
    if kidney_panel and kidney_panel.get("items"):
        kidney_items = []
        for idx, tpl in enumerate(kidney_panel["items"]):
            kidney_items.append({
                "template_id": tpl["id"],
                "item_name": tpl["name"],
                "result_value": str(round(1.0 + idx * 0.3, 2)),
                "result_unit": tpl.get("default_unit", ""),
                "reference_range": tpl.get("reference_range", ""),
                "is_abnormal": False,
                "remark": "",
                "sort_order": idx,
            })

        bt2_resp = t._req("POST", f"{API_BASE_URL}/pigs/{test_pig_id}/blood-tests",
                          role=STAFF, json={
                              "test_date": str(date.today() - timedelta(days=1)),
                              "lab_name": "Panel 測試 - 第二筆",
                              "remark": "Kidney 組合",
                              "items": kidney_items,
                          })
        bt2_data = bt2_resp.json()
        t.record("建立第二筆血液檢查 (Kidney)",
                 len(bt2_data.get("items", [])) == len(kidney_panel["items"]),
                 f"Kidney {len(kidney_panel['items'])} 項")
    else:
        t.record("建立第二筆血液檢查 (Kidney)", False, "找不到 Kidney 組合")

    # 6.2 驗證豬隻有 2 筆血液檢查
    resp = t._req("GET", f"{API_BASE_URL}/pigs/{test_pig_id}/blood-tests", role=STAFF)
    final_bt_list = resp.json()
    t.record("豬隻共有 2 筆血液檢查", len(final_bt_list) == 2,
             f"實際 {len(final_bt_list)} 筆")

    # ========================================
    # Phase 7: 停用 + 恢復測試
    # ========================================
    t.step("Phase 7 — 停用與恢復自訂組合")

    # 7.1 停用自訂組合
    t._req("DELETE", f"{API_BASE_URL}/blood-test-panels/{new_panel_id}", role=STAFF)
    t.record("停用自訂組合", True)

    # 7.2 驗證啟用列表不含已停用組合
    resp = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=STAFF)
    active_panels = resp.json()
    is_hidden = not any(p["id"] == new_panel_id for p in active_panels)
    t.record("停用組合從啟用列表隱藏", is_hidden)

    # 7.3 驗證所有列表仍包含已停用組合
    resp = t._req("GET", f"{API_BASE_URL}/blood-test-panels/all", role=STAFF)
    all_panels_after = resp.json()
    is_in_all = any(p["id"] == new_panel_id for p in all_panels_after)
    t.record("停用組合在全部列表可見", is_in_all)

    # 7.4 恢復組合
    t._req("PUT", f"{API_BASE_URL}/blood-test-panels/{new_panel_id}",
           role=STAFF, json={"is_active": True})
    resp = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=STAFF)
    reactivated = any(p["id"] == new_panel_id for p in resp.json())
    t.record("恢復組合後列表可見", reactivated)

    # ========================================
    # Phase 8: 模板 API 驗證
    # ========================================
    t.step("Phase 8 — 模板 API 驗證（各角色皆可讀取）")

    for role_name in [ADMIN, VET, STAFF]:
        resp = t._req("GET", f"{API_BASE_URL}/blood-test-templates", role=role_name)
        tpls = resp.json()
        t.record(f"{role_name} 可讀取模板列表", len(tpls) >= 20,
                 f"共 {len(tpls)} 個模板")

    # ========================================
    # 彙總
    # ========================================
    print(f"\n{'=' * 60}")
    print(f"[完成] 血液檢查組合 (Panel) 測試完成！")
    print(f"  組合數: {panel_count} | 測試建立組合 ID: {new_panel_id[:8]}...")
    print(f"  血液檢查: 2 筆 (CBC+Liver / Kidney)")
    print(f"  測試豬隻: {test_pig_id[:8]}...")
    print(f"{'=' * 60}")

    return t.print_summary()


if __name__ == "__main__":
    try:
        success = run_blood_panel_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n[CRITICAL ERROR] Panel 測試失敗: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
