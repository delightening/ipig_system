import os
import requests
import psycopg2
from dotenv import load_dotenv

load_dotenv()

# 資料庫連線資訊 (對齊 Docker/本機配置)
DATABASE_URL = "postgres://postgres:ipig_password_123@localhost:543/ipig_db"

USERS_ROLES = {
    "staff_test@example.com": "IACUC_STAFF",
    "rev1_test@example.com": "REVIEWER",
    "rev2_test@example.com": "REVIEWER",
    "rev3_test@example.com": "REVIEWER",
    "chair_test@example.com": "CHAIR",
    "pi_test@example.com": "PI",
    "vet_test@example.com": "VET",
}

def setup_roles():
    print("[Setup] Assigning roles to test users...")
    try:
        conn = psycopg2.connect(DATABASE_URL)
        cur = conn.cursor()

        for email, role_code in USERS_ROLES.items():
            print(f"  - Assigning {role_code} to {email}")
            # 1. 取得 User ID
            cur.execute("SELECT id FROM users WHERE email = %s", (email,))
            user_res = cur.fetchone()
            if not user_res:
                print(f"    ! User {email} not found. Skip.")
                continue
            user_id = user_res[0]

            # 2. 取得 Role ID
            cur.execute("SELECT id FROM roles WHERE code = %s", (role_code,))
            role_res = cur.fetchone()
            if not role_res:
                print(f"    ! Role {role_code} not found. Skip.")
                continue
            role_id = role_res[0]

            # 3. 插入或更新 user_roles
            cur.execute("""
                INSERT INTO user_roles (user_id, role_id)
                VALUES (%s, %s)
                ON CONFLICT (user_id, role_id) DO NOTHING
            """, (user_id, role_id))
        
        conn.commit()
        cur.close()
        conn.close()
        print("[Success] All roles assigned.")
    except Exception as e:
        print(f"[Error] Failed to setup roles: {e}")

if __name__ == "__main__":
    setup_roles()
