#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
重現：複製觀察紀錄後編輯無法儲存（500 資料庫操作失敗）

流程：
1. 建立動物 + 觀察紀錄
2. 複製該觀察紀錄
3. 編輯複製後的紀錄（PUT）
4. 驗證是否成功

用法：
    cd d:\\Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/test_reproduce_copy_edit_observation.py

需先啟動測試環境：
    docker compose -f docker-compose.test.yml up -d db-test api-test
"""

import os
import sys
from datetime import date

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL
from test_fixtures import get_users_for_roles, ANIMAL_ROLES

ANIMAL_TEST_USERS = get_users_for_roles(ANIMAL_ROLES)
_ROLE_ALIASES = {"VET": "VET_ANIMAL", "EXPERIMENT_STAFF": "EXPERIMENT_STAFF_ANIMAL"}


def main() -> int:
    t = BaseApiTester("複製後編輯觀察紀錄 — 重現測試")

    if not t.setup_test_users(ANIMAL_TEST_USERS):
        return 1
    if not t.login_all(ANIMAL_TEST_USERS):
        return 1
    for alias, real in _ROLE_ALIASES.items():
        if real in t.tokens:
            t.tokens[alias] = t.tokens[real]
        if real in t.user_ids:
            t.user_ids[alias] = t.user_ids[real]

    STAFF = "EXPERIMENT_STAFF"
    today = str(date.today())

    # Step 1: 取得或建立動物
    t.step("Step 1 — 取得測試動物")
    search = t._req("GET", f"{API_BASE_URL}/animals?keyword=001", role=STAFF)
    resp_data = search.json()
    animal_list = resp_data.get("data", []) if isinstance(resp_data, dict) else resp_data
    animals = [a for a in animal_list if isinstance(a, dict) and a.get("ear_tag") == "001"]
    if not animals:
        # 建立動物來源
        src = t._req("POST", f"{API_BASE_URL}/animal-sources", role=STAFF, json={
            "code": "REPRO-SRC",
            "name": "重現測試來源",
            "address": "測試",
            "contact": "測試",
        })
        source_id = src.json()["id"]
        # 建立動物
        create = t._req("POST", f"{API_BASE_URL}/animals", role=STAFF, json={
            "ear_tag": "001",
            "breed": "minipig",
            "gender": "male",
            "birth_date": today,
            "entry_date": today,
            "entry_weight": 20,
            "pen_location": "A-01",
            "source_id": source_id,
            "force_create": True,
        })
        animal_id = create.json()["id"]
    else:
        animal_id = animals[0]["id"]
    t.record("取得動物", True, f"animal_id={animal_id[:8]}...")

    # Step 2: 建立觀察紀錄
    t.step("Step 2 — 建立觀察紀錄")
    obs_payload = {
        "event_date": today,
        "record_type": "observation",
        "content": "兩個眼睛兩個鼻孔一個嘴巴四隻腳",
        "no_medication_needed": False,
        "treatments": [{"drug": "Atropine", "dosage": "10", "dosage_unit": "mg"}],
        "remark": "重現測試",
    }
    obs_resp = t._req("POST", f"{API_BASE_URL}/animals/{animal_id}/observations", role=STAFF, json=obs_payload)
    source_obs = obs_resp.json()
    source_obs_id = source_obs["id"]
    t.record("建立觀察紀錄", True, f"source_id={source_obs_id[:8]}...")

    # Step 3: 複製觀察紀錄
    t.step("Step 3 — 複製觀察紀錄")
    copy_resp = t._req("POST", f"{API_BASE_URL}/animals/{animal_id}/observations/copy", role=STAFF, json={
        "source_id": source_obs_id,
    })
    copied_obs = copy_resp.json()
    copied_obs_id = copied_obs["id"]
    t.record("複製觀察紀錄", True, f"copied_id={copied_obs_id[:8]}...")

    # Step 4: 編輯複製後的紀錄（PUT）
    t.step("Step 4 — 編輯複製後的紀錄 (PUT)")
    update_payload = {
        "event_date": today,
        "record_type": "observation",
        "content": "兩個（已編輯）",
        "no_medication_needed": False,
        "treatments": None,
        "remark": "複製後編輯測試",
    }
    try:
        # 使用 _req 但捕獲 500
        update_resp = t.session.request(
            "PUT",
            f"{API_BASE_URL}/observations/{copied_obs_id}",
            json=update_payload,
            headers=t.get_headers(STAFF),
        )
        t._refresh_csrf()
        for c in ("access_token", "refresh_token"):
            t.session.cookies.pop(c, None)

        if update_resp.status_code == 200:
            t.record("編輯複製紀錄成功", True, "PUT 回傳 200")
            # Step 5: 驗證版本歷史
            t.step("Step 5 — 驗證版本歷史")
            ver_resp = t._req("GET", f"{API_BASE_URL}/observations/{copied_obs_id}/versions", role=STAFF)
            ver_data = ver_resp.json()
            has_versions = ver_data.get("versions") and len(ver_data["versions"]) > 0
            t.record("版本歷史有資料", has_versions, f"versions={len(ver_data.get('versions', []))} 筆")
        else:
            err_body = update_resp.text[:300]
            t.record("編輯複製紀錄失敗", False, f"status={update_resp.status_code} body={err_body}")
            print(f"\n  ❌ 錯誤回應: {update_resp.status_code}")
            print(f"  Body: {update_resp.text[:500]}")
    except Exception as e:
        t.record("編輯複製紀錄異常", False, str(e))
        print(f"\n  ❌ 例外: {e}")

    t.print_summary()
    return 0 if all(r["success"] for r in t.results) else 1


if __name__ == "__main__":
    sys.exit(main())
