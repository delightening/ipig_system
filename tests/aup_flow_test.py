import os
import requests
import pytest
import time
from dotenv import load_dotenv

load_dotenv()

API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8000/api")

# 使用實際已知帳號進行扮演
USERS = {
    "ADMIN": {"email": "admin@ipig.local", "password": "admin123"},
    "YIJUN": {"email": "yijun@ipig.local", "password": "password123"}, # 扮演實際 STAFF 角色
    "PI": {"email": "pi_test@example.com", "password": "password123"},
    "VET": {"email": "vet_test@example.com", "password": "password123"},
    "REV1": {"email": "rev1_test@example.com", "password": "password123"},
    "CHAIR": {"email": "chair_test@example.com", "password": "password123"},
}

class AUPTester:
    def __init__(self):
        self.tokens = {}
        self.protocol_id = None
        self.pi_id = None
        self.yijun_id = None

    def login(self, role):
        user = USERS[role]
        resp = requests.post(f"{API_BASE_URL}/auth/login", json={
            "email": user["email"],
            "password": user["password"]
        })
        if resp.status_code != 200:
            print(f"Login failed for {role} ({user['email']}): {resp.text}")
            resp.raise_for_status()
        self.tokens[role] = resp.json()["access_token"]
        return self.tokens[role]

    def get_headers(self, role):
        return {"Authorization": f"Bearer {self.tokens[role]}"}

@pytest.fixture(scope="module")
def tester():
    t = AUPTester()
    # 登入所有帳號
    for role in USERS:
        try:
            t.login(role)
        except Exception as e:
            print(f"Warning: Login failed for {role}: {e}")
            if role == "YIJUN":
                 # 如果 yijun 不存在，改試 yijun@example.com 或提示
                 USERS["YIJUN"]["email"] = "yijun@example.com"
                 t.login(role)
    
    t.pi_id = requests.get(f"{API_BASE_URL}/me", headers=t.get_headers("PI")).json()["id"]
    t.yijun_id = requests.get(f"{API_BASE_URL}/me", headers=t.get_headers("YIJUN")).json()["id"]
    return t

def test_aup_real_account_flow(tester):
    # 步驟 1: ADMIN 建立計畫（最穩定的格式）並指派 PI
    print("\n[Step 1] Creating Protocol for PI...")
    payload = {
        "title": f"怡均實帳號核定展示_{int(time.time())}",
        "pi_user_id": tester.pi_id,
        "working_content": {
            "basic": {
                "study_title": "豬隻整合審查結果展示 (實帳號扮演)",
                "project_type": "Research",
                "is_glp": False,
                "apply_study_number": f"SHOW-ACT-{int(time.time() % 10000)}",
                "registration_authorities": []
            },
            "animals": []
        },
        "start_date": "2026-03-01", 
        "end_date": "2027-03-01"
    }
    
    resp = requests.post(f"{API_BASE_URL}/protocols", json=payload, headers=tester.get_headers("ADMIN"))
    resp.raise_for_status()
    tester.protocol_id = resp.json()["id"]

    # 關鍵：指派 YIJUN 為 Co-editor
    print("[Control] Assigning YIJUN as Co-editor...")
    requests.post(f"{API_BASE_URL}/protocols/{tester.protocol_id}/co-editors", json={
        "user_id": tester.yijun_id, 
        "protocol_id": tester.protocol_id
    }, headers=tester.get_headers("ADMIN")).raise_for_status()

    # 步驟 2: PI 提交
    print("[Step 2] PI Submitting...")
    requests.post(f"{API_BASE_URL}/protocols/{tester.protocol_id}/submit", headers=tester.get_headers("PI")).raise_for_status()

    # 步驟 3: YIJUN (怡均) 扮演 STAFF 執行預審
    print("[Step 3] YIJUN Pre-reviewing...")
    requests.post(f"{API_BASE_URL}/protocols/{tester.protocol_id}/status", json={
        "to_status": "PRE_REVIEW"
    }, headers=tester.get_headers("YIJUN")).raise_for_status()

    # 步驟 4: YIJUN 指派委員
    print("[Step 5] YIJUN Assigning Reviewers...")
    rev1_id = requests.get(f"{API_BASE_URL}/me", headers=tester.get_headers("REV1")).json()["id"]
    requests.post(f"{API_BASE_URL}/protocols/{tester.protocol_id}/status", json={
        "to_status": "UNDER_REVIEW", 
        "reviewer_ids": [rev1_id]
    }, headers=tester.get_headers("YIJUN")).raise_for_status()

    # 步驟 6: CHAIR 核定
    print("[Step 6] CHAIR Approving...")
    resp = requests.post(f"{API_BASE_URL}/protocols/{tester.protocol_id}/status", json={
        "to_status": "APPROVED", 
        "remark": "由自動化腳本扮演實體帳號完成展示"
    }, headers=tester.get_headers("CHAIR"))
    
    assert resp.status_code == 200
    assert resp.json()["status"] == "APPROVED"
    print(f"\n[SUCCESS] APPROVED! Protocol ID: {tester.protocol_id}")
    print("請前往網頁介面查看分類：已核准。")
