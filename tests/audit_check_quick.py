"""
快速操作日誌驗證腳本 - 查詢 admin/audit API 確認測試活動是否被正確記錄
"""
import sys
import json
import requests

sys.stdout.reconfigure(encoding="utf-8")

API = "http://localhost:8000/api"

# Login as admin
r = requests.post(f"{API}/auth/login", json={"email": "admin@ipig.local", "password": "admin123"})
if r.status_code != 200:
    print(f"Admin login failed: {r.status_code}")
    sys.exit(1)
token = r.json()["access_token"]
h = {"Authorization": f"Bearer {token}"}

print("=" * 70)
print("1. Dashboard 統計")
print("=" * 70)
r = requests.get(f"{API}/admin/audit/dashboard", headers=h)
if r.status_code == 200:
    d = r.json()
    for k, v in d.items():
        print(f"  {k}: {v}")
else:
    print(f"  FAILED: {r.status_code}")

print()
print("=" * 70)
print("2. 操作日誌 Activities (今日)")
print("=" * 70)
r = requests.get(
    f"{API}/admin/audit/activities",
    params={"from": "2026-02-14", "to": "2026-02-15", "per_page": 200},
    headers=h,
)
if r.status_code == 200:
    d = r.json()
    total = d.get("total", "?")
    print(f"  總筆數: {total}")
    activities = d.get("data", [])

    # Count by event_type
    type_counts = {}
    entity_counts = {}
    category_counts = {}
    for a in activities:
        et = a.get("event_type", "unknown")
        type_counts[et] = type_counts.get(et, 0) + 1
        ent = a.get("entity_type", "unknown")
        entity_counts[ent] = entity_counts.get(ent, 0) + 1
        cat = a.get("event_category", "unknown")
        category_counts[cat] = category_counts.get(cat, 0) + 1

    print(f"\n  === 事件類型分布 ===")
    for t, c in sorted(type_counts.items(), key=lambda x: -x[1]):
        print(f"    {t}: {c}")

    print(f"\n  === 實體類型分布 ===")
    for t, c in sorted(entity_counts.items(), key=lambda x: -x[1]):
        print(f"    {t}: {c}")

    print(f"\n  === 類別分布 ===")
    for t, c in sorted(category_counts.items(), key=lambda x: -x[1]):
        print(f"    {t}: {c}")

    # Print last 30 entries
    print(f"\n  --- 最近 30 筆活動 ---")
    for a in activities[:30]:
        desc = a.get("description", "-")
        if desc and len(desc) > 80:
            desc = desc[:80] + "..."
        entity_name = a.get("entity_name", "")
        entity_info = f" [{entity_name}]" if entity_name else ""
        print(
            f"    [{a.get('event_type', '')}] "
            f"{a.get('entity_type', '-')}{entity_info} | "
            f"user={a.get('user_email', '-')} | "
            f"{desc}"
        )
else:
    print(f"  FAILED: {r.status_code} {r.text[:200]}")

print()
print("=" * 70)
print("3. 登入事件 (今日)")
print("=" * 70)
r = requests.get(
    f"{API}/admin/audit/logins",
    params={"from": "2026-02-14", "to": "2026-02-15", "per_page": 100},
    headers=h,
)
if r.status_code == 200:
    d = r.json()
    events = d.get("data", [])
    total = d.get("total", "?")
    print(f"  總筆數: {total}")
    success = [e for e in events if e.get("event_type") == "login_success"]
    failed = [e for e in events if e.get("event_type") == "login_failed"]
    print(f"  成功登入: {len(success)}, 失敗登入: {len(failed)}")
    users = sorted(set(e.get("email", "") for e in success))
    print(f"  成功登入的使用者:")
    for u in users:
        print(f"    - {u}")
else:
    print(f"  FAILED: {r.status_code}")

print()
print("=" * 70)
print("4. 活躍 Sessions")
print("=" * 70)
r = requests.get(f"{API}/admin/audit/sessions", headers=h)
if r.status_code == 200:
    d = r.json()
    sessions = d.get("data", [])
    active = [s for s in sessions if s.get("is_active")]
    print(f"  總 sessions: {len(sessions)}, 活躍: {len(active)}")
    for s in active[:10]:
        print(f"    - {s.get('user_email', '?')} | started={s.get('started_at', '?')[:19]}")
else:
    print(f"  FAILED: {r.status_code}")

print()
print("=" * 70)
print("5. 安全警報")
print("=" * 70)
r = requests.get(f"{API}/admin/audit/alerts", headers=h)
if r.status_code == 200:
    d = r.json()
    alerts = d.get("data", [])
    print(f"  總警報: {len(alerts)}")
    for a in alerts[:5]:
        print(f"    - [{a.get('severity', '?')}] {a.get('title', '?')} | status={a.get('status', '?')}")
else:
    print(f"  FAILED: {r.status_code}")

print()
print("=" * 70)
print("驗證結論")
print("=" * 70)
