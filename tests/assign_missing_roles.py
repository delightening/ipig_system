import os
import psycopg2
from dotenv import load_dotenv

load_dotenv()

# 資料庫連線設定
DB_URL = "postgres://postgres:ipig_password_123@localhost:543/ipig_db"

# 帳號與角色對照表
USER_ROLES = {
    "pi_test@example.com": "PI",
    "staff_test@example.com": "IACUC_STAFF",
    "vet_test@example.com": "VET",
    "rev1_test@example.com": "REVIEWER",
    "rev2_test@example.com": "REVIEWER",
    "rev3_test@example.com": "REVIEWER",
    "chair_test@example.com": "IACUC_CHAIR",
    "rev_other_test@example.com": "REVIEWER",
}

def assign_roles():
    try:
        conn = psycopg2.connect(DB_URL)
        cur = conn.cursor()
        
        for email, role_code in USER_ROLES.items():
            print(f"Assigning {role_code} to {email}...")
            
            # 獲取 user_id
            cur.execute("SELECT id FROM users WHERE email = %s", (email,))
            row = cur.fetchone()
            if not row:
                print(f"  User {email} not found, skipping.")
                continue
            user_id = row[0]
            
            # 獲取 role_id
            cur.execute("SELECT id FROM roles WHERE code = %s", (role_code,))
            row = cur.fetchone()
            if not row:
                print(f"  Role {role_code} not found, skipping.")
                continue
            role_id = row[0]
            
            # 指派角色 (避免重複)
            cur.execute(
                "INSERT INTO user_roles (user_id, role_id) VALUES (%s, %s) ON CONFLICT DO NOTHING",
                (user_id, role_id)
            )
            
        conn.commit()
        print("\n[SUCCESS] Role assignment completed.")
        cur.close()
        conn.close()
    except Exception as e:
        print(f"\n[ERROR] Role assignment failed: {e}")

if __name__ == "__main__":
    assign_roles()
