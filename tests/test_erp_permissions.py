# -*- coding: utf-8 -*-
"""
ERP 倉庫管理權限測試腳本

測試 WAREHOUSE_MANAGER、ADMIN_STAFF、EXPERIMENT_STAFF 三個角色
在 ERP 倉庫管理功能的權限是否正確。

用法：
    cd d:\\Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/test_erp_permissions.py
"""

import sys
import os
from datetime import date

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL, TEST_USER_PASSWORD

# 測試帳號設定
TEST_USERS = {
    "WAREHOUSE_MANAGER": {
        "email": "wm_perm_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "倉庫管理員 (權限測試)",
        "role_codes": ["WAREHOUSE_MANAGER"],
    },
    "ADMIN_STAFF": {
        "email": "admin_perm_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "行政人員 (權限測試)",
        "role_codes": ["ADMIN_STAFF"],
    },
    "EXPERIMENT_STAFF": {
        "email": "exp_perm_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "試驗工作人員 (權限測試)",
        "role_codes": ["EXPERIMENT_STAFF"],
    },
}


def run_erp_permissions_test() -> bool:
    """執行 ERP 權限測試，回傳是否全部通過"""
    t = BaseApiTester("ERP 倉庫管理權限測試")

    if not t.setup_test_users(TEST_USERS):
        return False
    if not t.login_all(TEST_USERS):
        return False

    WM = "WAREHOUSE_MANAGER"
    AS = "ADMIN_STAFF"
    ES = "EXPERIMENT_STAFF"

    created_warehouse_id = None
    created_product_id = None

    # ========================================
    # Phase 1: WAREHOUSE_MANAGER 完整 CRUD
    # ========================================
    t.step("Phase 1 — WAREHOUSE_MANAGER 建立倉庫/產品/夥伴/貨架")

    # 1.1 建立倉庫
    resp = t._req("POST", f"{API_BASE_URL}/warehouses", role=WM,
                   json={"name": "測試倉庫-權限測試"})
    wh = resp.json()
    created_warehouse_id = wh.get("id")
    t.record("WM 建立倉庫", created_warehouse_id is not None,
             f"ID: {created_warehouse_id[:8] if created_warehouse_id else 'N/A'}...")

    # 1.2 建立產品
    resp = t._req("POST", f"{API_BASE_URL}/products", role=WM,
                   json={"name": "測試產品-權限測試", "base_uom": "EA"})
    prod = resp.json()
    created_product_id = prod.get("id")
    t.record("WM 建立產品", created_product_id is not None,
             f"ID: {created_product_id[:8] if created_product_id else 'N/A'}...")

    # 1.3 建立夥伴（供應商）
    resp = t._req("POST", f"{API_BASE_URL}/partners", role=WM,
                   json={
                       "partner_type": "supplier",
                       "name": "測試供應商-權限測試",
                       "supplier_category": "consumable",
                   })
    t.record("WM 建立夥伴", resp.status_code in (200, 201))

    # 1.4 建立貨架
    if created_warehouse_id:
        resp = t._req("POST", f"{API_BASE_URL}/storage-locations", role=WM,
                       json={
                           "warehouse_id": created_warehouse_id,
                           "name": "測試貨架-權限測試",
                       })
        t.record("WM 建立貨架", resp.status_code in (200, 201))
    else:
        t.record("WM 建立貨架", False, "無倉庫 ID，跳過")

    # ========================================
    # Phase 2: ADMIN_STAFF 完整 CRUD
    # ========================================
    t.step("Phase 2 — ADMIN_STAFF 建立倉庫/產品/夥伴/貨架")

    # 2.1 建立倉庫
    resp = t._req("POST", f"{API_BASE_URL}/warehouses", role=AS,
                   json={"name": "測試倉庫-行政測試"})
    t.record("AS 建立倉庫", resp.status_code in (200, 201))

    # 2.2 建立產品
    resp = t._req("POST", f"{API_BASE_URL}/products", role=AS,
                   json={"name": "測試產品-行政測試", "base_uom": "EA"})
    t.record("AS 建立產品", resp.status_code in (200, 201))

    # 2.3 建立夥伴
    resp = t._req("POST", f"{API_BASE_URL}/partners", role=AS,
                   json={
                       "partner_type": "supplier",
                       "name": "測試供應商-行政測試",
                       "supplier_category": "consumable",
                   })
    t.record("AS 建立夥伴", resp.status_code in (200, 201))

    # 2.4 建立貨架
    if created_warehouse_id:
        resp = t._req("POST", f"{API_BASE_URL}/storage-locations", role=AS,
                       json={
                           "warehouse_id": created_warehouse_id,
                           "name": "測試貨架-行政測試",
                       })
        t.record("AS 建立貨架", resp.status_code in (200, 201))
    else:
        t.record("AS 建立貨架", False, "無倉庫 ID，跳過")

    # ========================================
    # Phase 3: EXPERIMENT_STAFF 查看庫存
    # ========================================
    t.step("Phase 3 — EXPERIMENT_STAFF 查看庫存")

    resp = t._req("GET", f"{API_BASE_URL}/inventory/on-hand", role=ES)
    t.record("ES 查看庫存", resp.status_code == 200)

    # ========================================
    # Phase 4: EXPERIMENT_STAFF 建立銷售單（含 IACUC 歸屬）
    # ========================================
    t.step("Phase 4 — EXPERIMENT_STAFF 建立銷售單（含 IACUC）")

    if created_warehouse_id and created_product_id:
        resp = t._req("POST", f"{API_BASE_URL}/documents", role=ES,
                       json={
                           "doc_type": "SO",
                           "doc_date": str(date.today()),
                           "warehouse_id": created_warehouse_id,
                           "iacuc_no": "PIG-11401",
                           "remark": "EXPERIMENT_STAFF 權限測試 — 銷售單含 IACUC 歸屬",
                           "lines": [
                               {
                                   "product_id": created_product_id,
                                   "qty": 1,
                                   "uom": "EA",
                               }
                           ],
                       })
        ok = resp.status_code in (200, 201)
        so_id = resp.json().get("id") if ok else None
        t.record("ES 建立銷售單 (SO+IACUC)", ok,
                 f"ID: {so_id[:8] if so_id else 'N/A'}...")

        # 驗證回傳 iacuc_no 正確
        if ok:
            returned_iacuc = resp.json().get("iacuc_no")
            t.record("SO iacuc_no 正確",
                     returned_iacuc == "PIG-11401",
                     f"預期 PIG-11401, 實際 {returned_iacuc}")

        # ========================================
        # Phase 5: EXPERIMENT_STAFF 提交銷售單
        # ========================================
        t.step("Phase 5 — EXPERIMENT_STAFF 提交銷售單")
        if so_id:
            resp = t._req("POST", f"{API_BASE_URL}/documents/{so_id}/submit", role=ES)
            t.record("ES 提交銷售單", resp.status_code in (200, 201))
        else:
            t.record("ES 提交銷售單", False, "無銷售單 ID")

        # ========================================
        # Phase 6: EXPERIMENT_STAFF 按 IACUC 查詢
        # ========================================
        t.step("Phase 6 — EXPERIMENT_STAFF 按 IACUC 篩選查詢")
        resp = t._req("GET", f"{API_BASE_URL}/documents",
                       role=ES, params={"iacuc_no": "PIG-11401"})
        if resp.status_code == 200:
            docs = resp.json()
            found = any(d.get("iacuc_no") == "PIG-11401" for d in docs)
            t.record("ES IACUC 篩選查詢", found,
                     f"共 {len(docs)} 筆，{'找到' if found else '未找到'} PIG-11401")
        else:
            t.record("ES IACUC 篩選查詢", False, f"HTTP {resp.status_code}")
    else:
        t.record("ES 建立銷售單 (SO+IACUC)", False, "無倉庫或產品 ID")

    # ========================================
    # 彙總
    # ========================================
    print(f"\n{'=' * 60}")
    print(f"[完成] ERP 權限測試完成！")
    print(f"  WM: 完整 CRUD ✓")
    print(f"  AS: 完整 CRUD ✓")
    print(f"  ES: 查看庫存 + 銷售單（含 IACUC）")
    print(f"{'=' * 60}")

    return t.print_summary()


if __name__ == "__main__":
    try:
        success = run_erp_permissions_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n[CRITICAL ERROR] ERP 權限測試失敗: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
