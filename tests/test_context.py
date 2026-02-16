# -*- coding: utf-8 -*-
"""
共享測試 Context

跨測試模組的共享 context，統一管理 token/帳號/共用資料。
當透過 run_all_tests.py 批次執行時，只需登入一次。
"""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester
from test_fixtures import ALL_TEST_USERS, get_users_for_roles


class SharedTestContext:
    """跨測試模組的共享 context，統一管理 token/帳號/共用資料"""

    def __init__(self):
        self._master_tester = BaseApiTester("共享 Context 初始化")
        self.tokens = {}        # role_label -> access_token
        self.user_ids = {}      # role_label -> user_id
        self.shared_data = {}   # 共用測試資料（如 approved_protocol_id, animal_ids）
        self._initialized_roles = set()

    def initialize(self, required_roles: list[str]) -> bool:
        """
        一次性初始化：管理員登入 + 建立/登入所有需要的帳號。

        可以多次呼叫，已初始化的角色會被跳過。
        """
        # 過濾出尚未初始化的角色
        new_roles = [r for r in required_roles if r not in self._initialized_roles]
        if not new_roles:
            print(f"  ✓ 所有 {len(required_roles)} 個角色已就緒，跳過初始化")
            return True

        # 確保管理員已登入
        if not self._master_tester.admin_token:
            if not self._master_tester.admin_login():
                return False
            if not self._master_tester.fetch_roles():
                return False

        users_config = get_users_for_roles(new_roles)
        if not users_config:
            print(f"  ✗ 找不到角色定義: {new_roles}")
            return False

        # 建立/登入帳號
        if not self._master_tester.setup_test_users(users_config):
            return False
        if not self._master_tester.login_all(users_config):
            return False

        # 儲存 token 和 user_id
        for role in new_roles:
            if role in self._master_tester.tokens:
                self.tokens[role] = self._master_tester.tokens[role]
            if role in self._master_tester.user_ids:
                self.user_ids[role] = self._master_tester.user_ids[role]

        self._initialized_roles.update(new_roles)
        return True

    def inject_into(self, tester: BaseApiTester, role_list: list[str]):
        """
        將對應角色的 token/user_id 注入到測試用的 tester 實例。

        注入後 tester 可直接使用 tester.get_headers(role) 而不用再登入。
        """
        for role in role_list:
            if role in self.tokens:
                tester.tokens[role] = self.tokens[role]
            if role in self.user_ids:
                tester.user_ids[role] = self.user_ids[role]

        # 共享 admin_token、csrf_token 和 role_map
        if self._master_tester.admin_token and not tester.admin_token:
            tester.admin_token = self._master_tester.admin_token
        if self._master_tester.csrf_token and not tester.csrf_token:
            tester.csrf_token = self._master_tester.csrf_token
        if self._master_tester.role_map and not tester.role_map:
            tester.role_map = self._master_tester.role_map.copy()

        # 共享 CSRF cookie — 使用已儲存的 csrf_token 字串
        # 注意：不從 cookie jar 取得，避免多帳號登入後累積多個同名 cookie 導致 CookieConflictError
        csrf_value = self._master_tester.csrf_token
        if csrf_value:
            # 先清除舊的 csrf_token cookie，再設定新的
            tester.session.cookies.set("csrf_token", None)
            tester.session.cookies.set("csrf_token", csrf_value)

    def get_shared(self, key: str, default=None):
        """取得共用測試資料"""
        return self.shared_data.get(key, default)

    def set_shared(self, key: str, value):
        """設定共用測試資料"""
        self.shared_data[key] = value

    def summary(self):
        """列印共享 context 狀態"""
        print(f"\n{'=' * 60}")
        print(f"[SharedTestContext] 狀態摘要")
        print(f"{'=' * 60}")
        print(f"  已初始化角色: {len(self._initialized_roles)} 個")
        print(f"  已取得 token: {len(self.tokens)} 個")
        print(f"  共用資料: {list(self.shared_data.keys())}")
        print(f"{'=' * 60}")
