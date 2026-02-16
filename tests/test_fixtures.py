# -*- coding: utf-8 -*-
"""
測試帳號統一註冊表

所有測試模組共用的帳號定義集中在此，避免各測試重複定義相同帳號。
每個測試模組透過 ROLE 子集宣告自己需要的帳號。
"""

import os
from dotenv import load_dotenv

load_dotenv()

TEST_USER_PASSWORD = os.getenv("TEST_USER_PASSWORD", "password123")
ADMIN_CREDENTIALS = {
    "email": os.getenv("TEST_ADMIN_EMAIL", "admin@example.com"),
    "password": os.getenv("TEST_ADMIN_PASSWORD", "changeme"),
}

# ============================================================
# 全域帳號註冊表 — 所有測試共用
# ============================================================
ALL_TEST_USERS = {
    # --- AUP / Amendment 共用 ---
    "IACUC_STAFF": {
        "email": "staff_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "IACUC Staff (整合測試)",
        "role_codes": ["IACUC_STAFF"],
    },
    "REVIEWER1": {
        "email": "rev1_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "Reviewer 1 (整合測試)",
        "role_codes": ["REVIEWER"],
    },
    "REVIEWER2": {
        "email": "rev2_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "Reviewer 2 (整合測試)",
        "role_codes": ["REVIEWER"],
    },
    "REVIEWER3": {
        "email": "rev3_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "Reviewer 3 (整合測試)",
        "role_codes": ["REVIEWER"],
    },
    "IACUC_CHAIR": {
        "email": "chair_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "IACUC Chair (整合測試)",
        "role_codes": ["REVIEWER", "IACUC_CHAIR"],
    },
    "PI": {
        "email": "pi_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "PI (整合測試)",
        "role_codes": ["PI"],
    },
    "VET": {
        "email": "vet_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "VET (整合測試)",
        "role_codes": ["VET"],
    },
    "REV_OTHER": {
        "email": "revother_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "Reviewer Other (整合測試)",
        "role_codes": ["REVIEWER"],
    },

    # --- AUP Integration 額外帳號 ---
    "PI_A": {
        "email": "pia_integ@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "PI-A (整合測試)",
        "role_codes": ["PI"],
    },
    "PI_B": {
        "email": "pib_integ@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "PI-B (整合測試)",
        "role_codes": ["PI"],
    },
    "EXP_STAFF": {
        "email": "expstaff_integ@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "試驗人員 (整合測試)",
        "role_codes": ["EXPERIMENT_STAFF"],
    },

    # --- Animal / Blood 共用 ---
    "VET_ANIMAL": {
        "email": "vet_animal_int@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "獸醫師 (動物整合測試)",
        "role_codes": ["VET"],
    },
    "EXPERIMENT_STAFF_ANIMAL": {
        "email": "exp_animal_int@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "試驗工作人員 (動物整合測試)",
        "role_codes": ["EXPERIMENT_STAFF"],
    },

    # --- Blood Panel 額外帳號 ---
    "VET_BLOOD": {
        "email": "vet_blood_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "獸醫師 (血液檢查測試)",
        "role_codes": ["VET"],
    },
    "EXPERIMENT_STAFF_BLOOD": {
        "email": "exp_blood_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "試驗工作人員 (血液檢查測試)",
        "role_codes": ["EXPERIMENT_STAFF"],
    },

    # --- ERP ---
    "WAREHOUSE_MANAGER": {
        "email": "wm_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "倉庫管理員 (整合測試)",
        "role_codes": ["WAREHOUSE_MANAGER"],
    },
    "ADMIN_STAFF_ERP": {
        "email": "as_int_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "行政人員 (整合測試)",
        "role_codes": ["ADMIN_STAFF"],
    },

    # --- ERP Permissions 專用 ---
    "WAREHOUSE_MANAGER_PERM": {
        "email": "wm_perm_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "倉庫管理員 (權限測試)",
        "role_codes": ["WAREHOUSE_MANAGER"],
    },
    "ADMIN_STAFF_PERM": {
        "email": "admin_perm_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "行政人員 (權限測試)",
        "role_codes": ["ADMIN_STAFF"],
    },
    "EXPERIMENT_STAFF_PERM": {
        "email": "exp_perm_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "試驗工作人員 (權限測試)",
        "role_codes": ["EXPERIMENT_STAFF"],
    },

    # --- HR ---
    "ADMIN_HR": {
        "email": ADMIN_CREDENTIALS["email"],
        "password": ADMIN_CREDENTIALS["password"],
        "display_name": "系統管理員",
        "role_codes": ["ADMIN"],
    },
    "ADMIN_STAFF_HR": {
        "email": "hr_admin_staff_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "行政人員 (HR 測試)",
        "role_codes": ["ADMIN_STAFF"],
    },
    "EXPERIMENT_STAFF_HR": {
        "email": "hr_exp_staff_test@example.com",
        "password": TEST_USER_PASSWORD,
        "display_name": "試驗工作人員 (HR 測試)",
        "role_codes": ["EXPERIMENT_STAFF"],
    },

    # --- Blood Panel ADMIN 角色 ---
    "ADMIN_BLOOD": {
        "email": ADMIN_CREDENTIALS["email"],
        "password": ADMIN_CREDENTIALS["password"],
        "display_name": "系統管理員",
        "role_codes": ["ADMIN"],
    },
}


# ============================================================
# 每個測試模組的角色子集
# ============================================================

AUP_ROLES = [
    "IACUC_STAFF", "REVIEWER1", "REVIEWER2", "REVIEWER3",
    "IACUC_CHAIR", "PI", "VET", "REV_OTHER",
]

AMENDMENT_ROLES = [
    "IACUC_STAFF", "REVIEWER1", "REVIEWER2", "REVIEWER3",
    "IACUC_CHAIR", "PI", "VET",
]

ANIMAL_ROLES = ["VET_ANIMAL", "EXPERIMENT_STAFF_ANIMAL"]

BLOOD_PANEL_ROLES = ["ADMIN_BLOOD", "VET_BLOOD", "EXPERIMENT_STAFF_BLOOD"]

ERP_ROLES = ["WAREHOUSE_MANAGER", "ADMIN_STAFF_ERP"]

ERP_PERM_ROLES = ["WAREHOUSE_MANAGER_PERM", "ADMIN_STAFF_PERM", "EXPERIMENT_STAFF_PERM"]

HR_ROLES = ["ADMIN_HR", "ADMIN_STAFF_HR", "EXPERIMENT_STAFF_HR"]

AUP_INTEGRATION_ROLES = [
    "PI_A", "PI_B", "IACUC_STAFF", "IACUC_CHAIR",
    "REVIEWER1", "REVIEWER2", "REVIEWER3",
    "EXP_STAFF", "REV_OTHER", "VET",
]


def get_users_for_roles(role_list: list[str]) -> dict:
    """從 ALL_TEST_USERS 中取得指定角色的帳號子集"""
    return {role: ALL_TEST_USERS[role] for role in role_list if role in ALL_TEST_USERS}
