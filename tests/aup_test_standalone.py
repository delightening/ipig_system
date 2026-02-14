import os
import sys
import requests
import time
from dotenv import load_dotenv

# 修正 Windows 終端機的 Unicode 編碼問題 (cp950 -> utf-8)
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

load_dotenv()

API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8000/api")

# 管理員帳號（用於建立測試用戶）
ADMIN_CREDENTIALS = {"email": "jason4617987@gmail.com", "password": "kfknxJH6AjSvJh6?"}

# 測試帳號設定：角色代碼用來對應系統中的 role code
TEST_USERS = {
    "IACUC_STAFF": {"email": "staff_test@example.com", "password": "password123", "display_name": "IACUC Staff (Test)", "role_codes": ["IACUC_STAFF"]},
    "REVIEWER1":   {"email": "rev1_test@example.com",   "password": "password123", "display_name": "Reviewer 1 (Test)",  "role_codes": ["REVIEWER"]},
    "REVIEWER2":   {"email": "rev2_test@example.com",   "password": "password123", "display_name": "Reviewer 2 (Test)",  "role_codes": ["REVIEWER"]},
    "REVIEWER3":   {"email": "rev3_test@example.com",   "password": "password123", "display_name": "Reviewer 3 (Test)",  "role_codes": ["REVIEWER"]},
    "IACUC_CHAIR": {"email": "chair_test@example.com",  "password": "password123", "display_name": "IACUC Chair (Test)", "role_codes": ["REVIEWER", "IACUC_CHAIR"]},
    "PI":          {"email": "pi_test@example.com",     "password": "password123", "display_name": "Test PI",            "role_codes": ["PI"]},
    "VET":         {"email": "vet_test@example.com",    "password": "password123", "display_name": "Test Vet",           "role_codes": ["VET"]},
    "REV_OTHER":   {"email": "rev_other_test@example.com", "password": "password123", "display_name": "Reviewer Other (Test)", "role_codes": ["REVIEWER"]},
}


class AUPTester:
    def __init__(self):
        self.tokens = {}
        self.user_ids = {}
        self.protocol_id = None
        self.version_id = None

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
            print(f"    請確認管理員帳號 {ADMIN_CREDENTIALS['email']} 是否存在")
            return False
        # SEC-02：從 Set-Cookie header 提取 access_token
        admin_token = resp.cookies.get("access_token")
        if not admin_token:
            print(f"  ✗ 管理員登入成功但無法取得 access_token cookie")
            return False
        admin_headers = {"Authorization": f"Bearer {admin_token}"}
        print(f"  ✓ 管理員登入成功")

        # 2. 取得所有角色的 ID 對應表
        print("  [2/3] 讀取角色清單...")
        resp = requests.get(f"{API_BASE_URL}/roles", headers=admin_headers)
        if resp.status_code != 200:
            print(f"  ✗ 無法取得角色清單: {resp.status_code}")
            return False
        roles = resp.json()
        role_map = {}  # code -> id
        for r in roles:
            role_map[r["code"]] = r["id"]
        print(f"  ✓ 取得 {len(role_map)} 個角色: {list(role_map.keys())}")

        # 3. 逐一建立測試帳號
        print("  [3/3] 建立/確認測試帳號...")
        created = 0
        skipped = 0
        for role_label, user_info in TEST_USERS.items():
            # 先嘗試登入，若成功代表帳號已存在
            login_resp = requests.post(f"{API_BASE_URL}/auth/login", json={
                "email": user_info["email"],
                "password": user_info["password"]
            })
            if login_resp.status_code == 200:
                print(f"    ✓ {role_label:12s} ({user_info['email']}) — 已存在，跳過")
                skipped += 1
                continue

            # 帳號不存在，透過 API 建立
            role_ids = []
            for rc in user_info["role_codes"]:
                if rc in role_map:
                    role_ids.append(role_map[rc])
                else:
                    print(f"    ⚠ 角色 {rc} 在系統中不存在，略過")

            payload = {
                "email": user_info["email"],
                "password": user_info["password"],
                "display_name": user_info["display_name"],
                "role_ids": role_ids,
            }
            create_resp = requests.post(f"{API_BASE_URL}/users", json=payload, headers=admin_headers)
            if create_resp.status_code in (200, 201):
                print(f"    ✓ {role_label:12s} ({user_info['email']}) — 建立成功")
                created += 1
            else:
                print(f"    ✗ {role_label:12s} ({user_info['email']}) — 建立失敗: {create_resp.status_code} {create_resp.text}")

        print(f"\n  [結果] 新建 {created} 個 / 已存在 {skipped} 個 / 共 {len(TEST_USERS)} 個")
        return True

    # ========================================
    # 登入所有測試帳號
    # ========================================
    def login_all(self):
        print("\n" + "=" * 60)
        print("[Auth] 登入所有測試帳號...")
        print("=" * 60)
        success = 0
        for role, user_info in TEST_USERS.items():
            resp = requests.post(f"{API_BASE_URL}/auth/login", json={
                "email": user_info["email"],
                "password": user_info["password"]
            })
            if resp.status_code == 200:
                data = resp.json()
                self.tokens[role] = resp.cookies.get("access_token")
                self.user_ids[role] = data["user"]["id"]
                print(f"  ✓ {role:12s} 登入成功 (ID: {data['user']['id'][:8]}...)")
                success += 1
            else:
                print(f"  ✗ {role:12s} 登入失敗: {resp.status_code} {resp.text}")

        print(f"\n  [結果] {success}/{len(TEST_USERS)} 帳號登入成功")
        if success < len(TEST_USERS):
            raise RuntimeError(f"部分帳號登入失敗 ({success}/{len(TEST_USERS)})，無法繼續測試")

    def get_headers(self, role):
        return {"Authorization": f"Bearer {self.tokens[role]}"}

    def find_user_by_role(self, role_code):
        print(f"  [Search] Finding user with role: {role_code}...")
        resp = requests.get(f"{API_BASE_URL}/users", headers=self.get_headers("IACUC_STAFF"))
        if resp.status_code == 200:
            users = resp.json()
            for u in users:
                if role_code in u.get("roles", []):
                    print(f"  [Search] Found: {u['email']} (ID: {u['id']})")
                    return u["id"]
        return None

    def get_status(self):
        if not self.protocol_id: return "N/A"
        try:
            resp = requests.get(f"{API_BASE_URL}/protocols/{self.protocol_id}", headers=self.get_headers("IACUC_STAFF"))
            return resp.json().get("status", "Unknown")
        except:
            return "Error"

    def _req(self, method, url, **kwargs):
        resp = requests.request(method, url, **kwargs)
        if resp.status_code >= 400:
            error_data = {
                "method": method,
                "url": url,
                "status": resp.status_code,
                "payload": kwargs.get("json"),
                "response": resp.text,
                "current_protocol_status": self.get_status()
            }
            try:
                error_data["response_json"] = resp.json()
            except:
                pass
            
            import json
            with open("last_error.json", "w", encoding="utf-8") as f:
                json.dump(error_data, f, indent=2, ensure_ascii=False)
            
            print(f"\n[API ERROR] {method} {url} - Status: {resp.status_code}")
            print(f"Current AUP Status: {error_data['current_protocol_status']}")
            print(f"Details saved to last_error.json")
            
        print(f"  [{method}] {url.split('/')[-1]} -> {resp.status_code} (Status: {self.get_status()})")
        resp.raise_for_status()
        return resp

def run_flow():
    t = AUPTester()

    # Step 0: 建立測試帳號
    if not t.setup_test_users():
        raise RuntimeError("測試帳號建立失敗，無法繼續")

    # 登入所有帳號
    t.login_all()

    # 1. PI Creating Protocol
    print("\n" + "=" * 60)
    print("[Step 1] PI 建立計畫書...")
    print("=" * 60)
    full_content = {
        "basic": {
            "study_title": "AUP 完整流程極致驗證計畫 (Section 1-8)",
            "project_code": "FULL-TEST-2026",
            "project_type": "Research",
            "project_category": "Medicine",
            "is_glp": True,
            "apply_study_number": f"AUP-{int(time.time() * 1000) % 1000000}",
            "registration_authorities": ["FDA"],
            "pi_user_id": t.user_ids["PI"],
            "pi": {
                "name": "Test PI",
                "phone": "0912-345-678",
                "email": "pi_test@example.com",
                "address": "台北市南港區研究院路"
            },
            "sponsor": {
                "name": "生命科學研究基金會",
                "contact_person": "王大同",
                "contact_phone": "02-2789-0000",
                "contact_email": "sponsor@example.com"
            },
            "facility": {"id": "FAC-001", "title": "第一動物實驗中心"},
            "housing_location": "B1 無菌室"
        },
        "purpose": {
            "significance": "本研究旨在驗證 AUP 系統的穩定型與功能完整性。",
            "replacement": {
                "rationale": "目前無電腦模擬可替代。",
                "alt_search": {
                    "platforms": ["PubMed"],
                    "keywords": "AUP System, Lifecycle",
                    "conclusion": "尚無相關系統自動化測試之文獻。"
                }
            },
            "reduction": {"design": "使用統計學 Power Analysis 算出最小樣本數。"},
            "duplicate": {"experiment": False}
        },
        "items": {
            "use_test_item": True,
            "test_items": [{
                "name": "驗證劑-A", "form": "液態", "purpose": "標記",
                "storage_conditions": "4度C", "is_sterile": True
            }]
        },
        "design": {
            "procedures": "1. 每日觀察 2. 每週秤重 3. 最後進行麻醉與採樣。",
            "anesthesia": {
                "is_under_anesthesia": True,
                "anesthesia_type": "survival_surgery"
            },
            "pain": {
                "category": "C",
                "management_plan": "使用止痛藥緩解其不適感。"
            },
            "endpoints": {
                "experimental_endpoint": "計畫結束之動物處置方法。",
                "humane_endpoint": "若動物體重下降超過 20% 則進行安樂死。"
            }
        },
        "guidelines": {
            "content": "遵守實驗動物照顧及使用指引規範。",
            "references": [{"citation": "Guide for the Care and Use of Laboratory Animals", "url": "https://example.com"}]
        },
        "surgery": {
            "surgery_type": "無菌存活手術",
            "preop_preparation": "禁食 12 小時，皮膚除毛。",
            "surgery_description": "依照 SOP 進行腹腔切開。",
            "monitoring": "心跳、血氧、呼吸頻率。",
            "postop_care": "提供保溫設備與安靜環境。",
            "drugs": [{"drug_name": "抗生素", "dose": "10mg/kg", "route": "IM", "frequency": "QD", "purpose": "預防感染"}]
        },
        "animals": {
            "total_animals": 5,
            "animals": [{
                "species": "Pig", "strain": "L6", "sex": "MIXED", 
                "number": 5, "age_min": "8", "age_max": "10", "age_unlimited": False,
                "weight_min": "20", "weight_max": "30", "weight_unlimited": False,
                "housing_location": "B1-02"
            }]
        },
        "personnel": [{
            "name": "Test PI", "position": "教授", "years_experience": "15",
            "roles": ["計畫主持人", "實驗操作"],
            "trainings": ["動物實驗倫理培訓", "外科手術培訓"]
        }]
    }

    payload = {
        "title": f"AUP完整流程測試_{int(time.time())} +Final",
        "working_content": full_content,
        "start_date": "2026-05-01",
        "end_date": "2027-04-01",
        "pi_user_id": t.user_ids["PI"]
    }
    resp = t._req("POST", f"{API_BASE_URL}/protocols", json=payload, headers=t.get_headers("PI"))
    t.protocol_id = resp.json()["id"]
    
    # 指派工作人員 (Co-editor)
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/co-editors", json={
        "user_id": t.user_ids["IACUC_STAFF"], 
        "protocol_id": t.protocol_id
    }, headers=t.get_headers("IACUC_STAFF"))

    # PI Submit
    print("  - Submitting...")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI"))
    print("  - Status: SUBMITTED")

    # 2. Staff Pre-review
    print("\n[Step 2] STAFF 行政預審 (???)")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={"to_status": "PRE_REVIEW"}, headers=t.get_headers("IACUC_STAFF"))
    
    versions = t._req("GET", f"{API_BASE_URL}/protocols/{t.protocol_id}/versions", headers=t.get_headers("IACUC_STAFF")).json()
    v_id = versions[0]["id"]
    
    # Add ??? Comment
    comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", json={
        "protocol_version_id": v_id,
        "content": "???"
    }, headers=t.get_headers("IACUC_STAFF")).json()
    c_id = comment["id"]
    
    # 3. PI Replying !!!
    print("\n[Step 3] PI 回覆 (!!!)")
    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", json={
        "parent_comment_id": c_id,
        "content": "!!!"
    }, headers=t.get_headers("PI"))
    
    # 4. STAFF Revision Required -> PI title +1
    print("\n[Step 4] STAFF 要求修訂 -> PI 標題 +1")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={"to_status": "PRE_REVIEW_REVISION_REQUIRED", "remark": "請更正標題"}, headers=t.get_headers("IACUC_STAFF"))
    
    # Update title +1
    t._req("PUT", f"{API_BASE_URL}/protocols/{t.protocol_id}", json={
        "title": payload["title"] + " +1",
        "working_content": full_content
    }, headers=t.get_headers("PI"))
    
    # Resubmit
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI"))
    print("  - PI resubmitted with +1")

    # 5. Entering Vet Review (Medical)
    print("\n[Step 5] 醫療審查 (Medical)")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={"to_status": "PRE_REVIEW"}, headers=t.get_headers("IACUC_STAFF"))
    t._req("POST", f"{API_BASE_URL}/reviews/comments/{c_id}/resolve", headers=t.get_headers("IACUC_STAFF"))
    
    vet_id = t.user_ids["VET"]
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={"to_status": "VET_REVIEW", "vet_id": vet_id}, headers=t.get_headers("IACUC_STAFF"))
    
    vet_comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", json={
        "protocol_version_id": v_id,
        "content": "Medical assessment: The drug dosage seems high."
    }, headers=t.get_headers("VET")).json()
    vc_id = vet_comment["id"]
    
    # PI Replying
    print("  - PI Replying to Vet...")
    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", json={
        "parent_comment_id": vc_id,
        "content": "We have reduced the dosage in accordance with the GLP guidelines."
    }, headers=t.get_headers("PI"))
    
    # STAFF 退回 PI 修訂標題 +M
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={"to_status": "VET_REVISION_REQUIRED", "remark": "醫療意見修訂"}, headers=t.get_headers("IACUC_STAFF"))
    
    # Update title +M
    t._req("PUT", f"{API_BASE_URL}/protocols/{t.protocol_id}", json={
        "title": payload["title"] + " +M",
        "working_content": full_content
    }, headers=t.get_headers("PI"))
    
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI"))
    print("  - PI resubmitted with +M")

    # 6. Resolve Vet Comment
    print("\n[Step 6] 解決醫療審查意見")
    t._req("POST", f"{API_BASE_URL}/reviews/comments/{vc_id}/resolve", headers=t.get_headers("VET"))

    # 7. Prepare for Ethics Review
    print("\n[Step 7] 準備倫理審查 (指派委員)")
    reviewers = [t.user_ids["REVIEWER1"], t.user_ids["REVIEWER2"], t.user_ids["REVIEWER3"]]

    # 8. Staff Status Change (Must be at UNDER_REVIEW for comments)
    print("\n[Step 8] Staff 確認狀態 (UNDER_REVIEW)")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "UNDER_REVIEW",
        "reviewer_ids": reviewers
    }, headers=t.get_headers("IACUC_STAFF"))

    # 9. 3 Reviewers providing comments
    print("\n[Step 9] 3 名委員發表意見")
    # Note: re-fetch v_id as it might have changed
    v_id = t._req("GET", f"{API_BASE_URL}/protocols/{t.protocol_id}/versions", headers=t.get_headers("IACUC_STAFF")).json()[0]["id"]
    rev_cids = []
    for i, role in enumerate(["REVIEWER1", "REVIEWER2", "REVIEWER3"]):
        c = t._req("POST", f"{API_BASE_URL}/reviews/comments", json={
            "protocol_version_id": v_id,
            "content": f"Reviewer {i+1} comment on ethics."
        }, headers=t.get_headers(role)).json()
        rev_cids.append((role, c["id"]))

    # 10. Non-assigned reviewer providing comments
    print("\n[Step 10] 非指定委員發表意見 (REV_OTHER)")
    other_comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", json={
        "protocol_version_id": v_id,
        "content": "Other reviewer comment."
    }, headers=t.get_headers("REV_OTHER")).json()
    oc_id = other_comment["id"]

    # 11. PI Reply All !!!
    print("\n[Step 11] PI 回覆所有審查委員 (具有意義的內容)")
    replies = [
        "We have clarified the surgical procedure in Section 6.",
        "The animal housing conditions have been updated to meet the new standards.",
        "We have added more details about the post-operative monitoring plan."
    ]
    for i, (role, cid) in enumerate(rev_cids):
        t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", json={
            "parent_comment_id": cid,
            "content": replies[i]
        }, headers=t.get_headers("PI"))
    
    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", json={
        "parent_comment_id": oc_id,
        "content": "Thank you for the suggestion. We have included the requested reference."
    }, headers=t.get_headers("PI"))

    # 12. Staff set Revision Required
    print("\n[Step 12] Staff 要求修正計畫書 (REVISION_REQUIRED)")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={"to_status": "REVISION_REQUIRED"}, headers=t.get_headers("IACUC_STAFF"))

    # 13. Resolving comments and resubmitting
    print("\n[Step 13] 解決意見並重新提交")
    # Resolve all
    for role, cid in rev_cids:
        t._req("POST", f"{API_BASE_URL}/reviews/comments/{cid}/resolve", headers=t.get_headers(role))
    t._req("POST", f"{API_BASE_URL}/reviews/comments/{oc_id}/resolve", headers=t.get_headers("REV_OTHER"))
    
    # PI Update and Resubmit
    t._req("PUT", f"{API_BASE_URL}/protocols/{t.protocol_id}", json={
        "title": payload["title"] + " +Final",
        "working_content": payload["working_content"]
    }, headers=t.get_headers("PI"))
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI"))

    # 14. Chair Approve
    print("\n[Step 14] 主委最終核定")
    # 此時狀態是 RESUBMITTED (from Step 13), 轉為 UNDER_REVIEW
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "UNDER_REVIEW",
        "reviewer_ids": reviewers
    }, headers=t.get_headers("IACUC_STAFF"))

    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "APPROVED",
        "remark": "審核通過證明寄發。"
    }, headers=t.get_headers("IACUC_CHAIR"))
    
    print(f"\n{'=' * 60}")
    print(f"[SUCCESS] AUP 完整 14 步流程完成! Protocol ID: {t.protocol_id}")
    print(f"狀態已變更為: APPROVED")
    print(f"{'=' * 60}")


if __name__ == "__main__":
    try:
        run_flow()
    except Exception as e:
        print(f"\n[CRITICAL ERROR] Flow failed: {e}")
