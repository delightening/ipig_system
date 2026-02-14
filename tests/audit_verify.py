"""
驗證活動紀錄 - 自動化 API 測試腳本
涵蓋：Dashboard、Activities、Logins、Sessions、Alerts、Protocol Activities
"""
import os
import sys
import json
import requests
from dotenv import load_dotenv

load_dotenv()

API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8000/api")

# 測試帳號
STAFF_CREDS = {"email": "staff_test@example.com", "password": "password123"}

# 最近的 protocol_id（由 aup_test_standalone.py 產生）
KNOWN_PROTOCOL_ID = None  # 會在測試中自動取得

class AuditVerifier:
    def __init__(self):
        self.token = None
        self.user_id = None
        self.results = []
        self.protocol_id = None

    def login(self):
        print("\n[Auth] 登入 IACUC_STAFF...")
        resp = requests.post(f"{API_BASE_URL}/auth/login", json=STAFF_CREDS)
        assert resp.status_code == 200, f"登入失敗: {resp.status_code}"
        data = resp.json()
        # SEC-02：從 cookies 提取 access_token
        self.token = resp.cookies.get("access_token")
        self.user_id = data["user"]["id"]
        print(f"  ✓ 登入成功 (user_id: {self.user_id})")

    def headers(self):
        return {"Authorization": f"Bearer {self.token}"}

    def record(self, name, passed, detail=""):
        status = "✓ PASS" if passed else "✗ FAIL"
        self.results.append({"name": name, "passed": passed, "detail": detail})
        print(f"  {status}: {name}" + (f" — {detail}" if detail else ""))

    def verify_dashboard(self):
        """檢查 1: Dashboard 統計"""
        print("\n[1/6] 驗證 Dashboard 統計...")
        resp = requests.get(f"{API_BASE_URL}/admin/audit/dashboard", headers=self.headers())
        if resp.status_code != 200:
            self.record("Dashboard API 回傳 200", False, f"status={resp.status_code}")
            return
        self.record("Dashboard API 回傳 200", True)

        data = resp.json()
        expected_fields = [
            "active_users_today", "active_users_week", "active_users_month",
            "total_logins_today", "failed_logins_today",
            "active_sessions", "open_alerts", "critical_alerts"
        ]
        missing = [f for f in expected_fields if f not in data]
        self.record("Dashboard 包含所有必要欄位", len(missing) == 0,
                     f"缺少: {missing}" if missing else f"8/8 欄位齊全")

        # 驗證數值合理性
        logins = data.get("total_logins_today", 0)
        self.record("今日登入數 > 0", logins > 0, f"total_logins_today={logins}")

        sessions = data.get("active_sessions", 0)
        self.record("活躍 sessions >= 1", sessions >= 1, f"active_sessions={sessions}")

    def verify_activities(self):
        """檢查 2: 活動日誌"""
        print("\n[2/6] 驗證活動日誌...")
        resp = requests.get(
            f"{API_BASE_URL}/admin/audit/activities",
            params={"from": "2026-02-01", "to": "2026-02-28"},
            headers=self.headers()
        )
        if resp.status_code != 200:
            self.record("Activities API 回傳 200", False, f"status={resp.status_code}")
            return
        self.record("Activities API 回傳 200", True)

        data = resp.json()
        # 驗證 PaginatedResponse 格式
        paginated_fields = ["data", "total", "page", "per_page", "total_pages"]
        missing = [f for f in paginated_fields if f not in data]
        self.record("回傳 PaginatedResponse 格式", len(missing) == 0,
                     f"缺少: {missing}" if missing else "格式正確")

        activities = data.get("data", [])
        self.record("有活動紀錄", len(activities) > 0, f"共 {len(activities)} 筆")

        if activities:
            # 驗證活動資料欄位
            first = activities[0]
            log_fields = ["id", "event_category", "event_type", "created_at"]
            has_fields = all(f in first for f in log_fields)
            self.record("活動紀錄包含必要欄位", has_fields,
                         f"欄位: {list(first.keys())}")

            # 檢查是否有 PROTOCOL_* 事件
            protocol_events = [a for a in activities if "PROTOCOL" in a.get("event_type", "")]
            self.record("包含 PROTOCOL_* 事件", len(protocol_events) > 0,
                         f"共 {len(protocol_events)} 筆 Protocol 事件")

            # 列出事件類型分布
            event_types = {}
            for a in activities:
                et = a.get("event_type", "unknown")
                event_types[et] = event_types.get(et, 0) + 1
            print(f"    事件類型分布: {json.dumps(event_types, indent=2, ensure_ascii=False)}")

    def verify_logins(self):
        """檢查 3: 登入事件"""
        print("\n[3/6] 驗證登入事件...")
        resp = requests.get(
            f"{API_BASE_URL}/admin/audit/logins",
            params={"from": "2026-02-01", "to": "2026-02-28"},
            headers=self.headers()
        )
        if resp.status_code != 200:
            self.record("Logins API 回傳 200", False, f"status={resp.status_code}")
            return
        self.record("Logins API 回傳 200", True)

        data = resp.json()
        events = data.get("data", [])
        self.record("有登入事件", len(events) > 0, f"共 {len(events)} 筆")

        if events:
            first = events[0]
            login_fields = ["id", "email", "event_type", "created_at"]
            has_fields = all(f in first for f in login_fields)
            self.record("登入事件包含必要欄位", has_fields,
                         f"欄位: {list(first.keys())}")

            success_events = [e for e in events if e.get("event_type") == "login_success"]
            self.record("包含 login_success 事件", len(success_events) > 0,
                         f"共 {len(success_events)} 筆")

            # 打印異常欄位檢查
            unusual = [e for e in events if e.get("is_unusual_time") or e.get("is_unusual_location") or e.get("is_new_device") or e.get("is_mass_login")]
            print(f"    有異常標記的事件: {len(unusual)} 筆")

    def verify_sessions(self):
        """檢查 4: 活躍 Sessions"""
        print("\n[4/6] 驗證活躍 Sessions...")
        resp = requests.get(f"{API_BASE_URL}/admin/audit/sessions", headers=self.headers())
        if resp.status_code != 200:
            self.record("Sessions API 回傳 200", False, f"status={resp.status_code}")
            return
        self.record("Sessions API 回傳 200", True)

        data = resp.json()
        sessions = data.get("data", [])
        active = [s for s in sessions if s.get("is_active")]
        self.record("有活躍 Session", len(active) > 0, f"共 {len(active)} 個活躍")

        if active:
            first = active[0]
            session_fields = ["id", "user_name", "user_email", "started_at", "last_activity_at", "is_active"]
            has_fields = all(f in first for f in session_fields)
            self.record("Session 包含必要欄位", has_fields,
                         f"欄位: {list(first.keys())}")

    def verify_alerts(self):
        """檢查 5: 安全警報"""
        print("\n[5/6] 驗證安全警報...")
        resp = requests.get(f"{API_BASE_URL}/admin/audit/alerts", headers=self.headers())
        if resp.status_code != 200:
            self.record("Alerts API 回傳 200", False, f"status={resp.status_code}")
            return
        self.record("Alerts API 回傳 200", True)

        data = resp.json()
        alerts = data.get("data", [])
        print(f"    共 {len(alerts)} 筆安全警報")

        if alerts:
            first = alerts[0]
            alert_fields = ["id", "alert_type", "severity", "title", "status", "created_at"]
            has_fields = all(f in first for f in alert_fields)
            self.record("警報包含必要欄位", has_fields, f"欄位: {list(first.keys())}")

            open_alerts = [a for a in alerts if a.get("status") == "open"]
            print(f"    待處理: {len(open_alerts)} 筆, 已解決: {len(alerts) - len(open_alerts)} 筆")
        else:
            self.record("警報格式正確（目前無警報）", True, "空陣列是合理的")

    def verify_protocol_activities(self):
        """檢查 6: Protocol 活動紀錄完整性"""
        print("\n[6/6] 驗證 Protocol 活動紀錄完整性...")

        # 先取得最新的 protocol
        resp = requests.get(f"{API_BASE_URL}/protocols", headers=self.headers())
        if resp.status_code != 200:
            self.record("取得 Protocols 列表", False, f"status={resp.status_code}")
            return

        protocols = resp.json()
        if not protocols:
            self.record("存在 Protocol 可供驗證", False, "protocols 列表為空")
            return

        # 選最新的 APPROVED protocol
        approved = [p for p in protocols if p.get("status") == "APPROVED"]
        if not approved:
            target = protocols[0]
        else:
            target = approved[0]

        self.protocol_id = target["id"]
        print(f"    選用 Protocol: {target.get('protocol_no', 'N/A')} ({target['id'][:8]}...)")
        print(f"    狀態: {target.get('status')}, 標題: {target.get('title', 'N/A')[:40]}...")

        # 取得活動紀錄
        resp = requests.get(
            f"{API_BASE_URL}/protocols/{self.protocol_id}/activities",
            headers=self.headers()
        )
        if resp.status_code != 200:
            self.record("取得 Protocol Activities", False, f"status={resp.status_code}")
            return
        self.record("取得 Protocol Activities API 回傳 200", True)

        activities = resp.json()
        self.record("有活動紀錄", len(activities) > 0, f"共 {len(activities)} 筆")

        if activities:
            # 統計活動類型
            type_counts = {}
            for a in activities:
                at = a.get("activity_type", "unknown")
                type_counts[at] = type_counts.get(at, 0) + 1

            print(f"    活動類型分布:")
            for t, c in sorted(type_counts.items()):
                print(f"      - {t}: {c}")

            # 預期的關鍵活動類型（後端回傳大寫格式）
            expected_types = ["CREATED", "SUBMITTED", "STATUS_CHANGED"]
            found_types = list(type_counts.keys())
            for et in expected_types:
                has = et in found_types
                self.record(f"包含 {et} 活動", has,
                             f"{'找到' if has else '缺少'}")

            # 如果是 APPROVED，應有 APPROVED 活動
            if target.get("status") == "APPROVED":
                self.record("APPROVED 應有 APPROVED 活動", "APPROVED" in found_types)

    def print_summary(self):
        """列印驗證總結"""
        print("\n" + "=" * 60)
        print("驗證結果摘要")
        print("=" * 60)

        passed = sum(1 for r in self.results if r["passed"])
        total = len(self.results)
        failed = total - passed

        for r in self.results:
            s = "✓" if r["passed"] else "✗"
            print(f"  {s} {r['name']}")

        print(f"\n  Total: {total} | Passed: {passed} | Failed: {failed}")
        print("=" * 60)

        return failed == 0


def main():
    v = AuditVerifier()
    v.login()

    v.verify_dashboard()
    v.verify_activities()
    v.verify_logins()
    v.verify_sessions()
    v.verify_alerts()
    v.verify_protocol_activities()

    success = v.print_summary()

    # 輸出 JSON 結果
    with open("audit_verify_result.json", "w", encoding="utf-8") as f:
        json.dump({
            "total": len(v.results),
            "passed": sum(1 for r in v.results if r["passed"]),
            "failed": sum(1 for r in v.results if not r["passed"]),
            "details": v.results
        }, f, indent=2, ensure_ascii=False)
    print("\n結果已儲存至 audit_verify_result.json")

    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
