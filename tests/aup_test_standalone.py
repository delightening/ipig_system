import os
import requests
import time
from dotenv import load_dotenv

load_dotenv()

API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8000/api")

# 測試帳號設定 (對齊 setup_test_users.py)
USERS = {
    "IACUC_STAFF": {"email": "monkey20531@gmail.com", "password": "12345678"},
    "REVIEWER1": {"email": "rev1_test@example.com", "password": "password123"},
    "REVIEWER2": {"email": "rev2_test@example.com", "password": "password123"},
    "REVIEWER3": {"email": "rev3_test@example.com", "password": "password123"},
    "IACUC_CHAIR": {"email": "chair_test@example.com", "password": "password123"},
    "PI": {"email": "pi_test@example.com", "password": "password123"},
    "VET": {"email": "vet_test@example.com", "password": "password123"},
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
        "title": f"AUP測試_{int(time.time())}",
        "working_content": {
            "basic": {
                "study_title": "AUP 14步測試計畫 (自動化測試)",
                "project_type": "Research",
                "is_glp": False,
                "apply_study_number": f"AUP-{int(time.time() * 1000) % 1000000}",
                "registration_authorities": [],
                "pi_user_id": t.user_ids["PI"]
            },
            "animals": [{"species": "Pig", "count": 10, "strain": "L6", "sex": "MIXED", "age": "8 weeks"}]
        },
        "start_date": "2026-04-01",
        "end_date": "2027-04-01",
        "pi_user_id": t.user_ids["PI"]
    }
    resp = t._req("POST", f"{API_BASE_URL}/protocols", json=payload, headers=t.get_headers("PI"))
    t.protocol_id = resp.json()["id"]
    
    # 指派工作人員 (Co-editor)，這是進入行政預審的必經步驟
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/co-editors", json={
        "user_id": t.user_ids["IACUC_STAFF"], 
        "protocol_id": t.protocol_id
    }, headers=t.get_headers("IACUC_STAFF"))

    # PI Submit
    print("  - Submitting...")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI"))
    print("  - Protocol created and submitted.")

    # 2. Staff Pre-review
    print("\n[Step 2] Staff Pre-review...")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={"to_status": "PRE_REVIEW"}, headers=t.get_headers("IACUC_STAFF"))
    
    # Get Version ID for comments
    versions = t._req("GET", f"{API_BASE_URL}/protocols/{t.protocol_id}/versions", headers=t.get_headers("IACUC_STAFF")).json()
    v_id = versions[0]["id"]
    
    # Add Comment
    comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", json={
        "protocol_version_id": v_id,
        "content": "行政預審意見：請補充經費來源。"
    }, headers=t.get_headers("IACUC_STAFF")).json()
    c_id = comment["id"]
    
    # Revision Required
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "PRE_REVIEW_REVISION_REQUIRED", 
        "remark": "請修正行政預審意見"
    }, headers=t.get_headers("IACUC_STAFF"))
    print("  - Comment added and revision required.")

    # 3. PI Replying
    print("\n[Step 3] PI Replying...")
    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", json={
        "parent_comment_id": c_id,
        "content": "已補充經費來源為國科會。"
    }, headers=t.get_headers("PI"))
    
    # Update content
    t._req("PUT", f"{API_BASE_URL}/protocols/{t.protocol_id}", json={
        "title": payload["title"] + " (修正1)",
        "working_content": {
            "basic": payload["working_content"]["basic"],
            "animals": [{"species": "Pig", "count": 10, "strain": "L6", "sex": "MIXED", "age": "8 weeks"}]
        }
    }, headers=t.get_headers("PI"))
    
    # Resubmit
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI"))
    print("  - PI replied and resubmitted.")

    # 4. Staff Assign Vet
    print("\n[Step 4] Staff assigning Vet Review...")
    vet_id = t.find_user_by_role("VET") or t.user_ids["VET"]
    
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={"to_status": "PRE_REVIEW"}, headers=t.get_headers("IACUC_STAFF"))
    
    t._req("POST", f"{API_BASE_URL}/reviews/comments/{c_id}/resolve", headers=t.get_headers("IACUC_STAFF"))
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "VET_REVIEW",
        "vet_id": vet_id
    }, headers=t.get_headers("IACUC_STAFF"))
    print(f"  - Assigned to Vet (ID: {vet_id}).")

    # 5. Vet Review
    print("\n[Step 5] Vet Reviewing...")
    versions = t._req("GET", f"{API_BASE_URL}/protocols/{t.protocol_id}/versions", headers=t.get_headers("VET")).json()
    v_id = versions[0]["id"]
    vet_comment = t._req("POST", f"{API_BASE_URL}/reviews/comments", json={
        "protocol_version_id": v_id,
        "content": "醫療審查意見：請詳細說明麻醉劑劑量。"
    }, headers=t.get_headers("VET")).json()
    vc_id = vet_comment["id"]
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "VET_REVISION_REQUIRED",
        "remark": "請修正醫療審查意見"
    }, headers=t.get_headers("IACUC_STAFF"))
    print("  - Vet comment added.")

    # 6. PI Replying to Vet
    print("\n[Step 6] PI Replying to Vet...")
    t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", json={
        "parent_comment_id": vc_id,
        "content": "同意，已在計畫書更新劑量。"
    }, headers=t.get_headers("PI"))
    t._req("PUT", f"{API_BASE_URL}/protocols/{t.protocol_id}", json={
        "title": payload["title"] + " (修正2)",
        "working_content": {
            "basic": {**payload["working_content"]["basic"], "revision_note": "麻醉劑量修正"},
            "animals": [{"species": "Pig", "count": 10, "strain": "L6", "sex": "MIXED", "age": "8 weeks"}]
        }
    }, headers=t.get_headers("PI"))
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI"))
    print("  - PI resubmitted after Vet review.")

    # 7. Vet Complete
    print("\n[Step 7] Vet completing review...")
    t._req("POST", f"{API_BASE_URL}/reviews/comments/{vc_id}/resolve", headers=t.get_headers("VET"))
    # 注意：PI 重送後狀態為 RESUBMITTED，不需要再切換回 PRE_REVIEW，直接進入下一個環節
    print("  - Vet review resolved.")

    # 8. Staff Assign Reviewers
    print("\n[Step 8] Staff assigning 3 Reviewers...")
    reviewers = [t.user_ids["REVIEWER1"], t.user_ids["REVIEWER2"], t.user_ids["REVIEWER3"]]
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "UNDER_REVIEW",
        "reviewer_ids": reviewers
    }, headers=t.get_headers("IACUC_STAFF"))
    print("  - Under review.")

    # 9 & 10. Reviewer comments
    print("\n[Steps 9 & 10] Reviewers providing comments...")
    versions = t._req("GET", f"{API_BASE_URL}/protocols/{t.protocol_id}/versions", headers=t.get_headers("IACUC_STAFF")).json()
    v_id = versions[0]["id"]
    rev_cids = []
    for role in ["REVIEWER1", "REVIEWER2", "REVIEWER3"]:
        c = t._req("POST", f"{API_BASE_URL}/reviews/comments", json={
            "protocol_version_id": v_id,
            "content": f"{role} 審查委員意見。"
        }, headers=t.get_headers(role)).json()
        rev_cids.append((role, c["id"]))
    print("  - Designated reviewers commented.")

    # 11. PI Reply all
    print("\n[Step 11] PI replying to all...")
    for role, c_idx in rev_cids:
        t._req("POST", f"{API_BASE_URL}/reviews/comments/reply", json={
            "parent_comment_id": c_idx,
            "content": "同意修正。"
        }, headers=t.get_headers("PI"))
    print("  - PI replied all.")

    # 12. Staff set Revision Required
    print("\n[Step 12] Staff Revision Required...")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={"to_status": "REVISION_REQUIRED"}, headers=t.get_headers("IACUC_STAFF"))
    print("  - Status: REVISION_REQUIRED.")

    # 13. PI Final Submit & Resolve
    print("\n[Step 13] PI Final Submit & Resolve...")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI"))
    for role, c_idx in rev_cids:
         t._req("POST", f"{API_BASE_URL}/reviews/comments/{c_idx}/resolve", headers=t.get_headers(role))
    print("  - All resolved.")

    # 14. Chair Approve
    print("\n[Step 14] Chair Approving...")
    # 關鍵：進入核准前，必須先從 RESUBMITTED 回到 UNDER_REVIEW，且必須重新帶入審查委員名單
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "UNDER_REVIEW",
        "reviewer_ids": reviewers
    }, headers=t.get_headers("IACUC_STAFF"))
    
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/status", json={
        "to_status": "APPROVED",
        "remark": "核定通過。"
    }, headers=t.get_headers("IACUC_STAFF"))
    
    print(f"\n[SUCCESS] AUP Flow Completed! Protocol ID: {t.protocol_id}")

if __name__ == "__main__":
    try:
        run_flow()
    except Exception as e:
        print(f"\n[CRITICAL ERROR] Flow failed: {e}")
