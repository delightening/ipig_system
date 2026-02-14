"""
操作日誌深入驗證 - 不帶日期篩選查詢，確認是否有任何紀錄
"""
import sys
import json
import requests

sys.stdout.reconfigure(encoding="utf-8")

API = "http://localhost:8000/api"

r = requests.post(f"{API}/auth/login", json={"email": "jason4617987@gmail.com", "password": "kfknxJH6AjSvJh6?"})
if r.status_code != 200:
    print(f"Admin login failed: {r.status_code}")
    sys.exit(1)
token = r.cookies.get("access_token")  # SEC-02
h = {"Authorization": f"Bearer {token}"}

# 1. Activities 不帶日期篩選
print("=" * 70)
print("1. Activities (不帶日期篩選)")
print("=" * 70)
r = requests.get(f"{API}/admin/audit/activities", params={"per_page": 50}, headers=h)
if r.status_code == 200:
    d = r.json()
    total = d.get("total", "?")
    print(f"  總筆數: {total}")
    activities = d.get("data", [])
    print(f"  本頁: {len(activities)} 筆")

    type_counts = {}
    entity_counts = {}
    for a in activities:
        et = a.get("event_type", "unknown")
        type_counts[et] = type_counts.get(et, 0) + 1
        ent = a.get("entity_type", "unknown")
        entity_counts[ent] = entity_counts.get(ent, 0) + 1

    print(f"\n  === 事件類型分布 ===")
    for t, c in sorted(type_counts.items(), key=lambda x: -x[1]):
        print(f"    {t}: {c}")

    print(f"\n  === 實體類型分布 ===")
    for t, c in sorted(entity_counts.items(), key=lambda x: -x[1]):
        print(f"    {t}: {c}")

    print(f"\n  --- 最近 30 筆活動 ---")
    for a in activities[:30]:
        desc = a.get("description", "-")
        if desc and len(desc) > 100:
            desc = desc[:100] + "..."
        entity_name = a.get("entity_name", "")
        entity_info = f" [{entity_name}]" if entity_name else ""
        created = a.get("created_at", "?")[:19] if a.get("created_at") else "?"
        print(
            f"    {created} | [{a.get('event_type', '')}] "
            f"{a.get('entity_type', '-')}{entity_info} | "
            f"user={a.get('user_email', '-')} | "
            f"{desc}"
        )
else:
    print(f"  FAILED: {r.status_code} {r.text[:500]}")

# 2. Logins 不帶日期篩選
print()
print("=" * 70)
print("2. Login Events (不帶日期篩選)")
print("=" * 70)
r = requests.get(f"{API}/admin/audit/logins", params={"per_page": 50}, headers=h)
if r.status_code == 200:
    d = r.json()
    events = d.get("data", [])
    total = d.get("total", "?")
    print(f"  總筆數: {total}")
    print(f"  本頁: {len(events)} 筆")
    success = [e for e in events if e.get("event_type") == "login_success"]
    failed  = [e for e in events if e.get("event_type") == "login_failed"]
    print(f"  成功: {len(success)}, 失敗: {len(failed)}")
    for e in events[:20]:
        created = e.get("created_at", "?")[:19] if e.get("created_at") else "?"
        print(f"    {created} | {e.get('event_type', '')} | {e.get('email', '-')}")
else:
    print(f"  FAILED: {r.status_code}")

# 3. 直接查詢最近的 protocol 活動
print()
print("=" * 70)
print("3. Protocol Activities (最近的 protocol)")
print("=" * 70)
r = requests.get(f"{API}/protocols", headers=h)
if r.status_code == 200:
    protocols = r.json()
    if protocols:
        # 只取前幾個
        for p in protocols[:3]:
            pid = p["id"]
            pno = p.get("protocol_no", "?")
            pstatus = p.get("status", "?")
            print(f"\n  Protocol: {pno} ({pid[:8]}...) status={pstatus}")
            r2 = requests.get(f"{API}/protocols/{pid}/activities", headers=h)
            if r2.status_code == 200:
                acts = r2.json()
                print(f"    活動數: {len(acts)}")
                for a in acts[:10]:
                    created = a.get("created_at", "?")[:19] if a.get("created_at") else "?"
                    print(f"      {created} | {a.get('activity_type', '')} | {a.get('user_name', '-')} | {a.get('description', '-')[:60]}")
            else:
                print(f"    FAILED: {r2.status_code}")
    else:
        print("  沒有 protocols")
else:
    print(f"  FAILED: {r.status_code}")

# 4. 檢查 blood_test 相關日誌
print()
print("=" * 70)
print("4. Blood Test 活動日誌 (entity_type=blood_test)")
print("=" * 70)
r = requests.get(
    f"{API}/admin/audit/activities",
    params={"entity_type": "blood_test", "per_page": 20},
    headers=h,
)
if r.status_code == 200:
    d = r.json()
    total = d.get("total", "?")
    print(f"  blood_test 總筆數: {total}")
    for a in d.get("data", [])[:10]:
        desc = a.get("description", "-")
        if desc and len(desc) > 80:
            desc = desc[:80] + "..."
        created = a.get("created_at", "?")[:19] if a.get("created_at") else "?"
        print(f"    {created} | [{a.get('event_type', '')}] | {a.get('user_email', '-')} | {desc}")
else:
    print(f"  FAILED: {r.status_code}")

# 5. 各 entity_type 的日誌
print()
print("=" * 70)
print("5. 各 entity_type 活動日誌")
print("=" * 70)
for etype in ["user", "protocol", "pig", "blood_test", "blood_test_panel", "warehouse", "product", "document", "hr_leave", "hr_overtime"]:
    r = requests.get(
        f"{API}/admin/audit/activities",
        params={"entity_type": etype, "per_page": 5},
        headers=h,
    )
    if r.status_code == 200:
        d = r.json()
        total = d.get("total", "?")
        print(f"  {etype}: {total} 筆")
    else:
        print(f"  {etype}: FAILED ({r.status_code})")
