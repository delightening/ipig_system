"""
ERP 倉庫管理權限測試腳本

測試 WAREHOUSE_MANAGER、ADMIN_STAFF、EXPERIMENT_STAFF 三個角色
對 ERP 倉庫管理功能的權限是否正確。

用法：
    cd c:\\System Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/test_erp_permissions.py
"""

import os
import sys
import requests
from datetime import date
from dotenv import load_dotenv

# 修正 Windows 終端機的 Unicode 編碼問題 (cp950 -> utf-8)
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

load_dotenv()

API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8000/api")

# 管理員帳號（用於建立測試用戶）
ADMIN_CREDENTIALS = {"email": "admin@ipig.local", "password": "admin123"}

# 測試帳號設定
TEST_USERS = {
    "WAREHOUSE_MANAGER": {
        "email": "wm_perm_test@example.com",
        "password": "password123",
        "display_name": "倉庫管理員 (權限測試)",
        "role_codes": ["WAREHOUSE_MANAGER"],
    },
    "ADMIN_STAFF": {
        "email": "admin_perm_test@example.com",
        "password": "password123",
        "display_name": "行政人員 (權限測試)",
        "role_codes": ["ADMIN_STAFF"],
    },
    "EXPERIMENT_STAFF": {
        "email": "exp_perm_test@example.com",
        "password": "password123",
        "display_name": "試驗工作人員 (權限測試)",
        "role_codes": ["EXPERIMENT_STAFF"],
    },
}


class ERPPermissionTester:
    def __init__(self):
        self.tokens = {}
        self.user_ids = {}
        self.created_warehouse_id = None
        self.created_product_id = None
        self.results = []

    # ========================================
    # 前置作業：建立測試帳號
    # ========================================
    def setup_test_users(self):
        """透過 Admin API 建立所有測試帳號（若已存在則跳過）"""
        print("\n" + "=" * 60)
        print("[Setup] 建立測試帳號...")
        print("=" * 60)

        # 1. 以管理員登入
        print("  [1/3] 登入管理員帳號...")
        resp = requests.post(f"{API_BASE_URL}/auth/login", json=ADMIN_CREDENTIALS)
        if resp.status_code != 200:
            print(f"  ✗ 管理員登入失敗: {resp.status_code} {resp.text}")
            return False
        admin_token = resp.json()["access_token"]
        admin_headers = {"Authorization": f"Bearer {admin_token}"}
        print("  ✓ 管理員登入成功")

        # 2. 取得角色 ID 對應表
        print("  [2/3] 讀取角色清單...")
        resp = requests.get(f"{API_BASE_URL}/roles", headers=admin_headers)
        if resp.status_code != 200:
            print(f"  ✗ 無法取得角色清單: {resp.status_code}")
            return False
        roles = resp.json()
        role_map = {r["code"]: r["id"] for r in roles}
        print(f"  ✓ 取得 {len(role_map)} 個角色")

        # 3. 逐一建立測試帳號
        print("  [3/3] 建立/確認測試帳號...")
        for role_label, user_info in TEST_USERS.items():
            login_resp = requests.post(
                f"{API_BASE_URL}/auth/login",
                json={"email": user_info["email"], "password": user_info["password"]},
            )
            if login_resp.status_code == 200:
                print(f"    ✓ {role_label}: 帳號已存在，跳過建立")
                continue

            # 建立帳號
            role_ids = [role_map[rc] for rc in user_info["role_codes"] if rc in role_map]
            create_resp = requests.post(
                f"{API_BASE_URL}/users",
                json={
                    "email": user_info["email"],
                    "password": user_info["password"],
                    "display_name": user_info["display_name"],
                    "role_ids": role_ids,
                },
                headers=admin_headers,
            )
            if create_resp.status_code in (200, 201):
                print(f"    ✓ {role_label}: 建立成功")
            else:
                print(f"    ✗ {role_label}: 建立失敗 {create_resp.status_code} {create_resp.text}")
                return False

        return True

    # ========================================
    # 前置作業：登入所有測試帳號
    # ========================================
    def login_all_users(self):
        """登入所有測試帳號，取得 token"""
        print("\n" + "=" * 60)
        print("[Setup] 登入所有測試帳號...")
        print("=" * 60)

        for role_label, user_info in TEST_USERS.items():
            resp = requests.post(
                f"{API_BASE_URL}/auth/login",
                json={"email": user_info["email"], "password": user_info["password"]},
            )
            if resp.status_code != 200:
                print(f"  ✗ {role_label} 登入失敗: {resp.status_code}")
                return False
            data = resp.json()
            self.tokens[role_label] = data["access_token"]
            self.user_ids[role_label] = data.get("user", {}).get("id")
            print(f"  ✓ {role_label} 登入成功")

        return True

    def _headers(self, role):
        """取得指定角色的 Authorization headers"""
        return {"Authorization": f"Bearer {self.tokens[role]}"}

    def _record(self, test_name, role, operation, expected_status, actual_status, success):
        """記錄測試結果"""
        self.results.append({
            "test_name": test_name,
            "role": role,
            "operation": operation,
            "expected": expected_status,
            "actual": actual_status,
            "success": success,
        })

    # ========================================
    # 測試案例
    # ========================================
    def test_warehouse_manager_create_warehouse(self):
        """測試 1: WAREHOUSE_MANAGER 新增倉庫"""
        role = "WAREHOUSE_MANAGER"
        test_name = "WM 新增倉庫"
        resp = requests.post(
            f"{API_BASE_URL}/warehouses",
            json={"name": "測試倉庫-權限測試"},
            headers=self._headers(role),
        )
        ok = resp.status_code in (200, 201)
        if ok:
            self.created_warehouse_id = resp.json().get("id")
        self._record(test_name, role, "POST /warehouses", "200/201", resp.status_code, ok)
        return ok

    def test_warehouse_manager_create_product(self):
        """測試 2: WAREHOUSE_MANAGER 新增產品"""
        role = "WAREHOUSE_MANAGER"
        test_name = "WM 新增產品"
        resp = requests.post(
            f"{API_BASE_URL}/products",
            json={
                "name": "測試產品-權限測試",
                "base_uom": "EA",
            },
            headers=self._headers(role),
        )
        ok = resp.status_code in (200, 201)
        if ok:
            self.created_product_id = resp.json().get("id")
        self._record(test_name, role, "POST /products", "200/201", resp.status_code, ok)
        return ok

    def test_warehouse_manager_create_partner(self):
        """測試 3: WAREHOUSE_MANAGER 新增夥伴"""
        role = "WAREHOUSE_MANAGER"
        test_name = "WM 新增夥伴"
        resp = requests.post(
            f"{API_BASE_URL}/partners",
            json={
                "partner_type": "supplier",
                "name": "測試供應商-權限測試",
                "supplier_category": "consumable",
            },
            headers=self._headers(role),
        )
        ok = resp.status_code in (200, 201)
        self._record(test_name, role, "POST /partners", "200/201", resp.status_code, ok)
        return ok

    def test_warehouse_manager_create_storage_location(self):
        """測試 4: WAREHOUSE_MANAGER 新增儲位"""
        role = "WAREHOUSE_MANAGER"
        test_name = "WM 新增儲位"
        if not self.created_warehouse_id:
            self._record(test_name, role, "POST /storage-locations", "200/201", "SKIP", False)
            return False

        resp = requests.post(
            f"{API_BASE_URL}/storage-locations",
            json={
                "warehouse_id": self.created_warehouse_id,
                "name": "測試儲位-權限測試",
            },
            headers=self._headers(role),
        )
        ok = resp.status_code in (200, 201)
        self._record(test_name, role, "POST /storage-locations", "200/201", resp.status_code, ok)
        return ok

    def test_admin_staff_create_warehouse(self):
        """測試 5: ADMIN_STAFF 新增倉庫"""
        role = "ADMIN_STAFF"
        test_name = "AS 新增倉庫"
        resp = requests.post(
            f"{API_BASE_URL}/warehouses",
            json={"name": "測試倉庫-行政測試"},
            headers=self._headers(role),
        )
        ok = resp.status_code in (200, 201)
        self._record(test_name, role, "POST /warehouses", "200/201", resp.status_code, ok)
        return ok

    def test_admin_staff_create_product(self):
        """測試 6: ADMIN_STAFF 新增產品"""
        role = "ADMIN_STAFF"
        test_name = "AS 新增產品"
        resp = requests.post(
            f"{API_BASE_URL}/products",
            json={"name": "測試產品-行政測試", "base_uom": "EA"},
            headers=self._headers(role),
        )
        ok = resp.status_code in (200, 201)
        self._record(test_name, role, "POST /products", "200/201", resp.status_code, ok)
        return ok

    def test_admin_staff_create_partner(self):
        """測試 7: ADMIN_STAFF 新增夥伴"""
        role = "ADMIN_STAFF"
        test_name = "AS 新增夥伴"
        resp = requests.post(
            f"{API_BASE_URL}/partners",
            json={
                "partner_type": "supplier",
                "name": "測試供應商-行政測試",
                "supplier_category": "consumable",
            },
            headers=self._headers(role),
        )
        ok = resp.status_code in (200, 201)
        self._record(test_name, role, "POST /partners", "200/201", resp.status_code, ok)
        return ok

    def test_admin_staff_create_storage_location(self):
        """測試 8: ADMIN_STAFF 新增儲位"""
        role = "ADMIN_STAFF"
        test_name = "AS 新增儲位"
        if not self.created_warehouse_id:
            self._record(test_name, role, "POST /storage-locations", "200/201", "SKIP", False)
            return False

        resp = requests.post(
            f"{API_BASE_URL}/storage-locations",
            json={
                "warehouse_id": self.created_warehouse_id,
                "name": "測試儲位-行政測試",
            },
            headers=self._headers(role),
        )
        ok = resp.status_code in (200, 201)
        self._record(test_name, role, "POST /storage-locations", "200/201", resp.status_code, ok)
        return ok

    def test_experiment_staff_view_inventory(self):
        """測試 9: EXPERIMENT_STAFF 查看庫存"""
        role = "EXPERIMENT_STAFF"
        test_name = "ES 查看庫存"
        resp = requests.get(
            f"{API_BASE_URL}/inventory/on-hand",
            headers=self._headers(role),
        )
        ok = resp.status_code == 200
        self._record(test_name, role, "GET /inventory/on-hand", "200", resp.status_code, ok)
        return ok

    def test_experiment_staff_create_sales_order(self):
        """測試 10: EXPERIMENT_STAFF 建立銷售單 (含 IACUC 歸屬)"""
        role = "EXPERIMENT_STAFF"
        test_name = "ES 建立銷售單(IACUC)"
        resp = requests.post(
            f"{API_BASE_URL}/documents",
            json={
                "doc_type": "SO",
                "doc_date": str(date.today()),
                "warehouse_id": self.created_warehouse_id,
                "iacuc_no": "PIG-11401",
                "remark": "EXPERIMENT_STAFF 權限測試 — 銷售單含 IACUC 歸屬",
                "lines": [
                    {
                        "product_id": self.created_product_id,
                        "qty": 1,
                        "uom": "EA",
                    }
                ],
            },
            headers=self._headers(role),
        )
        ok = resp.status_code in (200, 201)
        if ok:
            self.created_so_id = resp.json().get("id")
            # 驗證回傳的 iacuc_no 正確
            returned_iacuc = resp.json().get("iacuc_no")
            if returned_iacuc != "PIG-11401":
                print(f"     ⚠️ iacuc_no 回傳不符: 預期 PIG-11401, 實際 {returned_iacuc}")
        self._record(test_name, role, "POST /documents (SO+IACUC)", "200/201", resp.status_code, ok)
        return ok

    def test_experiment_staff_submit_sales_order(self):
        """測試 11: EXPERIMENT_STAFF 提交銷售單"""
        role = "EXPERIMENT_STAFF"
        test_name = "ES 提交銷售單"
        if not getattr(self, "created_so_id", None):
            self._record(test_name, role, "POST /documents/:id/submit", "200/201", "SKIP", False)
            return False

        resp = requests.post(
            f"{API_BASE_URL}/documents/{self.created_so_id}/submit",
            headers=self._headers(role),
        )
        ok = resp.status_code in (200, 201)
        self._record(test_name, role, "POST /documents/:id/submit", "200/201", resp.status_code, ok)
        return ok

    def test_experiment_staff_query_by_iacuc(self):
        """測試 12: EXPERIMENT_STAFF 依 IACUC 篩選查詢單據"""
        role = "EXPERIMENT_STAFF"
        test_name = "ES IACUC篩選查詢"
        resp = requests.get(
            f"{API_BASE_URL}/documents",
            params={"iacuc_no": "PIG-11401"},
            headers=self._headers(role),
        )
        ok = resp.status_code == 200
        if ok:
            docs = resp.json()
            found = any(d.get("iacuc_no") == "PIG-11401" for d in docs)
            if not found:
                print(f"     ⚠️ IACUC 篩選結果中未找到 PIG-11401 的單據 (共 {len(docs)} 筆)")
                ok = False
        self._record(test_name, role, "GET /documents?iacuc_no=...", "200", resp.status_code, ok)
        return ok

    # ========================================
    # 執行所有測試
    # ========================================
    def run_all_tests(self):
        """依序執行所有測試案例"""
        print("\n" + "=" * 60)
        print("[Tests] 開始執行 ERP 權限測試...")
        print("=" * 60)

        tests = [
            ("1", self.test_warehouse_manager_create_warehouse),
            ("2", self.test_warehouse_manager_create_product),
            ("3", self.test_warehouse_manager_create_partner),
            ("4", self.test_warehouse_manager_create_storage_location),
            ("5", self.test_admin_staff_create_warehouse),
            ("6", self.test_admin_staff_create_product),
            ("7", self.test_admin_staff_create_partner),
            ("8", self.test_admin_staff_create_storage_location),
            ("9", self.test_experiment_staff_view_inventory),
            ("10", self.test_experiment_staff_create_sales_order),
            ("11", self.test_experiment_staff_submit_sales_order),
            ("12", self.test_experiment_staff_query_by_iacuc),
        ]

        for num, test_fn in tests:
            try:
                ok = test_fn()
                icon = "✅" if ok else "❌"
                print(f"  {icon} 測試 {num}: {test_fn.__doc__}")
                if not ok and self.results:
                    last = self.results[-1]
                    print(f"     預期: {last['expected']}, 實際: {last['actual']}")
            except Exception as e:
                print(f"  💥 測試 {num}: {test_fn.__doc__} — 例外: {e}")

    # ========================================
    # 彙總報告
    # ========================================
    def print_report(self):
        """輸出測試結果彙總"""
        print("\n" + "=" * 60)
        print("[Report] 測試結果彙總")
        print("=" * 60)

        passed = sum(1 for r in self.results if r["success"])
        failed = len(self.results) - passed

        print(f"\n{'測試名稱':<20} {'角色':<20} {'操作':<30} {'預期':<10} {'實際':<8} {'結果'}")
        print("-" * 100)
        for r in self.results:
            icon = "✅" if r["success"] else "❌"
            print(
                f"{r['test_name']:<18} {r['role']:<20} {r['operation']:<30} "
                f"{str(r['expected']):<10} {str(r['actual']):<8} {icon}"
            )

        print(f"\n總計: {len(self.results)} 項 | ✅ 通過: {passed} | ❌ 失敗: {failed}")

        if failed == 0:
            print("\n🎉 所有權限測試通過！")
        else:
            print("\n⚠️ 有測試失敗，請檢查角色權限設定。")

        return failed == 0


def main():
    tester = ERPPermissionTester()

    # 前置作業
    if not tester.setup_test_users():
        print("\n✗ 建立測試帳號失敗，中止測試")
        sys.exit(1)

    if not tester.login_all_users():
        print("\n✗ 登入測試帳號失敗，中止測試")
        sys.exit(1)

    # 執行測試
    tester.run_all_tests()

    # 彙總報告
    all_passed = tester.print_report()

    sys.exit(0 if all_passed else 1)


if __name__ == "__main__":
    main()
