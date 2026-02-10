import requests
import os
import json
from dotenv import load_dotenv

def test_single():
    load_dotenv()
    API_BASE_URL = os.getenv('API_BASE_URL', 'http://localhost:8000/api')
    
    # 登入為 STAFF (獲取資料)
    staff_resp = requests.post(f'{API_BASE_URL}/auth/login', json={
        'email': 'staff_test@example.com',
        'password': 'password123'
    })
    staff_token = staff_resp.json()['access_token']
    
    # 登入為 PI (進行回覆)
    pi_resp = requests.post(f'{API_BASE_URL}/auth/login', json={
        'email': 'pi_test@example.com',
        'password': 'password123'
    })
    pi_token = pi_resp.json()['access_token']
    
    # 1. 獲取最新計畫
    p_resp = requests.get(f'{API_BASE_URL}/protocols', headers={'Authorization': f'Bearer {staff_token}'})
    p_id = p_resp.json()[0]['id']
    print(f"Target Protocol: {p_id}")
    
    # 2. 獲取一個主留言
    c_resp = requests.get(f'{API_BASE_URL}/reviews/comments', params={'protocol_id': p_id}, headers={'Authorization': f'Bearer {staff_token}'})
    comments = c_resp.json()
    if not comments:
        print("No comments found for this protocol.")
    else:
        print(f"Comments for protocol {p_id}:")
        for c in comments:
            print(f"- ID: {c['id']}, PID: {c.get('protocol_id')}, Parent: {c.get('parent_comment_id')}, Content: {c['content'][:20]}")

    # 3. 嘗試回覆
    if comments:
        c_id = comments[0]['id']
        print(f"\nSending Reply to {c_id}...")
        reply_resp = requests.post(f'{API_BASE_URL}/reviews/comments/reply', json={
            'parent_comment_id': c_id,
            'content': 'Test Manual Reply'
        }, headers={'Authorization': f'Bearer {pi_token}'})
        
        reply_data = reply_resp.json()
        print(f"Reply Response Status: {reply_resp.status_code}")
        print(f"Reply Body PID: {reply_data.get('protocol_id')}")
        
        # 4. 再次查詢列表
        print("\nChecking list after reply...")
        list_resp = requests.get(f'{API_BASE_URL}/reviews/comments', params={'protocol_id': p_id}, headers={'Authorization': f'Bearer {staff_token}'})
        list_data = list_resp.json()
        print(f"Final Count: {len(list_data)}")
        for item in list_data:
            if item.get('parent_comment_id'):
                print(f"  -> FOUND REPLY: ID={item['id']}, PID={item.get('protocol_id')}")

if __name__ == "__main__":
    test_single()
