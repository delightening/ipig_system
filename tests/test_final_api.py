import requests
import os
import json
from dotenv import load_dotenv

def final_test():
    load_dotenv()
    API_BASE_URL = os.getenv('API_BASE_URL', 'http://localhost:8000/api')
    
    # 登入
    staff_resp = requests.post(f'{API_BASE_URL}/auth/login', json={
        'email': 'staff_test@example.com',
        'password': 'password123'
    })
    staff_token = staff_resp.json()['access_token']
    
    pi_resp = requests.post(f'{API_BASE_URL}/auth/login', json={
        'email': 'pi_test@example.com',
        'password': 'password123'
    })
    pi_token = pi_resp.json()['access_token']
    
    # 1. 獲取最新計畫
    p_resp = requests.get(f'{API_BASE_URL}/protocols', headers={'Authorization': f'Bearer {staff_token}'})
    p_id = p_resp.json()[0]['id']
    print(f"Latest Protocol ID: {p_id}")
    
    # 2. 獲取留言
    c_resp = requests.get(f'{API_BASE_URL}/reviews/comments', params={'protocol_id': p_id}, headers={'Authorization': f'Bearer {staff_token}'})
    comments = c_resp.json()
    if not comments:
        print("No comments.")
        return
        
    c_id = comments[0]['id']
    print(f"Replying to Comment: {c_id}")
    
    # 3. PI 回覆
    print("\n[PI] Replying...")
    r_resp = requests.post(f'{API_BASE_URL}/reviews/comments/reply', json={
        'parent_comment_id': c_id,
        'content': 'LATEST REPAIR VERIFICATION'
    }, headers={'Authorization': f'Bearer {pi_token}'})
    
    if r_resp.status_code == 200:
        data = r_resp.json()
        print("API Return JSON:")
        print(json.dumps(data, indent=2, ensure_ascii=False))
        
        # 4. 再次查詢
        print("\nChecking list again...")
        l_resp = requests.get(f'{API_BASE_URL}/reviews/comments', params={'protocol_id': p_id}, headers={'Authorization': f'Bearer {staff_token}'})
        l_data = l_resp.json()
        print(f"Total list count: {len(l_data)}")
        for item in l_data:
            if item['id'] == data['id']:
                print(f"  >>> SUCCESS: FOUND NEW REPLY IN LIST!")
                print(f"  >>> Reply Detail: ID={item['id']}, PID={item.get('protocol_id')}")
    else:
        print(f"Error: {r_resp.text}")

if __name__ == "__main__":
    final_test()
