import requests
import os
import json
from dotenv import load_dotenv

def verify():
    load_dotenv()
    API_BASE_URL = os.getenv('API_BASE_URL', 'http://localhost:8000/api')
    
    # 登入為 STAFF
    print("[1] Logging in as IACUC_STAFF...")
    login_resp = requests.post(f'{API_BASE_URL}/auth/login', json={
        'email': 'staff_test@example.com',
        'password': 'password123'
    })
    login_resp.raise_for_status()
    token = login_resp.json()['access_token']
    headers = {'Authorization': f'Bearer {token}'}
    
    # 獲取最新計畫
    print("[2] Fetching latest protocols...")
    protocols_resp = requests.get(f'{API_BASE_URL}/protocols', headers=headers)
    protocols_resp.raise_for_status()
    protocols = protocols_resp.json()
    
    latest_p = protocols[0]
    p_id = latest_p['id']
    print(f"Latest Protocol: {latest_p['protocol_no']} ({p_id})")
    
    # 獲取所有留言 (不帶過濾，看全量數據)
    print("[3] Fetching ALL comments (no filter)...")
    # 注意：如果後端不支援不帶參數，可能還是會報錯
    all_comments_resp = requests.get(f'{API_BASE_URL}/reviews/comments', headers=headers)
    
    # 獲取指定計畫留言
    print(f"[4] Fetching comments for protocol {p_id}...")
    comments_resp = requests.get(f'{API_BASE_URL}/reviews/comments', params={'protocol_id': p_id}, headers=headers)
    
    comments = comments_resp.json()
    print(f"\nProtocol Comments Count: {len(comments)}")
    for c in comments:
        print(f"- ID: {c['id']}, Parent: {c.get('parent_comment_id')}, VerID: {c.get('protocol_version_id')}, Content: {c['content'][:30]}")
    
    # 檢查是否有任何留言的 parent_comment_id 指向這些留言
    parent_ids = [c['id'] for c in comments]
    
    # 在全量數據中找尋回覆
    if all_comments_resp.status_code == 200:
        all_c = all_comments_resp.json()
        replies_to_protocol = [c for c in all_c if c.get('parent_comment_id') in parent_ids]
        print(f"\nFound {len(replies_to_protocol)} replies in ALL_COMMENTS that point to this protocol's comments.")
        for r in replies_to_protocol:
            print(f"  !!!!! DETECTED LOST REPLY !!!!!")
            print(f"  Reply ID: {r['id']}")
            print(f"  Protocol ID in Reply object: {r.get('protocol_id', 'MISSING')}")
            print(f"  Parent ID: {r['parent_comment_id']}")
            print(f"  Content: {r['content']}")

if __name__ == "__main__":
    verify()
