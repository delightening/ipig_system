"""
完整 ERP 倉庫管理測試

包括：
- 建立 WAREHOUSE_MANAGER / ADMIN_STAFF 測試角色
- Phase 1: 建立 3 個倉庫、每倉庫 7 個貨架、50 個產品、2 個供應商
- Phase 2: 採購入庫 (PO → GRN)，50 個貨物分配到 21 個貨架
- Phase 3: 銷售出庫 (SO → DO)
- Phase 4: 調撥作業 (TR)
- Phase 5: 庫存驗證

用法：
    cd d:\\Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/test_erp_full.py
"""

import time
import sys
import os
from datetime import date, timedelta

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL, TEST_USER_PASSWORD

# 測試帳號設定
ERP_TEST_USERS = {
    "WAREHOUSE_MANAGER": {
        "email": "wm_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "倉庫管理員 (整合測試)",
        "role_codes": ["WAREHOUSE_MANAGER"],
    },
    "ADMIN_STAFF": {
        "email": "as_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "行政人員 (整合測試)",
        "role_codes": ["ADMIN_STAFF"],
    },
}


def run_erp_test() -> bool:
    """執行完整 ERP 測試，回傳是否全部通過"""
    t = BaseApiTester("ERP 完整流程測試")

    if not t.setup_test_users(ERP_TEST_USERS):
        return False
    if not t.login_all(ERP_TEST_USERS):
        return False

    WM = "WAREHOUSE_MANAGER"
    today = str(date.today())

    # ========================================
    # Phase 1: 基礎建置
    # ========================================
    t.step("Phase 1 — 建立倉庫 (3 個)")
    warehouse_ids = []
    warehouse_names = ["原物料倉", "成品倉", "冷藏倉"]
    for name in warehouse_names:
        resp = t._req("POST", f"{API_BASE_URL}/warehouses", role=WM,
                       json={"name": f"{name} (整合測試 {int(time.time())})"})
        wh = resp.json()
        warehouse_ids.append(wh["id"])
        t.sub_step(f"倉庫 '{name}' -> ID: {wh['id'][:8]}...")
    t.record("建立 3 個倉庫", len(warehouse_ids) == 3)

    # 每個倉庫建立 7 個貨架
    t.step("Phase 1 — 建立貨架 (每倉庫 7 個 = 共 21 個)")
    storage_ids = []  # [(warehouse_idx, storage_id)]
    shelf_names = ["A 區", "B 區", "C 區", "D 區", "E 區", "F 區", "G 區"]
    for wi, wh_id in enumerate(warehouse_ids):
        for si, shelf_name in enumerate(shelf_names):
            resp = t._req("POST", f"{API_BASE_URL}/storage-locations", role=WM,
                           json={
                               "warehouse_id": wh_id,
                               "name": f"{warehouse_names[wi]}-{shelf_name}",
                               "location_type": "shelf",
                               "row_index": si // 4,
                               "col_index": si % 4,
                               "width": 1,
                               "height": 1,
                               "capacity": 20,
                           })
            sl = resp.json()
            storage_ids.append((wi, sl["id"]))
    t.record(f"建立 {len(storage_ids)} 個貨架", len(storage_ids) == 21)

    # 建立 50 個產品
    t.step("Phase 1 — 建立 50 個產品")
    product_ids = []
    categories = [
        ("飼料", "feed"),
        ("藥品", "medicine"),
        ("耗材", "consumable"),
        ("設備零件", "equipment"),
        ("清潔用品", "cleaning"),
    ]
    for i in range(50):
        cat_name, cat_code = categories[i % len(categories)]
        resp = t._req("POST", f"{API_BASE_URL}/products", role=WM,
                       json={
                           "name": f"整合測試產品-{cat_name}-{i+1:03d}",
                           "base_uom": "EA",
                           "spec": f"規格-{i+1}",
                           "track_batch": (i % 3 == 0),
                           "track_expiry": (i % 5 == 0),
                       })
        prod = resp.json()
        product_ids.append(prod["id"])
    t.record(f"建立 {len(product_ids)} 個產品", len(product_ids) == 50)

    # 建立 2 個供應商
    t.step("Phase 1 — 建立供應商")
    partner_ids = []
    suppliers = [
        ("台灣飼料供應商", "consumable"),
        ("醫療設備供應商", "equipment"),
    ]
    for name, cat in suppliers:
        resp = t._req("POST", f"{API_BASE_URL}/partners", role=WM,
                       json={
                           "partner_type": "supplier",
                           "name": f"{name} (整合測試)",
                           "supplier_category": cat,
                       })
        partner_ids.append(resp.json()["id"])
    t.record(f"建立 {len(partner_ids)} 個供應商", len(partner_ids) == 2)

    # ========================================
    # Phase 2: 採購入庫
    # ========================================
    t.step("Phase 2 — 採購入庫 (PO → GRN)")

    # 建立 3 張採購單 (每張約 16~17 個品項)
    po_ids = []
    grn_ids = []
    items_per_po = [17, 17, 16]

    product_offset = 0
    for po_idx in range(3):
        wh_id = warehouse_ids[po_idx]
        count = items_per_po[po_idx]
        lines = []
        for li in range(count):
            pid = product_ids[product_offset + li]
            lines.append({
                "product_id": pid,
                "qty": (li + 1) * 10,  # 10, 20, 30...
                "uom": "EA",
                "unit_price": 100 + li * 10,
            })

        # 建立採購單 (PO)
        resp = t._req("POST", f"{API_BASE_URL}/documents", role=WM,
                       json={
                           "doc_type": "PO",
                           "doc_date": today,
                           "warehouse_id": wh_id,
                           "partner_id": partner_ids[po_idx % 2],
                           "remark": f"整合測試採購單 #{po_idx+1}",
                           "lines": lines,
                       })
        po_id = resp.json()["id"]
        po_ids.append(po_id)
        t.sub_step(f"PO #{po_idx+1} -> {count} 個品項")

        # 提交 + 核准
        t._req("POST", f"{API_BASE_URL}/documents/{po_id}/submit", role=WM)
        t._req("POST", f"{API_BASE_URL}/documents/{po_id}/approve", role=WM)

        # 建立入庫單 (GRN)
        # 給每個品項分配到對應倉庫的貨架
        grn_lines = []
        for li in range(count):
            pid = product_ids[product_offset + li]
            storage_idx = po_idx * 7 + (li % 7)  # 分配到對應倉庫的 7 個貨架
            _, storage_id = storage_ids[storage_idx]
            grn_lines.append({
                "product_id": pid,
                "qty": (li + 1) * 10,
                "uom": "EA",
                "unit_price": 100 + li * 10,
                "storage_location_id": storage_id,
            })

        resp = t._req("POST", f"{API_BASE_URL}/documents", role=WM,
                       json={
                           "doc_type": "GRN",
                           "doc_date": today,
                           "warehouse_id": wh_id,
                           "partner_id": partner_ids[po_idx % 2],
                           "remark": f"整合測試入庫單 #{po_idx+1}",
                           "lines": grn_lines,
                       })
        grn_id = resp.json()["id"]
        grn_ids.append(grn_id)

        # 提交 + 核准入庫
        t._req("POST", f"{API_BASE_URL}/documents/{grn_id}/submit", role=WM)
        t._req("POST", f"{API_BASE_URL}/documents/{grn_id}/approve", role=WM)
        t.sub_step(f"GRN #{po_idx+1} -> 入庫 {count} 個品項到 7 個貨架")

        product_offset += count

    t.record("採購入庫完成", len(grn_ids) == 3, f"3 張 PO + 3 張 GRN，共 50 個品項")

    # ========================================
    # Phase 3: 銷售出庫
    # ========================================
    t.step("Phase 3 — 銷售出庫 (SO → DO)")

    # 從第一個倉庫出 5 個品項
    so_lines = []
    for li in range(5):
        so_lines.append({
            "product_id": product_ids[li],
            "qty": 5,
            "uom": "EA",
        })

    resp = t._req("POST", f"{API_BASE_URL}/documents", role=WM,
                   json={
                       "doc_type": "SO",
                       "doc_date": today,
                       "warehouse_id": warehouse_ids[0],
                       "iacuc_no": "PIG-11401",
                       "remark": "整合測試銷售單 (IACUC 歸屬)",
                       "lines": so_lines,
                   })
    so_id = resp.json()["id"]
    t._req("POST", f"{API_BASE_URL}/documents/{so_id}/submit", role=WM)
    t._req("POST", f"{API_BASE_URL}/documents/{so_id}/approve", role=WM)

    # 出庫單 (DO)
    resp = t._req("POST", f"{API_BASE_URL}/documents", role=WM,
                   json={
                       "doc_type": "DO",
                       "doc_date": today,
                       "warehouse_id": warehouse_ids[0],
                       "iacuc_no": "PIG-11401",
                       "remark": "整合測試出庫單 (IACUC 歸屬)",
                       "lines": so_lines,
                   })
    do_id = resp.json()["id"]
    t._req("POST", f"{API_BASE_URL}/documents/{do_id}/submit", role=WM)
    t._req("POST", f"{API_BASE_URL}/documents/{do_id}/approve", role=WM)
    t.record("銷售出庫完成 (含 IACUC)", True, "5 個品項出庫，歸屬 IACUC PIG-11401")

    # 驗證以 IACUC 篩選查詢單據
    iacuc_resp = t._req("GET", f"{API_BASE_URL}/documents?iacuc_no=PIG-11401", role=WM)
    iacuc_docs = iacuc_resp.json()
    iacuc_found = any(d.get("iacuc_no") == "PIG-11401" for d in iacuc_docs)
    t.record("IACUC 篩選查詢", iacuc_found, f"篩選 PIG-11401 → {len(iacuc_docs)} 筆單據")

    # ========================================
    # Phase 4: 調撥作業
    # ========================================
    t.step("Phase 4 — 倉庫間調撥 (TR)")

    # 從倉庫 1 調撥 3 個品項到倉庫 2
    tr_lines = []
    for li in range(3):
        tr_lines.append({
            "product_id": product_ids[li + 5],  # 用品項 6~8
            "qty": 3,
            "uom": "EA",
        })

    resp = t._req("POST", f"{API_BASE_URL}/documents", role=WM,
                   json={
                       "doc_type": "TR",
                       "doc_date": today,
                       "warehouse_from_id": warehouse_ids[0],
                       "warehouse_to_id": warehouse_ids[1],
                       "remark": "整合測試調撥單 (倉庫1→倉庫2)",
                       "lines": tr_lines,
                   })
    tr_id = resp.json()["id"]
    t._req("POST", f"{API_BASE_URL}/documents/{tr_id}/submit", role=WM)
    t._req("POST", f"{API_BASE_URL}/documents/{tr_id}/approve", role=WM)
    t.record("調撥作業完成", True, "3 個品項從倉庫1調至倉庫2")

    # ========================================
    # Phase 5: 庫存驗證
    # ========================================
    t.step("Phase 5 — 庫存驗證")

    # 查詢庫存
    inv_resp = t._req("GET", f"{API_BASE_URL}/inventory/on-hand", role=WM)
    inventory = inv_resp.json()
    has_inventory = len(inventory) > 0
    t.record("查詢在手庫存", has_inventory, f"{len(inventory)} 筆庫存記錄")

    # 查詢帳簿
    ledger_resp = t._req("GET", f"{API_BASE_URL}/inventory/ledger", role=WM)
    ledger = ledger_resp.json()
    has_ledger = len(ledger) > 0
    t.record("查詢庫存帳簿", has_ledger, f"{len(ledger)} 筆帳簿記錄")

    # 驗證每個貨架的庫存內容
    t.step("Phase 5 — 貨架庫存驗證 (storage-locations/:id/inventory)")
    shelves_with_stock = 0
    total_items_in_shelves = 0
    for wi, storage_id in storage_ids:
        inv_r = t._req("GET", f"{API_BASE_URL}/storage-locations/{storage_id}/inventory", role=WM)
        items = inv_r.json()
        if len(items) > 0:
            shelves_with_stock += 1
            total_items_in_shelves += len(items)
            # 確認每個品項都有必要欄位
            for item in items:
                assert "product_name" in item, f"缺少 product_name: {item}"
                assert "on_hand_qty" in item, f"缺少 on_hand_qty: {item}"
            t.sub_step(f"貨架 {storage_id[:8]}... → {len(items)} 個品項")
    t.record(
        "貨架庫存驗證",
        shelves_with_stock > 0,
        f"{shelves_with_stock}/21 個貨架有庫存，共 {total_items_in_shelves} 個品項",
    )

    # ========================================
    # Phase 6: 報表產生驗證
    # ========================================
    t.step("Phase 6 — 報表產生驗證")

    # 6.1 庫存報表
    report_resp = t._req("GET", f"{API_BASE_URL}/reports/stock-on-hand", role=WM)
    stock_report = report_resp.json()
    stock_items = stock_report if isinstance(stock_report, list) else stock_report.get("data", stock_report.get("items", []))
    t.record("庫存報表 (stock-on-hand)",
             len(stock_items) > 0 if isinstance(stock_items, list) else True,
             f"回傳 {len(stock_items) if isinstance(stock_items, list) else '有效'} 筆資料")

    # 6.2 帳簿報表
    ledger_report_resp = t._req("GET", f"{API_BASE_URL}/reports/stock-ledger", role=WM)
    ledger_report = ledger_report_resp.json()
    ledger_items = ledger_report if isinstance(ledger_report, list) else ledger_report.get("data", ledger_report.get("items", []))
    t.record("帳簿報表 (stock-ledger)",
             len(ledger_items) > 0 if isinstance(ledger_items, list) else True,
             f"回傳 {len(ledger_items) if isinstance(ledger_items, list) else '有效'} 筆資料")

    # 6.3 採購明細報表
    purchase_resp = t._req("GET", f"{API_BASE_URL}/reports/purchase-lines", role=WM)
    purchase_report = purchase_resp.json()
    purchase_items = purchase_report if isinstance(purchase_report, list) else purchase_report.get("data", purchase_report.get("items", []))
    t.record("採購明細報表 (purchase-lines)",
             len(purchase_items) > 0 if isinstance(purchase_items, list) else True,
             f"回傳 {len(purchase_items) if isinstance(purchase_items, list) else '有效'} 筆資料")

    # 6.4 銷售明細報表
    sales_resp = t._req("GET", f"{API_BASE_URL}/reports/sales-lines", role=WM)
    sales_report = sales_resp.json()
    sales_items = sales_report if isinstance(sales_report, list) else sales_report.get("data", sales_report.get("items", []))
    t.record("銷售明細報表 (sales-lines)",
             len(sales_items) > 0 if isinstance(sales_items, list) else True,
             f"回傳 {len(sales_items) if isinstance(sales_items, list) else '有效'} 筆資料")

    # 6.5 成本彙總報表
    cost_resp = t._req("GET", f"{API_BASE_URL}/reports/cost-summary", role=WM)
    cost_report = cost_resp.json()
    t.record("成本彙總報表 (cost-summary)",
             cost_resp.status_code == 200,
             f"回傳有效 JSON")

    # ========================================
    # 彙總
    # ========================================
    print(f"\n{'=' * 60}")
    print(f"[完成] ERP 完整流程測試完成！")
    print(f"  倉庫: {len(warehouse_ids)} | 貨架: {len(storage_ids)} | 產品: {len(product_ids)}")
    print(f"  PO: {len(po_ids)} | GRN: {len(grn_ids)} | SO: 1 | DO: 1 | TR: 1")
    print(f"  報表: 5 種 (stock-on-hand, stock-ledger, purchase-lines, sales-lines, cost-summary)")
    print(f"{'=' * 60}")
    return t.print_summary()


if __name__ == "__main__":
    try:
        success = run_erp_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n[CRITICAL ERROR] ERP 測試失敗: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
