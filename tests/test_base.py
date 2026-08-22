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
TEST_USER_PASSWORD = os.getenv("TEST_USER_PASSWORD", "password123")
ADMIN_CREDENTIALS = {
    "email": os.getenv("TEST_ADMIN_EMAIL", "admin@example.com"),
    "password": os.getenv("TEST_ADMIN_PASSWORD", "changeme"),
}


class BaseApiTester:
    """整合測試基底類別"""

    def __init__(self, test_name: str = ""):
        self.test_name = test_name
        self.tokens = {}
        self.user_ids = {}
        self.admin_token = None
        self.csrf_token = None
        self.role_map = {}  # code -> id
        self.results = []
        self.step_count = 0
        # 使用 Session 自動管理 Cookie（含 CSRF token）
        self.session = requests.Session()

    # ========================================
    # 前置作業
    # ========================================

    @staticmethod
    def _extract_cookie(response, cookie_name: str) -> str | None:
        """從 Set-Cookie header 提取指定 cookie 值（SEC-02 適配）"""
        for header_value in response.headers.get_all("Set-Cookie") if hasattr(response.headers, 'get_all') else response.headers.get("Set-Cookie", "").split(","):
            for part in header_value.split(";"):
                part = part.strip()
                if part.startswith(f"{cookie_name}="):
                    val = part.split("=", 1)[1]
                    if val:  # 排除清除 cookie 的空值
                        return val
        # 也嘗試 requests 的 cookies jar
        return response.cookies.get(cookie_name)

    @staticmethod
    def _login_with_retry(credentials: dict, max_retries: int = 3):
        """登入 API，遇到 429 自動等待重試（不經過 session，避免污染 cookie）"""
        for attempt in range(max_retries):
            resp = requests.post(f"{API_BASE_URL}/auth/login", json=credentials)
            if resp.status_code != 429:
                return resp
            # 取得 Retry-After 秒數（預設 60 秒）
            retry_after = int(resp.headers.get("Retry-After", "60"))
            print(f"  ⏳ 速率限制觸發 (429)，等待 {retry_after} 秒後重試 ({attempt + 1}/{max_retries})...")
            time.sleep(retry_after)
        return resp  # 最後一次嘗試的回應

    def admin_login(self) -> bool:
        """以管理員身份登入（含 429 自動重試 + CSRF token 取得）"""
        # 用 session 登入以建立 cookie 基礎（/auth/login 免 CSRF 驗證）
        for attempt in range(3):
            resp = self.session.post(f"{API_BASE_URL}/auth/login", json=ADMIN_CREDENTIALS)
            if resp.status_code != 429:
                break
            retry_after = int(resp.headers.get("Retry-After", "60"))
            print(f"  ⏳ 速率限制觸發 (429)，等待 {retry_after} 秒後重試 ({attempt + 1}/3)...")
            time.sleep(retry_after)
        if resp.status_code != 200:
            print(f"  ✗ 管理員登入失敗: {resp.status_code} {resp.text}")
            return False
        # SEC-02：從 Set-Cookie header 提取 access_token
        self.admin_token = self._extract_cookie(resp, "access_token")
        if not self.admin_token:
            print(f"  ✗ 管理員登入成功但無法取得 access_token cookie")
            return False
        print(f"  ✓ 管理員登入成功")
        return True

    def admin_headers(self) -> dict:
        self._refresh_csrf()
        h = {"Authorization": f"Bearer {self.admin_token}"}
        if self.csrf_token:
            h["X-CSRF-Token"] = self.csrf_token
        return h

    def _refresh_csrf(self):
        """從 Session cookies 更新 CSRF token（每次回應後可能輪替）"""
        try:
            new_csrf = self.session.cookies.get("csrf_token")
        except Exception:
            # 多次登入後可能累積多個同名 csrf_token cookie，
            # cookies.get() 會拋出 CookieConflictError
            # 遍歷 cookie jar 取最後一個值
            new_csrf = None
            for cookie in self.session.cookies:
                if cookie.name == "csrf_token" and cookie.value:
                    new_csrf = cookie.value
        if new_csrf:
            self.csrf_token = new_csrf

    def fetch_roles(self) -> bool:
        """取得系統角色 ID 對應表"""
        resp = self.session.get(f"{API_BASE_URL}/roles", headers=self.admin_headers())
        if resp.status_code != 200:
            print(f"  ✗ 無法取得角色清單: {resp.status_code}")
            return False
        for r in resp.json():
            self.role_map[r["code"]] = r["id"]
        # SEC-24：CSRF middleware 在 protected routes 回應時設定 csrf_token cookie
        self._refresh_csrf()
        print(f"  ✓ 取得 {len(self.role_map)} 個角色: {list(self.role_map.keys())}")
        return True

    def _save_login(self, role_label: str, resp) -> bool:
        """從登入回應中保存 token 和 user_id，回傳是否成功"""
        if resp.status_code != 200:
            return False
        data = resp.json()
        token = self._extract_cookie(resp, "access_token")
        if not token:
            return False
        self.tokens[role_label] = token
        self.user_ids[role_label] = data["user"]["id"]
        return True

    def setup_test_users(self, users_config: dict) -> bool:
        """建立測試帳號（若已存在則跳過），同時保存 token 供後續使用"""
        print(f"\n{'=' * 60}")
        print(f"[Setup] 建立測試帳號...")
        print(f"{'=' * 60}")

        if not self.admin_login():
            return False
        if not self.fetch_roles():
            return False

        created, skipped = 0, 0
        for role_label, user_info in users_config.items():
            # 嘗試登入，成功代表帳號已存在 → 同時保存 token（避免 login_all 重複登入）
            login_resp = self._login_with_retry({
                "email": user_info["email"],
                "password": user_info["password"]
            })
            if self._save_login(role_label, login_resp):
                uid_short = self.user_ids[role_label][:8]
                print(f"    ✓ {role_label:20s} ({user_info['email']}) — 已存在 (ID: {uid_short}...)")
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
            create_resp = self.session.post(f"{API_BASE_URL}/users", json=payload, headers=self.admin_headers())
            if create_resp.status_code in (200, 201):
                # 新建帳號後立即登入保存 token
                new_login = self._login_with_retry({
                    "email": user_info["email"],
                    "password": user_info["password"]
                })
                self._save_login(role_label, new_login)
                uid_short = self.user_ids.get(role_label, "?")[:8] if role_label in self.user_ids else "?"
                print(f"    ✓ {role_label:20s} ({user_info['email']}) — 建立成功 (ID: {uid_short}...)")
                created += 1
            else:
                print(f"    ✗ {role_label:20s} ({user_info['email']}) — 建立失敗: {create_resp.status_code}")
                return False

        print(f"\n  [結果] 新建 {created} / 已存在 {skipped} / 共 {len(users_config)}")
        return True

    def login_all(self, users_config: dict) -> bool:
        """登入所有測試帳號（跳過 setup_test_users 已登入的帳號）"""
        print(f"\n{'=' * 60}")
        print(f"[Auth] 登入所有測試帳號...")
        print(f"{'=' * 60}")
        success, reused = 0, 0
        for role, user_info in users_config.items():
            # 若 setup_test_users 已保存 token，直接沿用
            if role in self.tokens and role in self.user_ids:
                print(f"  ✓ {role:20s} 沿用已登入 token (ID: {self.user_ids[role][:8]}...)")
                success += 1
                reused += 1
                continue

            resp = self._login_with_retry({
                "email": user_info["email"],
                "password": user_info["password"]
            })
            if self._save_login(role, resp):
                print(f"  ✓ {role:20s} 登入成功 (ID: {self.user_ids[role][:8]}...)")
                success += 1
            else:
                print(f"  ✗ {role:20s} 登入失敗: {resp.status_code}")
        if success < len(users_config):
            print(f"\n  ✗ 部分帳號登入失敗 ({success}/{len(users_config)})")
            return False
        print(f"\n  ✓ {success}/{len(users_config)} 帳號全部就緒 (沿用 {reused} / 新登入 {success - reused})")
        return True

    # ========================================
    # HTTP 請求包裝
    # ========================================

    def get_headers(self, role: str) -> dict:
        self._refresh_csrf()
        h = {"Authorization": f"Bearer {self.tokens[role]}"}
        if self.csrf_token:
            h["X-CSRF-Token"] = self.csrf_token
        return h

    def _req(self, method: str, url: str, role: str = None, **kwargs):
        """統一 HTTP 請求，含 CSRF token、錯誤記錄與 429 自動重試
        
        使用 session 發送以確保 csrf_token cookie 被傳送（CSRF Double Submit）。
        但先清除 session 中的 access_token/refresh_token cookie，
        避免後端 auth_middleware (Cookie > Bearer) 用 admin cookie 覆蓋 Bearer token。
        """
        if role and "headers" not in kwargs:
            kwargs["headers"] = self.get_headers(role)

        # 清除 session 中的 auth cookie，只保留 csrf_token
        for cookie_name in ("access_token", "refresh_token"):
            self.session.cookies.pop(cookie_name, None)

        # 429 自動重試（最多 3 次）
        for attempt in range(3):
            resp = self.session.request(method, url, **kwargs)
            self._refresh_csrf()
            # 清除回應中可能設定的 auth cookie（避免後續請求汙染）
            for cookie_name in ("access_token", "refresh_token"):
                self.session.cookies.pop(cookie_name, None)
            if resp.status_code != 429:
                break
            retry_after = int(resp.headers.get("Retry-After", "60"))
            print(f"  ⏳ API 速率限制 (429)，等待 {retry_after} 秒... ({attempt + 1}/3)")
            time.sleep(retry_after)

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
        """輸出測試結果彙總，並自動儲存至 tests/results/"""
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

        # 自動儲存測試結果
        self.save_results()
        return failed == 0

    def save_results(self):
        """將測試結果儲存至 tests/results/YYYY_MM_DD_HH_MM_testname.txt"""
        from datetime import datetime

        results_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "results")
        os.makedirs(results_dir, exist_ok=True)

        # 產生檔名：YYYY_MM_DD_HH_MM_testname.txt
        now = datetime.now()
        # 將測試名稱轉換為安全的檔名（移除特殊字元，空格轉底線）
        safe_name = self.test_name.replace(" ", "_")
        for ch in "/<>:\"|?*":
            safe_name = safe_name.replace(ch, "")
        filename = f"{now.strftime('%Y_%m_%d_%H_%M')}_{safe_name}.txt"
        filepath = os.path.join(results_dir, filename)

        passed = sum(1 for r in self.results if r["success"])
        failed = len(self.results) - passed

        lines = []
        lines.append(f"{'=' * 60}")
        lines.append(f"[{self.test_name}] 測試結果")
        lines.append(f"執行時間: {now.strftime('%Y-%m-%d %H:%M:%S')}")
        lines.append(f"{'=' * 60}")
        lines.append("")
        for r in self.results:
            icon = "PASS" if r["success"] else "FAIL"
            line = f"  [{icon}] {r['name']}"
            if r.get("detail"):
                line += f" — {r['detail']}"
            lines.append(line)
        lines.append("")
        lines.append(f"總計: {len(self.results)} 項 | PASS {passed} | FAIL {failed}")
        lines.append(f"結果: {'ALL PASSED' if failed == 0 else f'{failed} FAILED'}")

        try:
            with open(filepath, "w", encoding="utf-8") as f:
                f.write("\n".join(lines) + "\n")
            print(f"\n  📄 結果已儲存: {filepath}")
        except Exception as e:
            print(f"\n  ⚠️ 無法儲存結果: {e}")

    # ========================================
    # 測試資料清理
    # ========================================

    @staticmethod
    def cleanup_test_data() -> bool:
        """
        執行測試資料清理：刪除業務記錄，保留安全審計資料。

        自動偵測 Docker / 本機環境並執行 cleanup_test_data.sql。
        """
        import subprocess

        # 找 SQL 腳本路徑
        project_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        sql_file = os.path.join(project_root, "backend", "cleanup_test_data.sql")
        if not os.path.exists(sql_file):
            print(f"  ✗ 找不到清理腳本: {sql_file}")
            return False

        print(f"\n{'=' * 60}")
        print(f"[Cleanup] 清理測試資料（保留審計記錄）...")
        print(f"{'=' * 60}")

        # 嘗試 Docker 模式
        try:
            result = subprocess.run(
                ["docker", "ps", "--filter", "name=ipig-db", "--format", "{{.Names}}"],
                capture_output=True, text=True, timeout=5
            )
            if "ipig-db" in result.stdout:
                print("  → 使用 Docker 模式")
                # 複製 SQL 到容器
                subprocess.run(
                    ["docker", "cp", sql_file, "ipig-db:/tmp/cleanup_test_data.sql"],
                    check=True, timeout=10
                )
                # 執行 SQL
                proc = subprocess.run(
                    ["docker", "exec", "ipig-db", "psql", "-U", "postgres", "-d", "ipig_db",
                     "-f", "/tmp/cleanup_test_data.sql"],
                    capture_output=True, text=True, timeout=30
                )
                # 清理暫存
                subprocess.run(
                    ["docker", "exec", "ipig-db", "rm", "-f", "/tmp/cleanup_test_data.sql"],
                    timeout=5
                )
                if proc.returncode == 0:
                    print("  ✓ 測試資料清理完成")
                    # 輸出 NOTICE 訊息
                    for line in proc.stderr.splitlines():
                        if "NOTICE" in line:
                            msg = line.split("NOTICE:")[1].strip() if "NOTICE:" in line else line
                            print(f"    {msg}")
                    return True
                else:
                    print(f"  ✗ 清理失敗: {proc.stderr}")
                    return False
        except (subprocess.TimeoutExpired, FileNotFoundError, subprocess.CalledProcessError):
            pass

        # 本機 psql 模式
        try:
            print("  → 使用本機 psql 模式")
            db_url = os.getenv("DATABASE_URL", "postgres://postgres:postgres@localhost:5432/ipig_db")
            db_url = db_url.replace("@db:", "@localhost:")
            proc = subprocess.run(
                ["psql", db_url, "-f", sql_file],
                capture_output=True, text=True, timeout=30
            )
            if proc.returncode == 0:
                print("  ✓ 測試資料清理完成")
                return True
            else:
                print(f"  ✗ 清理失敗: {proc.stderr}")
                return False
        except (subprocess.TimeoutExpired, FileNotFoundError) as e:
            print(f"  ✗ 無法執行 psql: {e}")
            return False
