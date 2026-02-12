"""
整合測試共用基底模組

提供所有測試腳本共用的功能：
- 管理員登入、測試帳號建立
- HTTP 請求包裝與錯誤記錄
- Token / Headers 管理
"""

import os
import sys
import json
import time
import requests
from dotenv import load_dotenv

# 修正 Windows 終端機的 Unicode 編碼問題 (cp950 -> utf-8)
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

load_dotenv()

API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8000/api")
ADMIN_CREDENTIALS = {"email": "admin@ipig.local", "password": "admin123"}


class BaseApiTester:
    """整合測試基底類別"""

    def __init__(self, test_name: str = ""):
        self.test_name = test_name
        self.tokens = {}
        self.user_ids = {}
        self.admin_token = None
        self.role_map = {}  # code -> id
        self.results = []
        self.step_count = 0

    # ========================================
    # 前置作業
    # ========================================

    def admin_login(self) -> bool:
        """以管理員身份登入"""
        resp = requests.post(f"{API_BASE_URL}/auth/login", json=ADMIN_CREDENTIALS)
        if resp.status_code != 200:
            print(f"  ✗ 管理員登入失敗: {resp.status_code} {resp.text}")
            return False
        self.admin_token = resp.json()["access_token"]
        print(f"  ✓ 管理員登入成功")
        return True

    def admin_headers(self) -> dict:
        return {"Authorization": f"Bearer {self.admin_token}"}

    def fetch_roles(self) -> bool:
        """取得系統角色 ID 對應表"""
        resp = requests.get(f"{API_BASE_URL}/roles", headers=self.admin_headers())
        if resp.status_code != 200:
            print(f"  ✗ 無法取得角色清單: {resp.status_code}")
            return False
        for r in resp.json():
            self.role_map[r["code"]] = r["id"]
        print(f"  ✓ 取得 {len(self.role_map)} 個角色: {list(self.role_map.keys())}")
        return True

    def setup_test_users(self, users_config: dict) -> bool:
        """建立測試帳號（若已存在則跳過）"""
        print(f"\n{'=' * 60}")
        print(f"[Setup] 建立測試帳號...")
        print(f"{'=' * 60}")

        if not self.admin_login():
            return False
        if not self.fetch_roles():
            return False

        created, skipped = 0, 0
        for role_label, user_info in users_config.items():
            # 嘗試登入，成功代表帳號已存在
            login_resp = requests.post(f"{API_BASE_URL}/auth/login", json={
                "email": user_info["email"],
                "password": user_info["password"]
            })
            if login_resp.status_code == 200:
                print(f"    ✓ {role_label:20s} ({user_info['email']}) — 已存在")
                skipped += 1
                continue

            # 建立帳號
            role_ids = [self.role_map[rc] for rc in user_info["role_codes"] if rc in self.role_map]
            payload = {
                "email": user_info["email"],
                "password": user_info["password"],
                "display_name": user_info["display_name"],
                "role_ids": role_ids,
            }
            create_resp = requests.post(f"{API_BASE_URL}/users", json=payload, headers=self.admin_headers())
            if create_resp.status_code in (200, 201):
                print(f"    ✓ {role_label:20s} ({user_info['email']}) — 建立成功")
                created += 1
            else:
                print(f"    ✗ {role_label:20s} ({user_info['email']}) — 建立失敗: {create_resp.status_code}")
                return False

        print(f"\n  [結果] 新建 {created} / 已存在 {skipped} / 共 {len(users_config)}")
        return True

    def login_all(self, users_config: dict) -> bool:
        """登入所有測試帳號"""
        print(f"\n{'=' * 60}")
        print(f"[Auth] 登入所有測試帳號...")
        print(f"{'=' * 60}")
        success = 0
        for role, user_info in users_config.items():
            resp = requests.post(f"{API_BASE_URL}/auth/login", json={
                "email": user_info["email"],
                "password": user_info["password"]
            })
            if resp.status_code == 200:
                data = resp.json()
                self.tokens[role] = data["access_token"]
                self.user_ids[role] = data["user"]["id"]
                print(f"  ✓ {role:20s} 登入成功 (ID: {data['user']['id'][:8]}...)")
                success += 1
            else:
                print(f"  ✗ {role:20s} 登入失敗: {resp.status_code}")
        if success < len(users_config):
            print(f"\n  ✗ 部分帳號登入失敗 ({success}/{len(users_config)})")
            return False
        print(f"\n  ✓ {success}/{len(users_config)} 帳號全部登入成功")
        return True

    # ========================================
    # HTTP 請求包裝
    # ========================================

    def get_headers(self, role: str) -> dict:
        return {"Authorization": f"Bearer {self.tokens[role]}"}

    def _req(self, method: str, url: str, role: str = None, **kwargs):
        """統一 HTTP 請求，含錯誤記錄"""
        if role and "headers" not in kwargs:
            kwargs["headers"] = self.get_headers(role)
        resp = requests.request(method, url, **kwargs)

        # 記錄錯誤
        if resp.status_code >= 400:
            error_data = {
                "test": self.test_name,
                "method": method,
                "url": url,
                "status": resp.status_code,
                "payload": kwargs.get("json"),
                "response": resp.text[:500],
            }
            try:
                error_data["response_json"] = resp.json()
            except Exception:
                pass
            with open("tests/last_error.json", "w", encoding="utf-8") as f:
                json.dump(error_data, f, indent=2, ensure_ascii=False)

        short_url = "/".join(url.split("/")[-3:])
        status_icon = "✓" if resp.status_code < 400 else "✗"
        print(f"  {status_icon} [{method:6s}] /{short_url} -> {resp.status_code}")
        resp.raise_for_status()
        return resp

    # ========================================
    # 步驟輸出
    # ========================================

    def step(self, msg: str):
        """輸出步驟標題"""
        self.step_count += 1
        print(f"\n{'=' * 60}")
        print(f"[Step {self.step_count}] {msg}")
        print(f"{'=' * 60}")

    def sub_step(self, msg: str):
        """輸出子步驟"""
        print(f"  → {msg}")

    def record(self, name: str, success: bool, detail: str = ""):
        """記錄測試結果"""
        icon = "✅" if success else "❌"
        self.results.append({"name": name, "success": success, "detail": detail})
        print(f"  {icon} {name}" + (f" — {detail}" if detail else ""))

    def print_summary(self) -> bool:
        """輸出測試結果彙總"""
        passed = sum(1 for r in self.results if r["success"])
        failed = len(self.results) - passed
        print(f"\n{'=' * 60}")
        print(f"[{self.test_name}] 測試結果彙總")
        print(f"{'=' * 60}")
        for r in self.results:
            icon = "✅" if r["success"] else "❌"
            print(f"  {icon} {r['name']}")
        print(f"\n  總計: {len(self.results)} 項 ✅ {passed} / ❌ {failed}")
        if failed == 0:
            print(f"\n🎉 {self.test_name} — 所有測試通過！")
        else:
            print(f"\n⚠️ {self.test_name} — 有 {failed} 項失敗！")
        return failed == 0
