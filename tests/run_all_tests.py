# -*- coding: utf-8 -*-
"""
整合測試統一入口

依序執行七大測試模組並輸出總報告

用法:
    cd d:\\Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/run_all_tests.py              # 全部執行
    .venv\\Scripts\\python.exe tests/run_all_tests.py --aup        # 只執行 AUP
    .venv\\Scripts\\python.exe tests/run_all_tests.py --erp        # 只執行 ERP
    .venv\\Scripts\\python.exe tests/run_all_tests.py --animal     # 只執行動物管理
    .venv\\Scripts\\python.exe tests/run_all_tests.py --blood      # 只執行血液檢驗
    .venv\\Scripts\\python.exe tests/run_all_tests.py --hr         # 只執行 HR 人員管理
    .venv\\Scripts\\python.exe tests/run_all_tests.py --amendment  # 只執行 Amendment
    .venv\\Scripts\\python.exe tests/run_all_tests.py --erp-perm   # 只執行 ERP 權限
"""

import argparse
import os
import sys
import time

# 修正 Windows 終端機的 Unicode 編碼問題 (cp950 -> utf-8)
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

# 確保 tests 目錄在 sys.path 中
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


def main():
    parser = argparse.ArgumentParser(description="iPig System 整合測試")
    parser.add_argument("--aup", action="store_true", help="只執行 AUP 測試")
    parser.add_argument("--erp", action="store_true", help="只執行 ERP 測試")
    parser.add_argument("--animal", action="store_true", help="只執行動物管理測試")
    parser.add_argument("--blood", action="store_true", help="只執行血液檢驗測試")
    parser.add_argument("--hr", action="store_true", help="只執行 HR 人員管理測試")
    parser.add_argument("--amendment", action="store_true", help="只執行 Amendment 測試")
    parser.add_argument("--erp-perm", dest="erp_perm", action="store_true", help="只執行 ERP 權限測試")
    parser.add_argument("--cleanup", action="store_true", help="測試結束後清理測試資料")
    args = parser.parse_args()

    run_all = not (args.aup or args.erp or args.animal or args.blood or args.hr or args.amendment or args.erp_perm)

    banner = r"""
============================================================
           iPig System 整合測試套件
                                                            
  1. AUP 完整審查流程測試                                    
  2. ERP 完整庫存管理測試                                    
  3. 動物管理系統完整測試                                    
  4. 血液檢驗面板完整測試                                    
  5. HR 人員管理系統完整測試                               
============================================================
"""
    print(banner)

    results = {}
    total_start = time.time()

    # ========================================
    # 1. AUP 完整審查流程測試
    # ========================================
    if run_all or args.aup:
        print("\n" + "=" * 60)
        print("[1/7] AUP 完整審查流程測試")
        print("=" * 60)
        start = time.time()
        try:
            from test_aup_full import run_aup_test
            results["AUP"] = run_aup_test()
        except Exception as e:
            print(f"\n[ERROR] AUP 測試例外: {e}")
            import traceback
            traceback.print_exc()
            results["AUP"] = False
        elapsed = time.time() - start
        print(f"  AUP 測試耗時: {elapsed:.1f} 秒")

    # ========================================
    # 2. ERP 測試
    # ========================================
    if run_all or args.erp:
        print("\n" + "=" * 60)
        print("[2/7] ERP 完整庫存管理測試（含報表驗證）")
        print("=" * 60)
        start = time.time()
        try:
            from test_erp_full import run_erp_test
            results["ERP"] = run_erp_test()
        except Exception as e:
            print(f"\n[ERROR] ERP 測試例外: {e}")
            import traceback
            traceback.print_exc()
            results["ERP"] = False
        elapsed = time.time() - start
        print(f"  ERP 測試耗時: {elapsed:.1f} 秒")

    # ========================================
    # 3. 動物管理測試
    # ========================================
    if run_all or args.animal:
        print("\n" + "=" * 60)
        print("[3/7] 動物管理系統完整測試")
        print("=" * 60)
        start = time.time()
        try:
            from test_animal_full import run_animal_test
            results["Animal"] = run_animal_test()
        except Exception as e:
            print(f"\n[ERROR] 動物管理測試例外: {e}")
            import traceback
            traceback.print_exc()
            results["Animal"] = False
        elapsed = time.time() - start
        print(f"  動物管理測試耗時: {elapsed:.1f} 秒")

    # ========================================
    # 4. 血液檢驗測試
    # ========================================
    if run_all or args.blood:
        print("\n" + "=" * 60)
        print("[4/7] 血液檢驗面板完整測試")
        print("=" * 60)
        start = time.time()
        try:
            from test_blood_panel import run_blood_panel_test
            results["Blood Panel"] = run_blood_panel_test()
        except Exception as e:
            print(f"\n[ERROR] 血液檢驗測試失敗: {e}")
            import traceback
            traceback.print_exc()
            results["Blood Panel"] = False
        elapsed = time.time() - start
        print(f"  血液檢驗測試耗時: {elapsed:.1f} 秒")

    # ========================================
    # 5. HR 人員管理測試
    # ========================================
    if run_all or args.hr:
        print("\n" + "=" * 60)
        print("[5/7] HR 人員管理系統完整測試")
        print("=" * 60)
        start = time.time()
        try:
            from test_hr_full import run_hr_test
            results["HR"] = run_hr_test()
        except Exception as e:
            print(f"\n[ERROR] HR 測試例外: {e}")
            import traceback
            traceback.print_exc()
            results["HR"] = False
        elapsed = time.time() - start
        print(f"  HR 測試耗時: {elapsed:.1f} 秒")

    # ========================================
    # 6. Amendment 完整流程測試
    # ========================================
    if run_all or args.amendment:
        print("\n" + "=" * 60)
        print("[6/7] Amendment 完整流程測試")
        print("=" * 60)
        start = time.time()
        try:
            from test_amendment_full import run_amendment_test
            results["Amendment"] = run_amendment_test()
        except Exception as e:
            print(f"\n[ERROR] Amendment 測試例外: {e}")
            import traceback
            traceback.print_exc()
            results["Amendment"] = False
        elapsed = time.time() - start
        print(f"  Amendment 測試耗時: {elapsed:.1f} 秒")

    # ========================================
    # 7. ERP 倉庫管理權限測試
    # ========================================
    if run_all or args.erp_perm:
        print("\n" + "=" * 60)
        print("[7/7] ERP 倉庫管理權限測試")
        print("=" * 60)
        start = time.time()
        try:
            from test_erp_permissions import run_erp_permissions_test
            results["ERP Permissions"] = run_erp_permissions_test()
        except Exception as e:
            print(f"\n[ERROR] ERP 權限測試例外: {e}")
            import traceback
            traceback.print_exc()
            results["ERP Permissions"] = False
        elapsed = time.time() - start
        print(f"  ERP 權限測試耗時: {elapsed:.1f} 秒")

    # ========================================
    # 彙總報告
    # ========================================
    total_elapsed = time.time() - total_start
    print("\n" + "=" * 60)
    print("  整合測試最終報告")
    print("=" * 60)
    all_passed = True
    for name, passed in results.items():
        icon = "PASS" if passed else "FAIL"
        print(f"  [{icon}]  {name}")
        if not passed:
            all_passed = False

    print(f"\n  總耗時: {total_elapsed:.1f} 秒")

    if all_passed:
        print("\n=== 全部測試通過！ ===")
    else:
        failed = [k for k, v in results.items() if not v]
        print(f"\n*** 以下測試失敗: {', '.join(failed)} ***")

    print("=" * 60)

    # ========================================
    # 測試後清理
    # ========================================
    if args.cleanup:
        from test_base import BaseApiTester
        BaseApiTester.cleanup_test_data()

    sys.exit(0 if all_passed else 1)


if __name__ == "__main__":
    main()
