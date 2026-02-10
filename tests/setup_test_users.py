import os
import subprocess
from dotenv import load_dotenv

load_dotenv()

# 測試帳號定義
USERS = [
    {"email": "pi_test@example.com", "password": "password123", "name": "Test PI"},
    {"email": "staff_test@example.com", "password": "password123", "name": "IACUC Staff"},
    {"email": "vet_test@example.com", "password": "password123", "name": "Test Vet"},
    {"email": "rev1_test@example.com", "password": "password123", "name": "Reviewer 1"},
    {"email": "rev2_test@example.com", "password": "password123", "name": "Reviewer 2"},
    {"email": "rev3_test@example.com", "password": "password123", "name": "Reviewer 3"},
    {"email": "chair_test@example.com", "password": "password123", "name": "IACUC Chair"},
    {"email": "rev_other_test@example.com", "password": "password123", "name": "Reviewer Other"},
]

def create_users():
    base_dir = r"d:\Coding\ipig_system"
    backend_dir = os.path.join(base_dir, "backend")
    
    # 讀取 .env 並修正為本機開發路徑
    env_vars = os.environ.copy()
    env_vars["DATABASE_URL"] = "postgres://postgres:ipig_password_123@localhost:543/ipig_db"

    print(f"Working directory: {backend_dir}")
    os.chdir(backend_dir)
    
    for user in USERS:
        print(f"Creating user: {user['email']}...")
        try:
            # 直接呼叫 cargo run
            cmd = f'cargo run --bin create_test_user {user["email"]} {user["password"]} "{user["name"]}"'
            result = subprocess.run(cmd, shell=True, check=True, capture_output=True, env=env_vars, text=True)
            print(f"Success: {user['email']}")
            print(f"Output: {result.stdout.strip()}")
        except subprocess.CalledProcessError as e:
            print(f"Failed to create {user['email']}")
            print(f"Exit code: {e.returncode}")
            print(f"Stdout: {e.stdout}")
            print(f"Stderr: {e.stderr}")
        except Exception as ex:
            print(f"Unexpected error: {ex}")

if __name__ == "__main__":
    try:
        create_users()
    except Exception as e:
        print(f"Main execution failed: {e}")
