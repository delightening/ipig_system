import os
import requests
import pytest
import time
from dotenv import load_dotenv

load_dotenv()

API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8000/api")

# 測試帳號設定 (根據 setup_test_users.py 與使用者提供)
USERS = {
    "IACUC_STAFF": {"email": "monkey20531@gmail.com", "password": "12345678"},
    "REVIEWER1": {"email": "rev1_test@example.com", "password": "password123"},
    "REVIEWER2": {"email": "rev2_test@example.com", "password": "password123"},
    "REVIEWER3": {"email": "rev3_test@example.com", "password": "password123"},
    "IACUC_CHAIR": {"email": "chair_test@example.com", "password": "password123"},
    "PI": {"email": "pi_test@example.com", "password": "password123"},
    "VET": {"email": "museum1925@gmail.com", "password": "12345678"}, # 扮演 Vet
    "REV_OTHER": {"email": "vet_test@example.com", "password": "password123"}, # 模擬外部委員
}

class AUPTester:
    def __init__(self):
        self.tokens = {}
        self.user_ids = {}
        self.protocol_id = None
        self.version_id = None
        self.staff_comment_id = None
        self.vet_comment_id = None
        self.rev_comments = []
        self.other_comment_id = None

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

    def _req(self, method, url, **kwargs):
        resp = requests.request(method, url, **kwargs)
        if resp.status_code >= 400:
            print(f"\n[API ERROR] {method} {url}")
            print(f"Status: {resp.status_code}")
            print(f"Response: {resp.text}")
            if "json" in kwargs:
                import json
                print(f"Payload: {json.dumps(kwargs['json'], indent=2, ensure_ascii=False)}")
        resp.raise_for_status()
        return resp

    def change_status(self, role, to_status, reviewer_ids=None, vet_id=None, remark=None):
        payload = {"to_status": to_status}
        if reviewer_ids: payload["reviewer_ids"] = reviewer_ids
        if vet_id: payload["vet_id"] = vet_id
        if remark: payload["remark"] = remark
        
        resp = self._req("POST",
            f"{API_BASE_URL}/protocols/{self.protocol_id}/status",
            json=payload,
            headers=self.get_headers(role)
        )
        return resp.json()

    def add_comment(self, role, content, stage):
        versions = self._req("GET",
            f"{API_BASE_URL}/protocols/{self.protocol_id}/versions",
            headers=self.get_headers(role)
        ).json()
        self.version_id = versions[0]["id"]

        resp = self._req("POST",
            f"{API_BASE_URL}/reviews/comments",
            json={
                "protocol_version_id": self.version_id,
                "content": content
            },
            headers=self.get_headers(role)
        )
        return resp.json()

    def reply_comment(self, role, comment_id, content):
        resp = self._req("POST",
            f"{API_BASE_URL}/reviews/comments/reply",
            json={
                "parent_comment_id": comment_id,
                "content": content
            },
            headers=self.get_headers(role)
        )
        return resp.json()

    def resolve_comment(self, role, comment_id):
        resp = self._req("POST",
            f"{API_BASE_URL}/reviews/comments/{comment_id}/resolve",
            headers=self.get_headers(role)
        )
        return resp.json()

@pytest.fixture(scope="module")
def t():
    tester = AUPTester()
    tester.login_all()
    return tester

def test_aup_14_steps_flow(t):
    # ---------------------------------------------------------
    # 1. 扮演PI 撰寫計劃書，滿足計劃書欄位
    # ---------------------------------------------------------
    print("\n[Step 1] PI Creating Protocol...")
    payload = {
        "title": f"AUP 14步測試計畫_{int(time.time())}",
        "working_content": {
            "basic": {
                "study_title": "AUP 14步測試計畫 (自動化測試)",
                "project_type": "Research",
                "is_glp": False,
                "apply_study_number": f"AUP-TEST-{int(time.time() % 10000)}",
                "registration_authorities": [],
                "pi_user_id": t.user_ids["PI"]
            },
            "animals": [{"species": "Pig", "count": 10, "strain": "L6", "sex": "Mixed", "age": "8 weeks"}]
        },
        "start_date": "2026-04-01",
        "end_date": "2027-04-01"
    }
    resp = t._req("POST", f"{API_BASE_URL}/protocols", json=payload, headers=t.get_headers("PI"))
    t.protocol_id = resp.json()["id"]
    
    print("  - [Rule] Assigning Co-editor (Staff Role)...")
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/co-editors", json={
        "user_id": t.user_ids["IACUC_STAFF"], 
        "protocol_id": t.protocol_id
    }, headers=t.get_headers("IACUC_STAFF"))

    # PI 提交
    t._req("POST", f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI"))
    print("  - Protocol created and submitted.")

    # ---------------------------------------------------------
    # 2. 扮演IACUC_staff，進行行政預審，並提交意見，發回更改
    # ---------------------------------------------------------
    print("\n[Step 2] Staff Pre-review and request revision...")
    t.change_status("IACUC_STAFF", "PRE_REVIEW")
    comment = t.add_comment("IACUC_STAFF", "行政預審意見：請補充經費來源。", "PRE_REVIEW")
    t.staff_comment_id = comment["id"]
    t.change_status("IACUC_STAFF", "PRE_REVIEW_REVISION_REQUIRED", remark="請修正行政預審意見")
    print("  - Comment added and revision required.")

    # ---------------------------------------------------------
    # 3. 扮演PI 回復IACUC_staff 意見，更改計畫內容
    # ---------------------------------------------------------
    print("\n[Step 3] PI Replying and resubmitting...")
    t.reply_comment("PI", t.staff_comment_id, "已補充經費來源為國科會。")
    # 模擬更新計畫內容
    resp = requests.put(f"{API_BASE_URL}/protocols/{t.protocol_id}", json={
        "title": f"AUP 14步測試計畫 (已修正)_{int(time.time())}",
        "working_content": {
            "basic": {
                "study_title": "AUP 14步測試計畫 (自動化測試-修正)",
                "project_type": "Research",
                "is_glp": False,
                "apply_study_number": f"AUP-TEST-{int(time.time() % 10000)}",
                "registration_authorities": []
            },
            "animals": [{"species": "Pig", "count": 10}]
        }
    }, headers=t.get_headers("PI"))
    if resp.status_code != 200:
        print(f"\n[DEBUG] Update Protocol Failed: {resp.text}")
    resp.raise_for_status()
    # 再次提交
    resp = requests.post(f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI"))
    if resp.status_code != 200:
        print(f"\n[DEBUG] Resubmit Protocol Failed: {resp.text}")
    resp.raise_for_status()
    print("  - PI replied and resubmitted.")

    # ---------------------------------------------------------
    # 4. 扮演IACUC_staff，通過行政預審，請 Vet 進行醫療審查
    # ---------------------------------------------------------
    print("\n[Step 4] Staff assigning Vet Review...")
    t.resolve_comment("IACUC_STAFF", t.staff_comment_id)
    t.change_status("IACUC_STAFF", "VET_REVIEW", vet_id=t.user_ids["VET"])
    print("  - Admin review passed, assigned to Vet.")

    # ---------------------------------------------------------
    # 5. 扮演Vet，進行醫療審查，提出意見，請 PI 更改計劃書
    # ---------------------------------------------------------
    print("\n[Step 5] Vet Reviewing and request revision...")
    comment = t.add_comment("VET", "醫療審查意見：請詳細說明麻醉劑劑量。", "VET_REVIEW")
    t.vet_comment_id = comment["id"]
    t.change_status("IACUC_STAFF", "VET_REVISION_REQUIRED", remark="請修正醫療審查意見")
    print("  - Vet comment added and revision required.")

    # ---------------------------------------------------------
    # 6. 扮演PI，同意Vet 意見，更改計劃書
    # ---------------------------------------------------------
    print("\n[Step 6] PI Agreeing and correcting...")
    t.reply_comment("PI", t.vet_comment_id, "同意，已在計畫書第 5 節更新劑量。")
    # 模擬更新
    requests.put(f"{API_BASE_URL}/protocols/{t.protocol_id}", json={
        "working_content": {"basic": {"revision_note": "麻醉劑量修正"}},
    }, headers=t.get_headers("PI")).raise_for_status()
    # 再次提交
    requests.post(f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI")).raise_for_status()
    print("  - PI replied to Vet and resubmitted.")

    # ---------------------------------------------------------
    # 7. 扮演Vet，完成醫療審查，通知IACUC_staff
    # ---------------------------------------------------------
    print("\n[Step 7] Vet completing review...")
    t.resolve_comment("VET", t.vet_comment_id)
    # 完成醫療審查 (狀態切換至 PRE_REVIEW 或待分案)
    t.change_status("IACUC_STAFF", "PRE_REVIEW", remark="醫療審查完成")
    print("  - Vet review resolved and notified staff.")

    # ---------------------------------------------------------
    # 8. 扮演IACUC_staff，指定三名委員，進行倫理審查
    # ---------------------------------------------------------
    print("\n[Step 8] Staff assigning 3 Reviewers...")
    reviewers = [t.user_ids["REVIEWER1"], t.user_ids["REVIEWER2"], t.user_ids["REVIEWER3"]]
    t.change_status("IACUC_STAFF", "UNDER_REVIEW", reviewer_ids=reviewers)
    print("  - Assigned 3 reviewers and status changed to UNDER_REVIEW.")

    # ---------------------------------------------------------
    # 9. 扮演被指定三名委員進行審查，並提出意見
    # ---------------------------------------------------------
    print("\n[Step 9] 3 Reviewers providing comments...")
    t.rev_comments = []
    for role in ["REVIEWER1", "REVIEWER2", "REVIEWER3"]:
        comment = t.add_comment(role, f"{role} 審查委員意見：關於人道終點設定。", "UNDER_REVIEW")
        t.rev_comments.append(comment["id"])
    print("  - All 3 primary reviewers added comments.")

    # ---------------------------------------------------------
    # 10 扮演未被指定的委員，提出意見
    # ---------------------------------------------------------
    print("\n[Step 10] Non-assigned reviewer providing comments...")
    comment = t.add_comment("REV_OTHER", "外部意見：豬隻活動空間建議。", "UNDER_REVIEW")
    t.other_comment_id = comment["id"]
    print("  - Non-assigned reviewer added comments.")

    # ---------------------------------------------------------
    # 11 扮演PI，回復所有委員意見，並同意更正計畫書
    # ---------------------------------------------------------
    print("\n[Step 11] PI replying to ALL comments...")
    for cid in t.rev_comments:
        t.reply_comment("PI", cid, "同意修正人道終點。")
    t.reply_comment("PI", t.other_comment_id, "感謝建議，已修正活動空間描述。")
    print("  - PI replied to all reviewers.")

    # ---------------------------------------------------------
    # 12 扮演IACUC_staff，變更計畫狀態，讓PI 可修正計畫書
    # ---------------------------------------------------------
    print("\n[Step 12] Staff setting Revision Required...")
    t.change_status("IACUC_STAFF", "REVISION_REQUIRED")
    print("  - Status changed to REVISION_REQUIRED.")

    # ---------------------------------------------------------
    # 13 計劃書，經所有發表過意見的委員同意後（已解決），通過倫理審查
    # ---------------------------------------------------------
    print("\n[Step 13] Resolving comments and resubmitting...")
    # PI 修正並提交
    requests.post(f"{API_BASE_URL}/protocols/{t.protocol_id}/submit", headers=t.get_headers("PI")).raise_for_status()
    # 委員解決意見
    for i, role in enumerate(["REVIEWER1", "REVIEWER2", "REVIEWER3"]):
        t.resolve_comment(role, t.rev_comments[i])
    t.resolve_comment("REV_OTHER", t.other_comment_id)
    print("  - All comments resolved.")

    # ---------------------------------------------------------
    # 14 Iacuc_Chair 同意通過，並簽發IACUC同意函
    # ---------------------------------------------------------
    print("\n[Step 14] Chair Approving...")
    resp = t.change_status("IACUC_CHAIR", "APPROVED", remark="同意核定，發出同意函。")
    
    assert resp["status"] == "APPROVED"
    print(f"\n[SUCCESS] AUP Flow Completed! Protocol ID: {t.protocol_id}")

if __name__ == "__main__":
    # 手動執行時顯示詳細過程
    pytest.main([__file__, "-s", "-v"])
