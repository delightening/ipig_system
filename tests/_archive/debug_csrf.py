#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""除錯 CSRF cookie 傳遞問題 — 使用 SharedTestContext"""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from test_base import BaseApiTester, API_BASE_URL
from test_context import SharedTestContext
from test_fixtures import AUP_ROLES

ctx = SharedTestContext()
print("=== Initialize with AUP roles ===")
ok = ctx.initialize(AUP_ROLES)
print(f"Init result: {ok}")

if ok:
    print(f"\nMaster csrf_token: {ctx._master_tester.csrf_token}")
    print("Master session cookies:")
    for c in ctx._master_tester.session.cookies:
        val = c.value[:30] if c.value else "None"
        print(f"  name={c.name} value={val}... domain={c.domain} path={c.path}")

    # Create child tester and inject
    t = BaseApiTester("AUP 除錯")
    ctx.inject_into(t, AUP_ROLES)

    print(f"\nChild csrf_token: {t.csrf_token}")
    print("Child session cookies:")
    for c in t.session.cookies:
        val = c.value[:30] if c.value else "None"
        print(f"  name={c.name} value={val}... domain={c.domain} path={c.path}")

    # Try POST request (requires CSRF)
    print(f"\n=== POST /protocols (requires CSRF) ===")
    headers = t.get_headers("PI")
    print(f"X-CSRF-Token: {headers.get('X-CSRF-Token', 'NONE')[:30]}...")
    resp = t.session.post(
        f"{API_BASE_URL}/protocols",
        json={"title": "CSRF debug", "animal_species": "mouse"},
        headers=headers
    )
    print(f"Status: {resp.status_code}")
    if resp.status_code >= 400:
        print(f"Response: {resp.text[:300]}")
