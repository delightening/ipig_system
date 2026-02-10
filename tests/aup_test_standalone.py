import os
import requests
import time
from dotenv import load_dotenv

load_dotenv()

API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8000/api")

# 測試帳號設定 (對齊 setup_test_users.py)
USERS = {
    "IACUC_STAFF": {"email": "staff_test@example.com", "password": "password123"},
    "REVIEWER1": {"email": "rev1_test@example.com", "password": "password123"},
    "REVIEWER2": {"email": "rev2_test@example.com", "password": "password123"},
    "REVIEWER3": {"email": "rev3_test@example.com", "password": "password123"},
    "IACUC_CHAIR": {"email": "chair_test@example.com", "password": "password123"},
    "PI": {"email": "pi_test@example.com", "password": "password123"},
    "VET": {"email": "vet_test@example.com", "password": "password123"},
    "REV_OTHER": {"email": "rev_other_test@example.com", "password": "password123"},
}

class AUPTester:
    def __init__(self):
        self.tokens = {}
        self.user_ids = {}
        self.protocol_id = None
        self.version_id = None

    def login_all(self):
        print("\n[Auth] Logging in all users...")
        for role, credentials in USERS.items():
            resp = requests.post(f"{API_BASE_URL}/auth/login", json=credentials)
            if resp.status_code == 200:
                data = resp.json()
                self.tokens[role] = data["access_token"]
                self.user_ids[role] = data["user"]["id"]
                print(f"  - {role} logged in.")
            else:
                print(f"  - Failed to login {role}: {resp.status_code} {resp.text}")

    def get_headers(self, role):
        return {"Authorization": f"Bearer {self.tokens[role]}"}

    def find_user_by_role(self, role_code):
        print(f"  [Search] Finding user with role: {role_code}...")
        resp = requests.get(f"{API_BASE_URL}/admin/users", headers=self.get_headers("IACUC_STAFF"))
        if resp.status_code == 200:
            users = resp.json()
            for u in users:
                if role_code in u.get("roles", []):
                    print(f"  [Search] Found: {u['email']} (ID: {u['id']})")
                    return u["id"]
        return None

    def _req(self, method, url, **kwargs):
        resp = requests.request(method, url, **kwargs)
        if resp.status_code >= 400:
            error_data = {
                "method": method,
                "url": url,
                "status": resp.status_code,
                "payload": kwargs.get("json"),
                "response": resp.text
            }
            try:
                error_data["response_json"] = resp.json()
            except:
                pass
            
            import json
            with open("last_error.json", "w", encoding="utf-8") as f:
                json.dump(error_data, f, indent=2, ensure_ascii=False)
            
            print(f"\n[API ERROR] {method} {url} - Status: {resp.status_code}")
            print(f"Details saved to last_error.json")
            
        resp.raise_for_status()
        return resp

def run_flow():
    t = AUPTester()
    t.login_all()

    # 1. PI Creating Protocol
    print("\n[Step 1] PI Creating Protocol...")
    payload = {
        "title": f"AUP完整流程測試_{int(time.time())}",
        "working_content": {
            "basic": {
                "study_title": "AUP 14步大滿貫測試計畫",
                "project_code": "FULL-TEST-001",
                "project_type": "Research",
                "is_glp": False,
                "apply_study_number": f"AUP-{int(time.time() * 1000) % 1000000}",
                "registration_authorities": [],
                "pi_user_id": t.user_ids["PI"]
            },
            "animals": [{"species": "Pig", "count": 5, "strain": "L6", "sex": "MIXED", "age": "8 weeks"}],
        },
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
        "working_content": payload["working_content"]
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
    
    v_id = t._req("GET", f"{API_BASE_URL}/protocols/{t.protocol_id}/versions", headers=t.get_headers("VET")).json()[0]["id"]
    vet_comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", json={
        "protocol_version_id": v_id,
        "content": "Medical"
    }, headers=t.get_headers("VET")).json()
    vc_id = vet_comment["id"]
    
    # PI Replying !!!
    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", json={
        "parent_comment_id": vc_id,
        "content": "!!!"
    }, headers=t.get_headers("PI"))
    
    # STAFF 退回 PI 修訂標題 +M
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={"to_status": "VET_REVISION_REQUIRED", "remark": "醫療意見修訂"}, headers=t.get_headers("IACUC_STAFF"))
    
    t._req("PUT", f"{API_BASE_URL}/protocols/{t.protocol_id}", json={
        "title": payload["title"] + " +1 +M",
        "working_content": payload["working_content"]
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
    print("\n[Step 11] PI 回覆所有審查委員 (!!!)")
    for role, cid in rev_cids:
        t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", json={
            "parent_comment_id": cid,
            "content": "!!!"
        }, headers=t.get_headers("PI"))
    
    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", json={
        "parent_comment_id": oc_id,
        "content": "!!!"
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
    # 確保狀態回到 UNDER_REVIEW 或適當狀態供 Chair 核定 (依系統邏輯可能需要 Staff 先處理)
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "UNDER_REVIEW",
        "reviewer_ids": reviewers
    }, headers=t.get_headers("IACUC_STAFF"))

    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "APPROVED",
        "remark": "審核通過證明寄發。"
    }, headers=t.get_headers("IACUC_CHAIR"))
    
    print(f"\n[SUCCESS] AUP 完整 14 步流程完成! Protocol ID: {t.protocol_id}")
    print("狀態已變更為: APPROVED")


if __name__ == "__main__":
    try:
        run_flow()
    except Exception as e:
        print(f"\n[CRITICAL ERROR] Flow failed: {e}")
