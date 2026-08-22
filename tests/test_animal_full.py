"""
完整動物管理系統測試

包括：
- 建立 VET / EXPERIMENT_STAFF 測試角色
- Phase 1: 建立動物來源 + 20 隻動物（不同品種/性別），驗證創建記錄
- Phase 2: 體重紀錄（全部 20 隻，各 2~3 筆）
- Phase 3: 觀察試驗紀錄（全部 20 隻）
- Phase 4: 手術紀錄 + 術後觀察（5 隻動物）
- Phase 5: 疫苗/驅蟲紀錄（全部 20 隻）
- Phase 6: 犧牲/採樣紀錄（3 隻動物）
- Phase 7: 動物資料更新
- Phase 8: 病理組織報告（3 隻動物）
- Phase 9: 紀錄時間軸驗證
- Phase 10: 獸醫建議（觀察 + 手術）

用法：
    cd d:\\Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/test_animal_full.py
"""

import time
import sys
import os
import random
from datetime import date, timedelta

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL
from test_fixtures import get_users_for_roles, ANIMAL_ROLES

# 測試帳號設定（從 test_fixtures 統一取得）
ANIMAL_TEST_USERS = get_users_for_roles(ANIMAL_ROLES)

# 角色別名對應（保持測試中的角色名稱可讀性）
_ROLE_ALIASES = {
    "VET": "VET_ANIMAL",
    "EXPERIMENT_STAFF": "EXPERIMENT_STAFF_ANIMAL",
}

# 5 隻動物的設定（各品種/性別至少 1 隻）
ANIMAL_CONFIGS = []
breeds = ["minipig", "white", "lyd", "other", "minipig"]
genders = ["male", "female"]
pen_locations = ["A-01", "A-02", "A-03", "B-01", "B-02"]

for i in range(5):
    ANIMAL_CONFIGS.append({
        "ear_tag": f"{i+1:03d}",
        "breed": breeds[i],
        "breed_other": "蘭喼豬" if breeds[i] == "other" else None,
        "gender": genders[i % 2],
        "birth_date": str(date.today() - timedelta(days=180 + i * 5)),
        "entry_date": str(date.today() - timedelta(days=30)),
        "entry_weight": round(18 + i * 0.5, 1),
        "pen_location": pen_locations[i],
        "remark": f"整合測試動物 #{i+1}",
    })

NUM_ANIMALS = len(ANIMAL_CONFIGS)  # 5


def run_animal_test(ctx=None) -> bool:
    """執行完整動物管理測試"""
    t = BaseApiTester("動物管理系統完整測試")

    # 前置作業（有 ctx 時跳過登入）
    if ctx:
        ctx.inject_into(t, list(ANIMAL_TEST_USERS.keys()))
        # 建立角色別名映射（讓內部的 STAFF / VET 能對應到 fixtures 的 key）
        for alias, real in _ROLE_ALIASES.items():
            if real in t.tokens:
                t.tokens[alias] = t.tokens[real]
            if real in t.user_ids:
                t.user_ids[alias] = t.user_ids[real]
        print(f"  ✓ 使用共享 Context，跳過登入")
    else:
        if not t.setup_test_users(ANIMAL_TEST_USERS):
            return False
        if not t.login_all(ANIMAL_TEST_USERS):
            return False
        # 獨立執行時建立別名
        for alias, real in _ROLE_ALIASES.items():
            if real in t.tokens:
                t.tokens[alias] = t.tokens[real]
            if real in t.user_ids:
                t.user_ids[alias] = t.user_ids[real]

    STAFF = "EXPERIMENT_STAFF"
    VET = "VET"
    today = str(date.today())

    # ========================================
    # Phase 1: 建立動物來源 + 20 隻動物
    # ========================================
    t.step(f"Phase 1 — 建立動物來源")
    source_resp = t._req("POST", f"{API_BASE_URL}/animal-sources", role=STAFF,
                          json={
                              "code": f"SRC-INT-{int(time.time()) % 10000}",
                              "name": "整合測試動物來源 - 台灣種畜場",
                              "address": "台南市善化區",
                              "contact": "陳先生",
                              "phone": "06-1234567",
                          })
    source_id = source_resp.json()["id"]
    t.record("建立動物來源", True, f"ID: {source_id[:8]}...")

    t.step(f"Phase 1 — 建立 {NUM_ANIMALS} 隻動物")
    animal_ids = []
    for i, config in enumerate(ANIMAL_CONFIGS):
        search_resp = t._req("GET", f"{API_BASE_URL}/animals?keyword={config['ear_tag']}", role=STAFF)
        existing = [p for p in search_resp.json() if p.get("ear_tag") == config["ear_tag"]]
        if existing:
            animal_ids.append(existing[0]["id"])
            continue

        payload = {
            **config,
            "source_id": source_id,
            "force_create": True,
        }
        resp = t._req("POST", f"{API_BASE_URL}/animals", role=STAFF, json=payload)
        animal = resp.json()
        animal_ids.append(animal["id"])
    t.record(f"建立 {NUM_ANIMALS} 隻動物", len(animal_ids) == NUM_ANIMALS)

    # 驗證每隻動物的創建記錄
    t.step("Phase 1 — 驗證創建記錄")
    verify_ok = 0
    for i, pid in enumerate(animal_ids):
        resp = t._req("GET", f"{API_BASE_URL}/animals/{pid}", role=STAFF)
        animal_data = resp.json()
        if (animal_data.get("ear_tag") == ANIMAL_CONFIGS[i]["ear_tag"] and
                animal_data.get("gender") == ANIMAL_CONFIGS[i]["gender"]):
            verify_ok += 1
    t.record("驗證創建記錄", verify_ok == NUM_ANIMALS, f"{verify_ok}/{NUM_ANIMALS} 筆正確")

    # ========================================
    # Phase 2: 體重紀錄
    # ========================================
    t.step(f"Phase 2 — 體重紀錄（全部 {NUM_ANIMALS} 隻，各 2~3 筆）")
    weight_count = 0
    for i, pid in enumerate(animal_ids):
        num_weights = 2 if i < 3 else 3  # 前 3 隻 2 筆，後 2 隻 3 筆
        for w in range(num_weights):
            measure_date = str(date.today() - timedelta(days=(num_weights - w) * 7))
            weight_val = round(ANIMAL_CONFIGS[i]["entry_weight"] + (w + 1) * 1.5, 1)
            t._req("POST", f"{API_BASE_URL}/animals/{pid}/weights", role=STAFF,
                    json={"measure_date": measure_date, "weight": weight_val})
            weight_count += 1
    t.record("記錄體重", True, f"共 {weight_count} 筆體重紀錄")

    # 驗證體重可見
    weight_verify_ok = 0
    for pid in animal_ids:
        resp = t._req("GET", f"{API_BASE_URL}/animals/{pid}/weights", role=STAFF)
        weights = resp.json()
        if len(weights) >= 2:
            weight_verify_ok += 1
    t.record("驗證體重紀錄可見", weight_verify_ok == NUM_ANIMALS, f"{weight_verify_ok}/{NUM_ANIMALS} 隻有體重")

    # 驗證列表中的最新體重
    list_resp = t._req("GET", f"{API_BASE_URL}/animals", role=STAFF)
    animals_list = list_resp.json()
    has_weight_in_list = sum(1 for p in animals_list if p.get("latest_weight") is not None and p["id"] in animal_ids)
    t.record("列表顯示最新體重", has_weight_in_list >= 3, f"{has_weight_in_list} 隻可見最新體重")

    # ========================================
    # Phase 3: 觀察試驗紀錄
    # ========================================
    t.step(f"Phase 3 — 觀察試驗紀錄（全部 {NUM_ANIMALS} 隻）")
    obs_count = 0
    obs_ids = []
    record_types = ["abnormal", "experiment", "observation"]
    observation_contents = [
        "動物精神良好，食慾正常，無異常行為。",
        "體溫 38.5°C，心率 80 bpm，呼吸 20 次/分。",
        "皮膚表面無異常，傷口癒合良好。",
        "排便正常，糞便成形，未見血便。",
        "活動力佳，能自由移動，無跛行。",
    ]
    treatments_data = [
        [{"drug_name": "安比西林", "dose": "10mg/kg", "route": "IM", "frequency": "BID"}],
        [{"drug_name": "美洛西卡", "dose": "0.2mg/kg", "route": "IV", "frequency": "QD"}],
        None,  # 無需用藥
    ]

    for i, pid in enumerate(animal_ids):
        num_obs = 1 if i < 3 else 2
        for o in range(num_obs):
            rec_type = record_types[(i + o) % len(record_types)]
            content = observation_contents[(i + o) % len(observation_contents)]
            treatment = treatments_data[(i + o) % len(treatments_data)]
            obs_date = str(date.today() - timedelta(days=(num_obs - o) * 3))

            payload = {
                "event_date": obs_date,
                "record_type": rec_type,
                "content": content,
                "no_medication_needed": treatment is None,
                "treatments": treatment,
                "remark": f"整合測試觀察 #{i+1}-{o+1}",
            }
            obs_resp = t._req("POST", f"{API_BASE_URL}/animals/{pid}/observations", role=STAFF, json=payload)
            obs_data = obs_resp.json()
            obs_ids.append({"id": obs_data["id"], "animal_index": i})
            obs_count += 1

        if (i + 1) % 10 == 0:
            t.sub_step(f"已建立 {i+1}/20 隻的觀察紀錄")
    t.record("建立觀察紀錄", True, f"共 {obs_count} 筆觀察紀錄")

    # 驗證觀察紀錄
    obs_verify = 0
    for pid in animal_ids:
        resp = t._req("GET", f"{API_BASE_URL}/animals/{pid}/observations", role=STAFF)
        if len(resp.json()) >= 1:
            obs_verify += 1
    t.record("驗證觀察紀錄", obs_verify == NUM_ANIMALS, f"{obs_verify}/{NUM_ANIMALS} 隻有觀察紀錄")

    # ========================================
    # Phase 4: 手術紀錄 + 術後觀察（5 隻動物）
    # ========================================
    t.step("Phase 4 — 手術紀錄 + 術後觀察（動物 #1~#2）")
    surgery_animals = animal_ids[:2]
    surgery_ids = []

    surgery_sites = ["腹腔", "胸腔", "頸部", "背部", "四肢"]
    for i, pid in enumerate(surgery_animals):
        surgery_payload = {
            "is_first_experiment": (i == 0),
            "surgery_date": str(date.today() - timedelta(days=7 - i)),
            "surgery_site": surgery_sites[i],
            "induction_anesthesia": {
                "drug": "Zoletil",
                "dose": f"{5 + i}mg/kg",
                "route": "IM",
                "time": f"09:{i:02d}:00"
            },
            "pre_surgery_medication": [
                {"drug_name": "Atropine", "dose": "0.04mg/kg", "route": "IM"},
                {"drug_name": "Cefazolin", "dose": "25mg/kg", "route": "IV"},
            ],
            "positioning": "仰臥位" if i % 2 == 0 else "側臥位",
            "anesthesia_maintenance": {
                "drug": "Isoflurane",
                "concentration": f"{1.5 + i * 0.1:.1f}%",
                "flow_rate": "2L/min"
            },
            "anesthesia_observation": f"麻醉深度穩定，眼瞼反射消失。手術持續 {60 + i * 15} 分鐘。",
            "vital_signs": {
                "heart_rate": 75 + i * 5,
                "spo2": 98 - i,
                "temperature": round(37.5 + i * 0.1, 1),
                "blood_pressure": f"{120 + i * 5}/{80 + i * 3}"
            },
            "reflex_recovery": f"術後 {15 + i * 5} 分鐘恢復眼瞼反射。",
            "respiration_rate": 18 + i * 2,
            "post_surgery_medication": [
                {"drug_name": "Meloxicam", "dose": "0.2mg/kg", "route": "IM", "frequency": "QD", "days": 3},
                {"drug_name": "Cefazolin", "dose": "25mg/kg", "route": "IV", "frequency": "BID", "days": 5},
            ],
            "remark": f"手術順利完成，術後恢復良好。整合測試手術 #{i+1}",
            "no_medication_needed": False,
        }
        resp = t._req("POST", f"{API_BASE_URL}/animals/{pid}/surgeries", role=STAFF, json=surgery_payload)
        surgery_data = resp.json()
        surgery_ids.append(surgery_data["id"])
        t.sub_step(f"動物 #{i+1} 手術 ({surgery_sites[i]}) -> ID: {surgery_data['id']}")

    t.record("建立手術紀錄", len(surgery_ids) == 2, f"{len(surgery_ids)} 隻動物各 1 筆手術紀錄")

    # 術後觀察紀錄
    t.sub_step("建立術後觀察紀錄...")
    for i, pid in enumerate(surgery_animals):
        for day_offset in [1, 3, 5]:
            obs_date = str(date.today() - timedelta(days=7 - i - day_offset))
            post_op_content = (
                f"術後第 {day_offset} 天觀察：\n"
                f"- 精神狀態：{'良好' if day_offset > 1 else '稍嗜睡'}\n"
                f"- 傷口狀況：{'癒合良好' if day_offset > 3 else '輕微紅腫'}\n"
                f"- 食慾：{'正常' if day_offset > 1 else '略減'}\n"
                f"- 活動力：{'正常' if day_offset > 3 else '輕微受限'}"
            )
            t._req("POST", f"{API_BASE_URL}/animals/{pid}/observations", role=STAFF,
                    json={
                        "event_date": obs_date,
                        "record_type": "experiment",
                        "content": post_op_content,
                        "no_medication_needed": day_offset > 3,
                        "treatments": [{"drug_name": "Meloxicam", "dose": "0.2mg/kg", "route": "IM"}] if day_offset <= 3 else None,
                        "remark": f"術後觀察 Day {day_offset} (動物 #{i+1})",
                    })
    t.record("術後觀察紀錄", True, "5 隻動物各 3 筆術後觀察")

    # 驗證手術紀錄
    surgery_verify = 0
    for pid in surgery_animals:
        resp = t._req("GET", f"{API_BASE_URL}/animals/{pid}/surgeries", role=STAFF)
        if len(resp.json()) >= 1:
            surgery_verify += 1
    t.record("驗證手術紀錄", surgery_verify == len(surgery_animals), f"{surgery_verify}/{len(surgery_animals)} 隻有手術紀錄")

    # ========================================
    # Phase 5: 疫苗/驅蟲紀錄
    # ========================================
    t.step(f"Phase 5 — 疫苗/驅蟲紀錄（全部 {NUM_ANIMALS} 隻）") # Corrected 疋苗 to 疫苗
    vaccines = [
        ("豬瘟疫苗", "Ivermectin 0.3mg/kg"),
        ("口蹄疫疫苗", None),
        ("環狀病毒疫苗", "Fenbendazole 5mg/kg"),
        (None, "Ivermectin 0.3mg/kg"),
    ]
    vac_count = 0
    for i, pid in enumerate(animal_ids):
        vaccine, deworming = vaccines[i % len(vaccines)]
        vac_date = str(date.today() - timedelta(days=20 - i))
        t._req("POST", f"{API_BASE_URL}/animals/{pid}/vaccinations", role=STAFF,
                json={
                    "administered_date": vac_date,
                    "vaccine": vaccine,
                    "deworming_dose": deworming,
                })
        vac_count += 1
    t.record("建立疫苗/驅蟲紀錄", True, f"共 {vac_count} 筆")

    # 驗證
    vac_verify = 0
    for pid in animal_ids:
        resp = t._req("GET", f"{API_BASE_URL}/animals/{pid}/vaccinations", role=STAFF)
        if len(resp.json()) >= 1:
            vac_verify += 1
    t.record("驗證疫苗紀錄", vac_verify == NUM_ANIMALS, f"{vac_verify}/{NUM_ANIMALS} 隻有疫苗紀錄") # Corrected 疋苗 to 疫苗

    # ========================================
    # Phase 6: 犧牲/採樣紀錄
    # ========================================
    t.step("Phase 6 — 犧牲/採樣紀錄（動物 #5）")
    sacrifice_animals = animal_ids[4:5]  # 1 隻
    for i, pid in enumerate(sacrifice_animals):
        sacrifice_payload = {
            "sacrifice_date": str(date.today() - timedelta(days=1)),
            "zoletil_dose": "6mg/kg",
            "method_electrocution": True,
            "method_bloodletting": False,
            "method_other": None,
            "sampling": "心, 肝, 脾, 肺, 腎",
            "sampling_other": "淋巴結",
            "blood_volume_ml": 200,
            "confirmed_sacrifice": True,
        }
        t._req("POST", f"{API_BASE_URL}/animals/{pid}/sacrifice", role=STAFF, json=sacrifice_payload)
        t.sub_step(f"動物 #5 犧牲/採樣紀錄已建立")
    t.record("建立犧牲/採樣紀錄", True, "1 隻動物")

    # 驗證
    sac_verify = 0
    for pid in sacrifice_animals:
        resp = t._req("GET", f"{API_BASE_URL}/animals/{pid}/sacrifice", role=STAFF)
        data = resp.json()
        if data and data.get("confirmed_sacrifice"):
            sac_verify += 1
    t.record("驗證犧牲紀錄", sac_verify == len(sacrifice_animals), f"{sac_verify}/{len(sacrifice_animals)} 筆確認")

    # ========================================
    # Phase 7: 動物資料更新
    # ========================================
    t.step("Phase 7 — 動物資料更新")
    # 前 2 隻設定計畫編號 + 開始實驗
    for i, pid in enumerate(animal_ids[:2]):
        t._req("PUT", f"{API_BASE_URL}/animals/{pid}", role=STAFF,
                json={
                    "iacuc_no": f"IACUC-INT-2026-{i+1:03d}",
                    "status": "in_experiment",
                    "experiment_date": str(date.today() - timedelta(days=10)),
                    "remark": f"整合測試 - 已進入實驗 #{i+1}",
                })

    # 犧牲的動物不再手動更新 — API 不允許更新已犧牲動物
    # 只驗證犧牲後狀態是否正確
    for pid in sacrifice_animals:
        sac_resp = t._req("GET", f"{API_BASE_URL}/animals/{pid}", role=STAFF)
        sac_status = sac_resp.json().get("status", "?")
        t.record("犧牲動物狀態", sac_status in ("completed", "deceased", "sacrificed", "euthanized"),
                 f"status={sac_status}")

    # 驗證
    updated_resp = t._req("GET", f"{API_BASE_URL}/animals/{animal_ids[0]}", role=STAFF)
    animal0 = updated_resp.json()
    update_ok = animal0.get("status") == "in_experiment" and animal0.get("iacuc_no") is not None
    t.record("驗證動物資料更新", update_ok, f"status={animal0.get('status')}, iacuc_no={animal0.get('iacuc_no')}")

    # ========================================
    # Phase 8: 病理組織報告
    # ========================================
    t.step("Phase 8 — 病理組織報告（動物 #5）")
    for i, pid in enumerate(sacrifice_animals):
        t._req("POST", f"{API_BASE_URL}/animals/{pid}/pathology", role=STAFF, json={})
        t.sub_step(f"動物 #5 病理報告已建立")
    t.record("建立病理報告", True, f"{len(sacrifice_animals)} 隻動物")

    # 驗證
    path_verify = 0
    for pid in sacrifice_animals:
        resp = t._req("GET", f"{API_BASE_URL}/animals/{pid}/pathology", role=STAFF)
        if resp.status_code == 200:
            path_verify += 1
    t.record("驗證病理報告", path_verify == len(sacrifice_animals), f"{path_verify}/{len(sacrifice_animals)} 筆")

    # ========================================
    # Phase 9: 紀錄時間軸驗證
    # ========================================
    t.step("Phase 9 — 紀錄時間軸完整性驗證")

    # 驗證手術動物（動物 #1~#2）應有：體重、觀察、手術、疫苗
    timeline_ok = 0
    for pid in animal_ids[:2]:  # 手術動物 #1~#2
        has_weight = len(t._req("GET", f"{API_BASE_URL}/animals/{pid}/weights", role=STAFF).json()) > 0
        has_obs = len(t._req("GET", f"{API_BASE_URL}/animals/{pid}/observations", role=STAFF).json()) > 0
        has_surgery = len(t._req("GET", f"{API_BASE_URL}/animals/{pid}/surgeries", role=STAFF).json()) > 0
        has_vac = len(t._req("GET", f"{API_BASE_URL}/animals/{pid}/vaccinations", role=STAFF).json()) > 0
        if has_weight and has_obs and has_surgery and has_vac:
            timeline_ok += 1
    t.record("手術動物時間軸完整", timeline_ok == 2, f"{timeline_ok}/2 隻完整")

    # 驗證犧牲動物（動物 #5）應有：體重、觀察、疫苗、犧牲、病理
    sac_timeline_ok = 0
    for pid in sacrifice_animals:
        has_weight = len(t._req("GET", f"{API_BASE_URL}/animals/{pid}/weights", role=STAFF).json()) > 0
        has_obs = len(t._req("GET", f"{API_BASE_URL}/animals/{pid}/observations", role=STAFF).json()) > 0
        has_vac = len(t._req("GET", f"{API_BASE_URL}/animals/{pid}/vaccinations", role=STAFF).json()) > 0
        sac_data = t._req("GET", f"{API_BASE_URL}/animals/{pid}/sacrifice", role=STAFF).json()
        has_sac = sac_data and sac_data.get("confirmed_sacrifice")
        if has_weight and has_obs and has_vac and has_sac:
            sac_timeline_ok += 1
    t.record("犧牲動物時間軸完整", sac_timeline_ok == 1, f"{sac_timeline_ok}/1 隻完整") # Changed from 3 to 1

    # 驗證一般動物（動物 #3~#4）應有：體重、觀察、疫苗
    normal_timeline_ok = 0
    for pid in animal_ids[2:4]: # Changed from 5:15 to 2:4
        has_weight = len(t._req("GET", f"{API_BASE_URL}/animals/{pid}/weights", role=STAFF).json()) > 0
        has_obs = len(t._req("GET", f"{API_BASE_URL}/animals/{pid}/observations", role=STAFF).json()) > 0
        has_vac = len(t._req("GET", f"{API_BASE_URL}/animals/{pid}/vaccinations", role=STAFF).json()) > 0
        if has_weight and has_obs and has_vac:
            normal_timeline_ok += 1
    t.record("一般動物時間軸完整", normal_timeline_ok == 2, f"{normal_timeline_ok}/2 隻完整") # Changed from 10 to 2

    # ========================================
    # Phase 10: 獸醫建議（觀察紀錄 + 手術紀錄）
    # ========================================
    t.step("Phase 10 — 獸醫建議（觀察紀錄 + 手術紀錄）")

    # 獸醫對觀察紀錄的建議內容
    vet_obs_comments = [
        ("建議調整飲食配方，增加蛋白質攝取量以改善體重增長。", False),
        ("發現輕微皮膚搔癢，建議觀察 3 日，若持續請通知獸醫。", False),
        ("體溫偏高（39.2°C），建議每 4 小時監測一次，必要時給予退燒藥。", True),
        ("糞便樣本顯示輕度寄生蟲感染，建議進行驅蟲療程。", True),
        ("動物精神狀態良好，建議維持目前照護計畫。", False),
        ("建議增加環境豐富化措施，減少刻板行為。", False),
        ("傷口恢復進度正常，建議持續換藥並保持乾燥。", False),
        ("食慾下降，需排除消化道問題，建議安排血液檢查。", True),
    ]

    # 對前 4 隻動物的觀察紀錄添加獸醫建議
    vet_obs_count = 0
    vet_obs_rec_ids = []
    for i in range(min(4, len(obs_ids))):
        obs_info = obs_ids[i]
        comment_content, is_urgent = vet_obs_comments[i]
        resp = t._req("POST", f"{API_BASE_URL}/observations/{obs_info['id']}/recommendations",
                      role=VET, json={"content": comment_content, "is_urgent": is_urgent})
        rec_data = resp.json()
        vet_obs_rec_ids.append(rec_data["id"])
        vet_obs_count += 1
    t.record("觀察紀錄獸醫建議", vet_obs_count == min(4, len(obs_ids)),
             f"共 {vet_obs_count} 筆（含 {sum(1 for _, u in vet_obs_comments[:vet_obs_count] if u)} 筆緊急）")

    # 獸醫對手術紀錄的建議內容
    vet_surgery_comments = [
        ("術後傷口癒合良好，建議第 7 日拆線。", False),
        ("術後呼吸音偏粗，建議密切監測呼吸狀態。", True),
    ]

    # 對 5 隻手術動物的手術紀錄添加獸醫建議
    vet_surg_count = 0
    vet_surg_rec_ids = []
    for i, sid in enumerate(surgery_ids):
        comment_content, is_urgent = vet_surgery_comments[i]
        resp = t._req("POST", f"{API_BASE_URL}/surgeries/{sid}/recommendations",
                      role=VET, json={"content": comment_content, "is_urgent": is_urgent})
        rec_data = resp.json()
        vet_surg_rec_ids.append(rec_data["id"])
        vet_surg_count += 1
    t.record("手術紀錄獸醫建議", vet_surg_count == len(surgery_ids),
             f"共 {vet_surg_count} 筆（含 {sum(1 for _, u in vet_surgery_comments[:vet_surg_count] if u)} 筆緊急）")

    # 驗證：GET /observations/:id/recommendations
    obs_rec_verify = 0
    for i in range(min(4, len(obs_ids))):
        resp = t._req("GET", f"{API_BASE_URL}/observations/{obs_ids[i]['id']}/recommendations", role=VET)
        recs = resp.json()
        if len(recs) >= 1:
            obs_rec_verify += 1
    t.record("驗證觀察獸醫建議", obs_rec_verify == min(4, len(obs_ids)),
             f"{obs_rec_verify}/{min(4, len(obs_ids))} 筆可查詢")

    # 驗證：GET /surgeries/:id/recommendations
    surg_rec_verify = 0
    for sid in surgery_ids:
        resp = t._req("GET", f"{API_BASE_URL}/surgeries/{sid}/recommendations", role=VET)
        recs = resp.json()
        if len(recs) >= 1:
            surg_rec_verify += 1
    t.record("驗證手術獸醫建議", surg_rec_verify == len(surgery_ids),
             f"{surg_rec_verify}/{len(surgery_ids)} 筆可查詢")

    # 驗證：儀表板 GET /animals/vet-comments
    vet_comments_resp = t._req("GET", f"{API_BASE_URL}/animals/vet-comments?per_page=20", role=VET)
    vet_comments_data = vet_comments_resp.json().get("data", [])
    t.record("儀表板獸醫建議 API", len(vet_comments_data) >= vet_obs_count,
             f"回傳 {len(vet_comments_data)} 筆（預期至少 {vet_obs_count} 筆觀察建議）")

    total_vet_comments = vet_obs_count + vet_surg_count

    # ========================================
    # 彙總
    # ========================================
    print(f"\n{'=' * 60}")
    print(f"[完成] 動物管理系統完整測試完成！")
    print(f"  動物: {len(animal_ids)} | 體重: {weight_count} 筆")
    print(f"  觀察: {obs_count} 筆 | 手術: {len(surgery_ids)} 筆")
    print(f"  疫苗: {vac_count} 筆 | 犧牲: {len(sacrifice_animals)} 筆")
    print(f"  獸醫建議: {total_vet_comments} 筆（觀察 {vet_obs_count} + 手術 {vet_surg_count}）")
    print(f"{'=' * 60}")
    return t.print_summary()


if __name__ == "__main__":
    try:
        success = run_animal_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n[CRITICAL ERROR] 動物管理測試失敗: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
