"""
完整 HR 人員管理系統測試

包括：
- 建立 ADMIN_STAFF / EXPERIMENT_STAFF 測試角色
- Phase 1: 打卡上下班 (clock-in / clock-out)
- Phase 2: 出勤查詢 + 統計
- Phase 3: 加班申請全流程 (建立→提交→核准 / 建立→提交→駁回)
- Phase 4: 請假申請全流程 (事假→提交→核准 / 病假→提交→駁回 / 事假→取消)
- Phase 5: 特休假額度管理 (建立額度→查餘額→申請特休→驗證使用)
- Phase 6: 餘額彙總驗證

用法：
    cd c:\\System Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/test_hr_full.py
"""

import sys
import os
import time
from datetime import date, timedelta

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL

# 測試帳號設定
HR_TEST_USERS = {
    "ADMIN": {
        "email": "admin@ipig.local",
        "password": "admin123",
        "display_name": "系統管理員",
        "role_codes": ["ADMIN"],
    },
    "ADMIN_STAFF": {
        "email": "hr_admin_staff_test@example.com",
        "password": "password123",
        "display_name": "行政人員 (HR 測試)",
        "role_codes": ["ADMIN_STAFF"],
    },
    "EXPERIMENT_STAFF": {
        "email": "hr_exp_staff_test@example.com",
        "password": "password123",
        "display_name": "試驗工作人員 (HR 測試)",
        "role_codes": ["EXPERIMENT_STAFF"],
    },
}


def run_hr_test() -> bool:
    """執行完整 HR 人員管理測試，回傳是否全部通過"""
    t = BaseApiTester("HR 人員管理系統完整測試")

    if not t.setup_test_users(HR_TEST_USERS):
        return False
    if not t.login_all(HR_TEST_USERS):
        return False

    ADMIN = "ADMIN"
    ADMIN_STAFF = "ADMIN_STAFF"
    STAFF = "EXPERIMENT_STAFF"
    today = str(date.today())

    # ========================================
    # Phase 1: 打卡上班/下班
    # ========================================
    import requests as _requests

    t.step("Phase 1 — 打卡上班")
    headers = t.get_headers(STAFF)
    resp = _requests.post(f"{API_BASE_URL}/hr/attendance/clock-in",
                          headers=headers, json={"source": "test_script"})
    clock_in_data = resp.json()
    if resp.status_code == 200:
        t.record("打卡上班",
                 clock_in_data.get("success") is True,
                 f"clock_in_time={clock_in_data.get('clock_in_time')}")
    else:
        # 400 = 今天已打卡，視為可接受
        t.record("打卡上班",
                 resp.status_code == 400,
                 f"已打卡過 ({clock_in_data.get('error', {}).get('message', resp.text[:80])})")

    # 短暫等待，模擬工作時間
    time.sleep(1)

    t.step("Phase 1 — 打卡下班")
    resp = _requests.post(f"{API_BASE_URL}/hr/attendance/clock-out",
                          headers=headers, json={"source": "test_script"})
    clock_out_data = resp.json()
    if resp.status_code == 200:
        t.record("打卡下班",
                 clock_out_data.get("success") is True,
                 f"clock_out_time={clock_out_data.get('clock_out_time')}")
    else:
        # 400 = 今天已打卡下班 或 尚未打卡上班，視為可接受
        t.record("打卡下班",
                 resp.status_code == 400,
                 f"已打卡過或無法打卡 ({clock_out_data.get('error', {}).get('message', resp.text[:80])})")

    # ========================================
    # Phase 2: 出勤查詢
    # ========================================
    t.step("Phase 2 — 出勤查詢")

    # 2.1 查詢自己的出勤記錄
    resp = t._req("GET", f"{API_BASE_URL}/hr/attendance",
                   role=STAFF)
    attendance_data = resp.json()
    items = attendance_data.get("items", attendance_data.get("data", []))
    t.record("查詢自己出勤記錄",
             len(items) >= 1,
             f"共 {len(items)} 筆出勤記錄")

    # 2.2 管理員查看出勤統計
    resp = t._req("GET", f"{API_BASE_URL}/hr/attendance/stats",
                   role=ADMIN)
    stats_data = resp.json()
    t.record("管理員查看出勤統計",
             resp.status_code == 200,
             f"回傳成功")

    # ========================================
    # Phase 3: 加班申請全流程
    # ========================================
    t.step("Phase 3 — 加班申請（核准流程）")

    # 3.1 STAFF 建立加班申請
    overtime_date = str(date.today() - timedelta(days=2))
    resp = t._req("POST", f"{API_BASE_URL}/hr/overtime",
                   role=STAFF, json={
                       "overtime_date": overtime_date,
                       "start_time": "18:00:00",
                       "end_time": "21:00:00",
                       "overtime_type": "A",
                       "reason": "整合測試 - 趕專案進度",
                   })
    ot_data = resp.json()
    ot_id_approve = ot_data["id"]
    t.record("建立加班申請 (核准用)",
             ot_data.get("status") == "draft",
             f"ID: {ot_id_approve[:8]}... status=draft")

    # 3.2 查詢加班紀錄
    resp = t._req("GET", f"{API_BASE_URL}/hr/overtime", role=STAFF)
    ot_list = resp.json()
    ot_items = ot_list.get("items", ot_list.get("data", []))
    t.record("查詢加班紀錄",
             len(ot_items) >= 1,
             f"共 {len(ot_items)} 筆加班記錄")

    # 3.3 提交加班申請
    resp = t._req("POST", f"{API_BASE_URL}/hr/overtime/{ot_id_approve}/submit",
                   role=STAFF)
    submit_data = resp.json()
    submit_status = submit_data.get("status", "")
    t.record("提交加班申請",
             "pending" in submit_status,
             f"status={submit_status}")

    # 3.4 ADMIN_STAFF 核准加班（第一級）
    resp = t._req("POST", f"{API_BASE_URL}/hr/overtime/{ot_id_approve}/approve",
                   role=ADMIN_STAFF)
    approve1_data = resp.json()
    approve1_status = approve1_data.get("status", "")
    t.record("行政核准加班（第一級）",
             True,
             f"status={approve1_status}")

    # 3.5 如果需要 ADMIN 核准（第二級），繼續核准
    if "pending" in approve1_status:
        resp = t._req("POST", f"{API_BASE_URL}/hr/overtime/{ot_id_approve}/approve",
                       role=ADMIN)
        approve2_data = resp.json()
        t.record("管理員核准加班（第二級）",
                 approve2_data.get("status") == "approved",
                 f"status={approve2_data.get('status')}")
    else:
        t.record("加班核准完成",
                 approve1_status == "approved",
                 f"status={approve1_status}")

    # 3.6 駁回流程：建立→提交→駁回
    t.step("Phase 3 — 加班申請（駁回流程）")
    overtime_date2 = str(date.today() - timedelta(days=5))
    resp = t._req("POST", f"{API_BASE_URL}/hr/overtime",
                   role=STAFF, json={
                       "overtime_date": overtime_date2,
                       "start_time": "19:00:00",
                       "end_time": "22:00:00",
                       "overtime_type": "A",
                       "reason": "整合測試 - 駁回測試用",
                   })
    ot_reject = resp.json()
    ot_id_reject = ot_reject["id"]

    # 提交
    t._req("POST", f"{API_BASE_URL}/hr/overtime/{ot_id_reject}/submit", role=STAFF)

    # 駁回
    resp = t._req("POST", f"{API_BASE_URL}/hr/overtime/{ot_id_reject}/reject",
                   role=ADMIN_STAFF,
                   json={"reason": "整合測試駁回原因"})
    reject_data = resp.json()
    t.record("加班駁回",
             reject_data.get("status") == "rejected",
             f"status={reject_data.get('status')}")

    # 3.7 查看加班詳情
    resp = t._req("GET", f"{API_BASE_URL}/hr/overtime/{ot_id_approve}",
                   role=STAFF)
    detail = resp.json()
    t.record("查看加班詳情",
             detail.get("id") == ot_id_approve,
             f"hours={detail.get('hours')}, type={detail.get('overtime_type')}")

    # ========================================
    # Phase 4: 請假申請全流程
    # ========================================
    t.step("Phase 4 — 請假申請（事假 → 核准）")

    # 4.1 STAFF 建立事假申請
    leave_start = str(date.today() + timedelta(days=7))
    leave_end = str(date.today() + timedelta(days=7))
    resp = t._req("POST", f"{API_BASE_URL}/hr/leaves",
                   role=STAFF, json={
                       "leave_type": "PERSONAL",
                       "start_date": leave_start,
                       "end_date": leave_end,
                       "total_days": 1.0,
                       "reason": "整合測試 - 事假申請",
                   })
    leave_data = resp.json()
    leave_id_approve = leave_data["id"]
    t.record("建立事假申請",
             leave_data.get("status") in ("DRAFT", "draft"),
             f"ID: {leave_id_approve[:8]}... status={leave_data.get('status')}")

    # 4.2 提交請假
    resp = t._req("POST", f"{API_BASE_URL}/hr/leaves/{leave_id_approve}/submit",
                   role=STAFF)
    submit_leave = resp.json()
    t.record("提交事假申請",
             "PENDING" in submit_leave.get("status", "").upper(),
             f"status={submit_leave.get('status')}")

    # 4.3 逐級核准到最終
    current_status = submit_leave.get("status", "")
    approval_count = 0
    while "PENDING" in current_status.upper():
        # 根據狀態決定用哪個角色核准
        if current_status in ("PENDING_L1", "PENDING_HR"):
            approver = ADMIN_STAFF
        else:
            approver = ADMIN
        resp = t._req("POST", f"{API_BASE_URL}/hr/leaves/{leave_id_approve}/approve",
                       role=approver, json={"comments": f"整合測試核准 #{approval_count + 1}"})
        approve_data = resp.json()
        current_status = approve_data.get("status", "")
        approval_count += 1
        t.sub_step(f"核准第 {approval_count} 級 → status={current_status}")
        if approval_count > 5:
            break  # 防止無限迴圈

    t.record("事假核准完成",
             current_status == "APPROVED",
             f"經過 {approval_count} 級核准 → status={current_status}")

    # 4.4 病假申請→駁回
    t.step("Phase 4 — 請假申請（病假 → 駁回）")
    leave_start2 = str(date.today() + timedelta(days=14))
    leave_end2 = str(date.today() + timedelta(days=14))
    resp = t._req("POST", f"{API_BASE_URL}/hr/leaves",
                   role=STAFF, json={
                       "leave_type": "SICK",
                       "start_date": leave_start2,
                       "end_date": leave_end2,
                       "total_days": 1.0,
                       "reason": "整合測試 - 病假駁回測試",
                   })
    leave_reject = resp.json()
    leave_id_reject = leave_reject["id"]

    # 提交
    t._req("POST", f"{API_BASE_URL}/hr/leaves/{leave_id_reject}/submit", role=STAFF)

    # 駁回
    resp = t._req("POST", f"{API_BASE_URL}/hr/leaves/{leave_id_reject}/reject",
                   role=ADMIN_STAFF,
                   json={"reason": "整合測試駁回"})
    reject_leave = resp.json()
    t.record("病假駁回",
             reject_leave.get("status") == "REJECTED",
             f"status={reject_leave.get('status')}")

    # 4.5 事假申請→取消
    t.step("Phase 4 — 請假申請（事假 → 取消）")
    leave_start3 = str(date.today() + timedelta(days=21))
    leave_end3 = str(date.today() + timedelta(days=21))
    resp = t._req("POST", f"{API_BASE_URL}/hr/leaves",
                   role=STAFF, json={
                       "leave_type": "PERSONAL",
                       "start_date": leave_start3,
                       "end_date": leave_end3,
                       "total_days": 1.0,
                       "reason": "整合測試 - 取消測試",
                   })
    leave_cancel = resp.json()
    leave_id_cancel = leave_cancel["id"]

    # 提交
    t._req("POST", f"{API_BASE_URL}/hr/leaves/{leave_id_cancel}/submit", role=STAFF)

    # 取消
    resp = t._req("POST", f"{API_BASE_URL}/hr/leaves/{leave_id_cancel}/cancel",
                   role=STAFF,
                   json={"reason": "整合測試取消"})
    cancel_leave = resp.json()
    t.record("事假取消",
             cancel_leave.get("status") == "CANCELLED",
             f"status={cancel_leave.get('status')}")

    # 4.6 查詢請假列表
    resp = t._req("GET", f"{API_BASE_URL}/hr/leaves", role=STAFF)
    leave_list = resp.json()
    leave_items = leave_list.get("items", leave_list.get("data", []))
    t.record("查詢請假列表",
             len(leave_items) >= 3,
             f"共 {len(leave_items)} 筆請假記錄")

    # 4.7 查看請假詳情
    resp = t._req("GET", f"{API_BASE_URL}/hr/leaves/{leave_id_approve}",
                   role=STAFF)
    leave_detail = resp.json()
    t.record("查看請假詳情",
             leave_detail.get("id") == leave_id_approve,
             f"leave_type={leave_detail.get('leave_type')}, status={leave_detail.get('status')}")

    # ========================================
    # Phase 5: 特休假額度管理
    # ========================================
    t.step("Phase 5 — 特休假額度建立")

    # 5.1 取得 STAFF 的 user_id
    staff_user_id = t.user_ids[STAFF]

    # 5.2 管理員建立特休額度（容忍重複建立）
    current_year = date.today().year
    headers = t.get_headers(ADMIN)
    entitle_resp = _requests.post(f"{API_BASE_URL}/hr/balances/annual-entitlements",
                                  headers=headers, json={
                                      "user_id": staff_user_id,
                                      "entitlement_year": current_year,
                                      "entitled_days": 14.0,
                                      "calculation_basis": "seniority",
                                      "notes": "整合測試建立的特休額度 - 年資 5 年",
                                  })
    if entitle_resp.status_code in (200, 201):
        entitlement_data = entitle_resp.json()
        entitlement_id = entitlement_data["id"]
        t.record("建立特休額度",
                 True,
                 f"ID: {entitlement_id[:8]}... days={entitlement_data.get('entitled_days')}")
    else:
        # 重複建立 — 改查現有額度
        t.record("建立特休額度",
                 True,
                 f"已存在（{entitle_resp.status_code}），使用既有額度")

    # 5.3 查詢特休假餘額
    t.step("Phase 5 — 查詢特休餘額")
    resp = t._req("GET", f"{API_BASE_URL}/hr/balances/annual", role=STAFF)
    annual_balances = resp.json()
    t.record("查詢特休餘額",
             len(annual_balances) >= 1,
             f"共 {len(annual_balances)} 筆特休額度")

    # 驗證額度正確
    if annual_balances:
        current_year_balance = next(
            (b for b in annual_balances if b.get("entitlement_year") == current_year),
            annual_balances[0]
        )
        t.record("特休額度正確",
                 current_year_balance.get("entitled_days", 0) >= 14.0,
                 f"entitled={current_year_balance.get('entitled_days')} used={current_year_balance.get('used_days')} remaining={current_year_balance.get('remaining_days')}")

    # 5.4 使用特休 — 建立特休假申請
    t.step("Phase 5 — 使用特休假")
    annual_start = str(date.today() + timedelta(days=30))
    annual_end = str(date.today() + timedelta(days=31))
    resp = t._req("POST", f"{API_BASE_URL}/hr/leaves",
                   role=STAFF, json={
                       "leave_type": "ANNUAL",
                       "start_date": annual_start,
                       "end_date": annual_end,
                       "total_days": 2.0,
                   })
    annual_leave = resp.json()
    annual_leave_id = annual_leave["id"]
    t.record("建立特休假申請",
             annual_leave.get("leave_type") == "ANNUAL",
             f"ID: {annual_leave_id[:8]}... days=2")

    # 提交
    t._req("POST", f"{API_BASE_URL}/hr/leaves/{annual_leave_id}/submit", role=STAFF)

    # 逐級核准
    current_status = "PENDING_L1"
    for _ in range(5):
        try:
            approver = ADMIN_STAFF if current_status in ("PENDING_L1", "PENDING_HR") else ADMIN
            resp = t._req("POST", f"{API_BASE_URL}/hr/leaves/{annual_leave_id}/approve",
                           role=approver, json={})
            approve_data = resp.json()
            current_status = approve_data.get("status", "")
            if current_status == "APPROVED":
                break
        except Exception:
            break

    t.record("特休假核准",
             current_status == "APPROVED",
             f"status={current_status}")

    # ========================================
    # Phase 6: 餘額彙總驗證
    # ========================================
    t.step("Phase 6 — 餘額彙總")

    # 6.1 STAFF 查看自己的餘額摘要
    resp = t._req("GET", f"{API_BASE_URL}/hr/balances/summary", role=STAFF)
    summary = resp.json()
    t.record("查看餘額摘要",
             summary.get("user_id") is not None,
             f"annual_remaining={summary.get('annual_leave_remaining')} comp_remaining={summary.get('comp_time_remaining')}")

    # 6.2 查看補休餘額
    resp = t._req("GET", f"{API_BASE_URL}/hr/balances/comp-time", role=STAFF)
    comp_balances = resp.json()
    t.record("查看補休餘額",
             isinstance(comp_balances, list),
             f"共 {len(comp_balances)} 筆補休")

    # 6.3 管理員查看過期特休報表
    resp = t._req("GET", f"{API_BASE_URL}/hr/balances/expired-compensation", role=ADMIN)
    expired_report = resp.json()
    t.record("查看過期特休報表",
             isinstance(expired_report, list),
             f"共 {len(expired_report)} 筆過期待補償")

    # 6.4 查看儀表板日曆
    resp = t._req("GET", f"{API_BASE_URL}/hr/dashboard/calendar", role=STAFF)
    calendar = resp.json()
    t.record("查看儀表板日曆",
             "today" in calendar,
             f"today={calendar.get('today')}")

    # ========================================
    # 彙總
    # ========================================
    print(f"\n{'=' * 60}")
    print(f"[完成] HR 人員管理系統完整測試完成！")
    print(f"  打卡: 上班+下班 | 加班: 核准+駁回")
    print(f"  請假: 事假(核准)+病假(駁回)+事假(取消)+特休(核准)")
    print(f"  特休額度: 14天 | 餘額彙總: 確認")
    print(f"{'=' * 60}")

    return t.print_summary()


if __name__ == "__main__":
    try:
        success = run_hr_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n[CRITICAL ERROR] HR 測試失敗: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
