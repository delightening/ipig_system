import requests
import os
import json
from dotenv import load_dotenv

def check():
    load_dotenv()
    API_BASE_URL = os.getenv('API_BASE_URL', 'http://localhost:8000/api')
    
    # 登入獲取 Token
    staff_resp = requests.post(f'{API_BASE_URL}/auth/login', json={
        'email': 'staff_test@example.com',
        'password': 'password123'
    })
    token = staff_resp.json()['access_token']
    headers = {'Authorization': f'Bearer {token}'}
    
    # 使用 get_protocol_activities 也能看到部分資訊，或直接查 comments
    # 既然沒有 SQL admin，我們用 get_comments 且不帶參數試試 (如果後端有預防，這裡會報錯)
    resp = requests.get(f'{API_BASE_URL}/reviews/comments', headers=headers)
    print(f"Status: {resp.status_code}")
    if resp.status_code == 200:
        data = resp.json()
        print(f"Total rows: {len(data)}")
        for r in data[:20]:
            print(f"ID: {r['id']}, PID: {r.get('protocol_id')}, Parent: {r.get('parent_comment_id')}, Content: {r['content'][:20]}")

if __name__ == "__main__":
    check()
