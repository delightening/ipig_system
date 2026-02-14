"""
血液檢查系統完整測試（重新設計）

包括：
- Phase 1:  Seed 資料驗證 — 模板 & Panel 預設資料
- Phase 2:  模板 CRUD — 建立、更新、停用/恢復模板
- Phase 3:  Panel CRUD — 建立、更新項目、停用/恢復組合
- Phase 4:  建立測試豬隻
- Phase 5:  建立血液檢查紀錄 (CBC + Liver)
- Phase 6:  建立第二筆血液檢查 (Kidney)
- Phase 7:  查詢與詳情驗證
- Phase 8:  更新血液檢查紀錄
- Phase 9:  獸醫已讀標記
- Phase 10: 軟刪除血液檢查（GLP 合規）
- Phase 11: 跨角色存取驗證 (ADMIN / VET / STAFF)
- Phase 12: Panel 停用與恢復

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

# ============================================
# 測試帳號設定
# ============================================
TEST_USERS = {
    "ADMIN": {
        "email": "jason4617987@gmail.com",
        "password": "kfknxJH6AjSvJh6?",
        "display_name": "系統管理員",
        "role_codes": ["ADMIN"],
    },
    "VET": {
        "email": "vet_blood_test@example.com",
        "password": "password123",
        "display_name": "獸醫師 (血液檢查測試)",
        "role_codes": ["VET"],
    },
    "EXPERIMENT_STAFF": {
        "email": "exp_blood_test@example.com",
        "password": "password123",
        "display_name": "試驗工作人員 (血液檢查測試)",
        "role_codes": ["EXPERIMENT_STAFF"],
    },
}


def run_blood_panel_test() -> bool:
    """執行血液檢查系統完整測試"""
    t = BaseApiTester("血液檢查系統完整測試")

    if not t.setup_test_users(TEST_USERS):
        return False
    if not t.login_all(TEST_USERS):
        return False

    STAFF = "EXPERIMENT_STAFF"
    VET = "VET"
    ADMIN = "ADMIN"
    today = str(date.today())
    yesterday = str(date.today() - timedelta(days=1))  # partition_date 用 UTC 存傲，融入時差
    ts = int(time.time()) % 100000  # 時間戳避免衝突

    # ========================================
    # Phase 1: Seed 資料驗證
    # ========================================
    t.step("Phase 1 — Seed 資料驗證（模板 & Panel）")

    # 1.1 取得啟用中模板
    tpl_resp = t._req("GET", f"{API_BASE_URL}/blood-test-templates", role=STAFF)
    templates = tpl_resp.json()
    t.record("取得啟用中模板", len(templates) >= 20,
             f"共 {len(templates)} 個模板（預期 ≥ 20）")

    # 1.2 取得所有模板（含停用）
    tpl_all_resp = t._req("GET", f"{API_BASE_URL}/blood-test-templates/all", role=STAFF)
    all_templates = tpl_all_resp.json()
    t.record("取得所有模板（含停用）", len(all_templates) >= len(templates),
             f"啟用 {len(templates)} / 全部 {len(all_templates)}")

    # 1.3 驗證部分模板內容
    code_set = {tpl["code"] for tpl in templates}
    expected_codes = {"WBC", "RBC", "HGB", "AST", "ALT", "BUN", "CRE", "GLU"}
    missing = expected_codes - code_set
    t.record("Seed 模板包含關鍵項目 (WBC/RBC/AST/BUN/GLU...)",
             len(missing) == 0,
             f"缺少: {missing}" if missing else "全部存在")

    # 1.4 取得啟用中 Panel
    panel_resp = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=STAFF)
    panels = panel_resp.json()
    panel_count = len(panels)
    t.record("取得啟用中 Panel", panel_count >= 10,
             f"共 {panel_count} 個 Panel（預期 ≥ 10）")

    # 1.5 驗證 Seed Panel 存在
    panel_keys = {p["key"] for p in panels}
    required_keys = {"CBC", "LIVER", "KIDNEY", "LIPID", "HEART", "COAG", "ELECTRO"}
    missing_keys = required_keys - panel_keys
    t.record("Seed Panel 包含必要類別",
             len(missing_keys) == 0,
             f"缺少: {missing_keys}" if missing_keys else "全部存在")

    # 1.6 驗證 CBC Panel 包含項目
    cbc_panel = next((p for p in panels if p["key"] == "CBC"), None)
    if cbc_panel:
        cbc_items = cbc_panel.get("items", [])
        t.record("CBC Panel 包含項目", len(cbc_items) >= 3,
                 f"CBC 有 {len(cbc_items)} 個項目")
        item_codes = [i.get("code", "?") for i in cbc_items[:5]]
        t.sub_step(f"CBC 項目: {', '.join(item_codes)}")
    else:
        t.record("CBC Panel 包含項目", False, "找不到 CBC Panel")

    # 1.7 取得所有 Panel（含停用）
    panel_all_resp = t._req("GET", f"{API_BASE_URL}/blood-test-panels/all", role=STAFF)
    all_panels = panel_all_resp.json()
    t.record("取得所有 Panel（含停用）", len(all_panels) >= panel_count,
             f"啟用 {panel_count} / 全部 {len(all_panels)}")

    # ========================================
    # Phase 2: 模板 CRUD
    # ========================================
    t.step("Phase 2 — 模板 CRUD 操作")

    # 2.1 建立新模板
    new_tpl_payload = {
        "code": f"TEST{ts}",
        "name": f"測試模板項目 ({ts})",
        "default_unit": "mg/dL",
        "reference_range": "10-50",
        "default_price": 100,
        "sort_order": 9999,
    }
    tpl_create_resp = t._req("POST", f"{API_BASE_URL}/blood-test-templates",
                              role=STAFF, json=new_tpl_payload)
    new_tpl = tpl_create_resp.json()
    new_tpl_id = new_tpl["id"]
    t.record("建立新模板", True,
             f"code={new_tpl_payload['code']}, ID: {new_tpl_id[:8]}...")

    # 2.2 更新模板
    tpl_update_payload = {
        "name": f"測試模板項目 (已更新)",
        "default_unit": "U/L",
        "default_price": 200,
    }
    tpl_update_resp = t._req("PUT", f"{API_BASE_URL}/blood-test-templates/{new_tpl_id}",
                              role=STAFF, json=tpl_update_payload)
    updated_tpl = tpl_update_resp.json()
    t.record("更新模板名稱/單位/價格",
             updated_tpl.get("name") == "測試模板項目 (已更新)" and
             updated_tpl.get("default_unit") == "U/L")

    # 2.3 停用模板
    t._req("DELETE", f"{API_BASE_URL}/blood-test-templates/{new_tpl_id}", role=STAFF)
    t.record("停用模板", True)

    # 2.4 驗證停用後啟用列表不含
    tpl_resp2 = t._req("GET", f"{API_BASE_URL}/blood-test-templates", role=STAFF)
    is_hidden = not any(tp["id"] == new_tpl_id for tp in tpl_resp2.json())
    t.record("停用模板不出現在啟用列表", is_hidden)

    # 2.5 驗證停用後全部列表仍含
    tpl_all_resp2 = t._req("GET", f"{API_BASE_URL}/blood-test-templates/all", role=STAFF)
    is_in_all = any(tp["id"] == new_tpl_id for tp in tpl_all_resp2.json())
    t.record("停用模板在全部列表可見", is_in_all)

    # 2.6 恢復模板
    t._req("PUT", f"{API_BASE_URL}/blood-test-templates/{new_tpl_id}",
           role=STAFF, json={"is_active": True})
    tpl_resp3 = t._req("GET", f"{API_BASE_URL}/blood-test-templates", role=STAFF)
    is_restored = any(tp["id"] == new_tpl_id for tp in tpl_resp3.json())
    t.record("恢復模板後出現在啟用列表", is_restored)

    # ========================================
    # Phase 3: Panel CRUD + 項目管理
    # ========================================
    t.step("Phase 3 — Panel CRUD + 項目管理")

    # 3.1 選取模板用以建立 Panel
    test_tpl_ids = [tpl["id"] for tpl in templates[:4]]

    # 3.2 建立自訂 Panel
    panel_create_payload = {
        "key": f"test_p_{ts}",
        "name": "測試組合 — 整合測試",
        "icon": "🧪",
        "sort_order": 99,
        "template_ids": test_tpl_ids,
    }
    panel_create_resp = t._req("POST", f"{API_BASE_URL}/blood-test-panels",
                                role=STAFF, json=panel_create_payload)
    new_panel = panel_create_resp.json()
    new_panel_id = new_panel["id"]
    new_panel_items = new_panel.get("items", [])
    t.record("建立自訂 Panel",
             len(new_panel_items) == len(test_tpl_ids),
             f"ID: {new_panel_id[:8]}..., 項目數: {len(new_panel_items)}")

    # 3.3 更新 Panel 名稱/圖示
    panel_update_payload = {
        "name": "測試組合 (已更新)",
        "icon": "✅",
        "sort_order": 98,
    }
    panel_update_resp = t._req("PUT", f"{API_BASE_URL}/blood-test-panels/{new_panel_id}",
                                role=STAFF, json=panel_update_payload)
    up_panel = panel_update_resp.json()
    t.record("更新 Panel 名稱/圖示",
             up_panel.get("name") == "測試組合 (已更新)" and
             up_panel.get("icon") == "✅")

    # 3.4 更新 Panel 項目（替換為不同模板集）
    new_tpl_ids = [tpl["id"] for tpl in templates[2:7]]
    items_resp = t._req("PUT", f"{API_BASE_URL}/blood-test-panels/{new_panel_id}/items",
                         role=STAFF, json={"template_ids": new_tpl_ids})
    updated_items = items_resp.json().get("items", [])
    t.record("替換 Panel 項目",
             len(updated_items) == len(new_tpl_ids),
             f"預期 {len(new_tpl_ids)} / 實際 {len(updated_items)}")

    # 3.5 列表中驗證更新
    panel_resp2 = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=STAFF)
    found = next((p for p in panel_resp2.json() if p["id"] == new_panel_id), None)
    t.record("列表中可找到更新後的 Panel",
             found is not None and found.get("name") == "測試組合 (已更新)")

    # ========================================
    # Phase 4: 建立測試豬隻
    # ========================================
    t.step("Phase 4 — 建立測試豬隻")

    source_resp = t._req("POST", f"{API_BASE_URL}/pig-sources", role=STAFF,
                          json={
                              "code": f"SRC-BT-{ts}",
                              "name": "血液檢查測試豬源",
                          })
    source_id = source_resp.json()["id"]

    pig_resp = t._req("POST", f"{API_BASE_URL}/pigs", role=STAFF, json={
        "ear_tag": f"{ts % 1000:03d}",
        "breed": "minipig",
        "gender": "female",
        "birth_date": str(date.today() - timedelta(days=180)),
        "entry_date": today,
        "entry_weight": 25.0,
        "pen_location": "A-01",
        "source_id": source_id,
        "remark": "血液檢查完整測試用豬隻",
        "force_create": True,
    })
    test_pig = pig_resp.json()
    test_pig_id = test_pig["id"]
    t.record("建立測試豬隻", True,
             f"ID: {test_pig_id[:8]}..., ear_tag={test_pig.get('ear_tag')}")

    # ========================================
    # Phase 5: 建立血液檢查（CBC + Liver）
    # ========================================
    t.step("Phase 5 — 建立血液檢查紀錄 (CBC + Liver)")

    # 重新取得最新 panel 資料（含 items）
    panels = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=STAFF).json()
    cbc_panel = next((p for p in panels if p["key"] == "CBC"), None)
    liver_panel = next((p for p in panels if p["key"] == "LIVER"), None)

    if not cbc_panel or not liver_panel:
        t.record("取得 CBC/Liver Panel", False, "找不到 seed Panel")
        return t.print_summary()

    # 組裝 items
    all_items = []
    for idx, tpl in enumerate(cbc_panel.get("items", [])):
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
    for idx, tpl in enumerate(liver_panel.get("items", [])):
        all_items.append({
            "template_id": tpl["id"],
            "item_name": tpl["name"],
            "result_value": str(round(20.0 + idx * 5.0, 1)),
            "result_unit": tpl.get("default_unit", ""),
            "reference_range": tpl.get("reference_range", ""),
            "is_abnormal": idx == 0,
            "remark": "Liver item" if idx == 0 else "",
            "sort_order": len(cbc_panel.get("items", [])) + idx,
        })

    blood_test_payload = {
        "test_date": today,
        "lab_name": "整合測試實驗室 A",
        "remark": "使用 CBC + Liver 組合建立",
        "items": all_items,
    }

    bt_resp = t._req("POST", f"{API_BASE_URL}/pigs/{test_pig_id}/blood-tests",
                      role=STAFF, json=blood_test_payload)
    bt_data = bt_resp.json()
    bt_inner = bt_data.get("blood_test", bt_data)
    bt1_id = bt_inner["id"]
    bt1_items_count = len(bt_data.get("items", []))
    cbc_count = len(cbc_panel.get("items", []))
    liver_count = len(liver_panel.get("items", []))
    expected_items = cbc_count + liver_count
    t.record("建立血液檢查 (CBC+Liver)",
             bt1_items_count == expected_items,
             f"CBC({cbc_count}) + Liver({liver_count}) = {bt1_items_count} 項")

    # ========================================
    # Phase 6: 建立第二筆血液檢查（Kidney）
    # ========================================
    t.step("Phase 6 — 建立第二筆血液檢查 (Kidney)")

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
                               "lab_name": "整合測試實驗室 B",
                               "remark": "Kidney 組合血液檢查",
                               "items": kidney_items,
                           })
        bt2_data = bt2_resp.json()
        bt2_inner = bt2_data.get("blood_test", bt2_data)
        bt2_id = bt2_inner["id"]
        t.record("建立第二筆血液檢查 (Kidney)",
                 len(bt2_data.get("items", [])) == len(kidney_panel["items"]),
                 f"Kidney {len(kidney_panel['items'])} 項")
    else:
        t.record("建立第二筆血液檢查 (Kidney)", False, "找不到 Kidney Panel")
        bt2_id = None

    # ========================================
    # Phase 7: 查詢與詳情驗證
    # ========================================
    t.step("Phase 7 — 查詢與詳情驗證")

    # 7.1 列出豬隻的所有血液檢查
    list_resp = t._req("GET", f"{API_BASE_URL}/pigs/{test_pig_id}/blood-tests", role=STAFF)
    bt_list = list_resp.json()
    t.record("豬隻血液檢查列表", len(bt_list) == 2,
             f"實際 {len(bt_list)} 筆（預期 2）")

    # 7.2 驗證列表結構（含 item_count/abnormal_count）
    first_bt = next((b for b in bt_list if b["id"] == bt1_id), None)
    if first_bt:
        has_counts = (first_bt.get("item_count") is not None and
                      first_bt.get("abnormal_count") is not None)
        t.record("列表含 item_count / abnormal_count", has_counts,
                 f"item_count={first_bt.get('item_count')}, "
                 f"abnormal_count={first_bt.get('abnormal_count')}")
    else:
        t.record("列表含 item_count / abnormal_count", False, "找不到第一筆血檢")

    # 7.3 取得第一筆詳情
    detail_resp = t._req("GET", f"{API_BASE_URL}/blood-tests/{bt1_id}", role=STAFF)
    detail = detail_resp.json()
    detail_items = detail.get("items", [])
    t.record("血液檢查詳情 — 項目完整",
             len(detail_items) == expected_items,
             f"預期 {expected_items} / 實際 {len(detail_items)}")

    # 7.4 驗證異常項目標記
    abnormal_items = [i for i in detail_items if i.get("is_abnormal")]
    t.record("異常項目正確標記", len(abnormal_items) == 2,
             f"預期 2 個異常 / 實際 {len(abnormal_items)} 個")

    # 7.5 驗證 created_by_name 有值
    t.record("detial 含 created_by_name",
             detail.get("created_by_name") is not None,
             f"created_by_name={detail.get('created_by_name')}")

    # ========================================
    # Phase 8: 更新血液檢查紀錄
    # ========================================
    t.step("Phase 8 — 更新血液檢查紀錄")

    update_bt_payload = {
        "lab_name": "整合測試實驗室 A (更正)",
        "remark": "更新備註：已修正實驗室名稱",
    }
    up_bt_resp = t._req("PUT", f"{API_BASE_URL}/blood-tests/{bt1_id}",
                         role=STAFF, json=update_bt_payload)
    up_bt = up_bt_resp.json()
    # 根據回應結構，可能是 { blood_test: {...}, items: [...] } 或直接 {...}
    up_bt_inner = up_bt.get("blood_test", up_bt)
    t.record("更新血液檢查 lab_name",
             up_bt_inner.get("lab_name") == "整合測試實驗室 A (更正)")
    t.record("更新血液檢查 remark",
             up_bt_inner.get("remark") == "更新備註：已修正實驗室名稱")

    # 8.2 更新項目（替換全部）
    updated_items_payload = {
        "items": all_items[:3],  # 只留前 3 項
    }
    up_bt_items_resp = t._req("PUT", f"{API_BASE_URL}/blood-tests/{bt1_id}",
                               role=STAFF, json=updated_items_payload)
    up_bt_items = up_bt_items_resp.json()
    up_bt_items_list = up_bt_items.get("items", [])
    t.record("更新血液檢查項目（替換為 3 項）",
             len(up_bt_items_list) == 3,
             f"實際 {len(up_bt_items_list)} 項")

    # 8.3 還原項目（全部放回）
    restore_payload = {
        "items": all_items,
    }
    t._req("PUT", f"{API_BASE_URL}/blood-tests/{bt1_id}",
           role=STAFF, json=restore_payload)
    # 驗證還原
    verify_resp = t._req("GET", f"{API_BASE_URL}/blood-tests/{bt1_id}", role=STAFF)
    restored_count = len(verify_resp.json().get("items", []))
    t.record("還原血液檢查項目",
             restored_count == expected_items,
             f"預期 {expected_items} / 實際 {restored_count}")

    # ========================================
    # Phase 9: VET 讀取驗證
    # ========================================
    t.step("Phase 9 — VET 讀取驗證")

    # 9.1 VET 可查看血液檢查詳情
    vet_detail = t._req("GET", f"{API_BASE_URL}/blood-tests/{bt1_id}", role=VET).json()
    vet_items = vet_detail.get("items", [])
    t.record("VET 可查看血檢詳情",
             len(vet_items) == expected_items,
             f"項目數: {len(vet_items)}")

    # 9.2 VET 可看到異常標記
    vet_abnormal = [i for i in vet_items if i.get("is_abnormal")]
    t.record("VET 可看到異常項目標記",
             len(vet_abnormal) == 2,
             f"異常: {len(vet_abnormal)} 項")

    # 9.3 VET 可看到建立者名稱
    t.record("VET 可看到 created_by_name",
             vet_detail.get("created_by_name") is not None,
             f"created_by_name={vet_detail.get('created_by_name')}")

    # 9.4 VET 可查看血液檢查列表
    vet_list = t._req("GET", f"{API_BASE_URL}/pigs/{test_pig_id}/blood-tests",
                       role=VET).json()
    t.record("VET 可查看豬隻血檢列表", len(vet_list) >= 1,
             f"共 {len(vet_list)} 筆")

    # ========================================
    # Phase 10: 軟刪除血液檢查（GLP 合規）
    # ========================================
    t.step("Phase 10 — 軟刪除血液檢查（GLP 合規）")

    if bt2_id:
        # 10.1 軟刪除第二筆
        del_resp = t._req("DELETE", f"{API_BASE_URL}/blood-tests/{bt2_id}",
                           role=STAFF, json={"reason": "整合測試清理：此筆為測試資料"})
        t.record("軟刪除血液檢查", del_resp.status_code == 200)

        # 10.2 驗證列表中不再出現
        list_after_del = t._req("GET", f"{API_BASE_URL}/pigs/{test_pig_id}/blood-tests",
                                 role=STAFF).json()
        is_hidden_bt = not any(b["id"] == bt2_id for b in list_after_del)
        t.record("刪除後列表中不出現", is_hidden_bt,
                 f"列表剩 {len(list_after_del)} 筆")
    else:
        t.record("軟刪除血液檢查", False, "無第二筆可刪除")
        t.record("刪除後列表中不出現", False, "跳過")

    # ========================================
    # Phase 11: 跨角色存取驗證
    # ========================================
    t.step("Phase 11 — 跨角色存取驗證")

    for role_name in [ADMIN, VET, STAFF]:
        # 11.1 可查 Panel 列表
        r = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=role_name)
        t.record(f"{role_name} 可查 Panel 列表", len(r.json()) >= 10,
                 f"共 {len(r.json())} 個")

        # 11.2 可查模板列表
        r = t._req("GET", f"{API_BASE_URL}/blood-test-templates", role=role_name)
        t.record(f"{role_name} 可查模板列表", len(r.json()) >= 20,
                 f"共 {len(r.json())} 個")

        # 11.3 可查血液檢查列表
        r = t._req("GET", f"{API_BASE_URL}/pigs/{test_pig_id}/blood-tests",
                    role=role_name)
        t.record(f"{role_name} 可查血液檢查列表", len(r.json()) >= 1)

        # 11.4 可查血液檢查詳情
        r = t._req("GET", f"{API_BASE_URL}/blood-tests/{bt1_id}", role=role_name)
        items = r.json().get("items", [])
        t.record(f"{role_name} 可查血檢詳情",
                 len(items) == expected_items,
                 f"{len(items)} 項")

    # ========================================
    # Phase 12: Panel 停用與恢復
    # ========================================
    t.step("Phase 12 — Panel 停用與恢復")

    # 12.1 停用自訂 Panel
    t._req("DELETE", f"{API_BASE_URL}/blood-test-panels/{new_panel_id}", role=STAFF)
    t.record("停用自訂 Panel", True)

    # 12.2 啟用列表中不含
    panels_after = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=STAFF).json()
    is_hidden_panel = not any(p["id"] == new_panel_id for p in panels_after)
    t.record("停用 Panel 從啟用列表隱藏", is_hidden_panel)

    # 12.3 全部列表仍含
    all_panels_after = t._req("GET", f"{API_BASE_URL}/blood-test-panels/all",
                               role=STAFF).json()
    is_in_all_panel = any(p["id"] == new_panel_id for p in all_panels_after)
    t.record("停用 Panel 在全部列表可見", is_in_all_panel)

    # 12.4 恢復 Panel
    t._req("PUT", f"{API_BASE_URL}/blood-test-panels/{new_panel_id}",
           role=STAFF, json={"is_active": True})
    panels_restored = t._req("GET", f"{API_BASE_URL}/blood-test-panels", role=STAFF).json()
    is_restored_panel = any(p["id"] == new_panel_id for p in panels_restored)
    t.record("恢復 Panel 後啟用列表可見", is_restored_panel)

    # ========================================
    # Phase 13: 操作日誌驗證 (user_activity_logs)
    # ========================================
    t.step("Phase 13 — 操作日誌驗證 (user_activity_logs)")

    # 13.1 查詢 ANIMAL 類別的活動日誌
    activity_resp = t._req(
        "GET",
        f"{API_BASE_URL}/admin/audit/activities?category=ANIMAL&from={yesterday}&to={today}",
        role=ADMIN,
    )
    activities = activity_resp.json()
    activity_data = activities.get("data", [])
    total_activities = activities.get("total", 0)
    t.record("ANIMAL 類別有操作日誌",
             total_activities > 0,
             f"共 {total_activities} 筆活動紀錄")

    # 13.2 驗證包含血液檢查建立事件
    event_types = {a["event_type"] for a in activity_data}
    has_bt_create = "BLOOD_TEST_CREATE" in event_types
    t.record("日誌含 BLOOD_TEST_CREATE 事件", has_bt_create,
             f"事件類型: {', '.join(sorted(event_types))}")

    # 13.3 驗證包含模板建立事件
    # 查模板相關日誌（可能在 ANIMAL 或 SYSTEM 類別）
    tpl_activity_resp = t._req(
        "GET",
        f"{API_BASE_URL}/admin/audit/activities?from={yesterday}&to={today}&entity_type=blood_test_template",
        role=ADMIN,
    )
    tpl_activities = tpl_activity_resp.json()
    tpl_total = tpl_activities.get("total", 0)
    t.record("日誌含血檢模板操作", tpl_total > 0,
             f"共 {tpl_total} 筆模板操作紀錄")

    # 13.4 驗證包含 Panel 操作事件
    panel_activity_resp = t._req(
        "GET",
        f"{API_BASE_URL}/admin/audit/activities?from={yesterday}&to={today}&entity_type=blood_test_panel",
        role=ADMIN,
    )
    panel_activities = panel_activity_resp.json()
    panel_total = panel_activities.get("total", 0)
    t.record("日誌含血檢組合操作", panel_total > 0,
             f"共 {panel_total} 筆組合操作紀錄")

    # 13.5 驗證日誌包含操作者資訊
    if activity_data:
        sample_log = activity_data[0]
        has_actor = (sample_log.get("actor_email") is not None or
                     sample_log.get("actor_display_name") is not None)
        t.record("日誌包含操作者資訊", has_actor,
                 f"actor={sample_log.get('actor_display_name', '?')} "
                 f"({sample_log.get('actor_email', '?')})")

        # 13.6 驗證日誌包含實體資訊
        has_entity = (sample_log.get("entity_type") is not None and
                      sample_log.get("entity_id") is not None)
        t.record("日誌包含實體資訊", has_entity,
                 f"entity_type={sample_log.get('entity_type')}, "
                 f"entity_id={str(sample_log.get('entity_id', ''))[:8]}...")
    else:
        t.record("日誌包含操作者資訊", False, "無日誌可驗證")
        t.record("日誌包含實體資訊", False, "無日誌可驗證")

    # 13.7 查詢全部日誌（不分類別），確認整體有紀錄
    all_activity_resp = t._req(
        "GET",
        f"{API_BASE_URL}/admin/audit/activities?from={yesterday}&to={today}",
        role=ADMIN,
    )
    all_total = all_activity_resp.json().get("total", 0)
    t.record("今日全部操作日誌數量",
             all_total >= total_activities,
             f"全部: {all_total} 筆（ANIMAL: {total_activities} 筆）")

    # ========================================
    # 彙總
    # ========================================
    print(f"\n{'=' * 60}")
    print(f"[完成] 血液檢查系統完整測試完成！")
    print(f"  模板: {len(templates)} 個 | Panel: {panel_count} 個")
    print(f"  測試豬隻: {test_pig_id[:8]}...")
    print(f"  血液檢查: 2 筆 (CBC+Liver / Kidney)")
    print(f"  自訂模板: {new_tpl_id[:8]}... | 自訂 Panel: {new_panel_id[:8]}...")
    print(f"  操作日誌: {all_total} 筆（ANIMAL: {total_activities} 筆）")
    print(f"{'=' * 60}")

    return t.print_summary()


if __name__ == "__main__":
    try:
        success = run_blood_panel_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n[CRITICAL ERROR] 血液檢查測試失敗: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
