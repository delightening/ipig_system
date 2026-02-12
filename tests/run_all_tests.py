"""
整合測試統一入口

依序執行三大測試模組並輸出彙總報告。

用法：
    cd d:\\Coding\\ipig_system
    .venv\\Scripts\\python.exe tests/run_all_tests.py           # 全跑
    .venv\\Scripts\\python.exe tests/run_all_tests.py --aup     # 只跑 AUP
    .venv\\Scripts\\python.exe tests/run_all_tests.py --erp     # 只跑 ERP
    .venv\\Scripts\\python.exe tests/run_all_tests.py --animal  # 只跑動物管理
"""

import sys
import os
import time
import argparse

# 修正 Windows 終端機的 Unicode 編碼問題
sys.stdout.reconfigure(encoding="utf-8")
sys.stderr.reconfigure(encoding="utf-8")

# 確保 tests 目錄在 path 中
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))


def main():
    parser = argparse.ArgumentParser(description="iPig System 整合測試")
    parser.add_argument("--aup", action="store_true", help="只執行 AUP 測試")
    parser.add_argument("--erp", action="store_true", help="只執行 ERP 測試")
    parser.add_argument("--animal", action="store_true", help="只執行動物管理測試")
    parser.add_argument("--cleanup", action="store_true", help="測試結束後清理測試資料（保留審計記錄）")
    args = parser.parse_args()

    # 如果沒指定任何 flag，就全部跑
    run_all = not (args.aup or args.erp or args.animal)

    results = {}
    total_start = time.time()

    banner = """
╔══════════════════════════════════════════════════════════╗
║            iPig System — 整合測試套件                  ║
║                                                          ║
║   1. AUP 完整審查流程測試                               ║
║   2. ERP 完整倉庫管理測試                               ║
║   3. 動物管理系統完整測試                                ║
╚══════════════════════════════════════════════════════════╝
"""
    print(banner)

    # ========================================
    # 1. AUP 測試
    # ========================================
    if run_all or args.aup:
        print("\n" + "█" * 60)
        print("█  [1/3] AUP 完整審查流程測試")
        print("█" * 60)
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
        print(f"  ⏱ AUP 測試耗時: {elapsed:.1f} 秒")

    # ========================================
    # 2. ERP 測試
    # ========================================
    if run_all or args.erp:
        print("\n" + "█" * 60)
        print("█  [2/3] ERP 完整倉庫管理測試")
        print("█" * 60)
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
        print(f"  ⏱ ERP 測試耗時: {elapsed:.1f} 秒")

    # ========================================
    # 3. 動物管理測試
    # ========================================
    if run_all or args.animal:
        print("\n" + "█" * 60)
        print("█  [3/3] 動物管理系統完整測試")
        print("█" * 60)
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
        print(f"  ⏱ 動物管理測試耗時: {elapsed:.1f} 秒")

    # ========================================
    # 彙總報告
    # ========================================
    total_elapsed = time.time() - total_start
    print("\n" + "═" * 60)
    print("  整合測試最終報告")
    print("═" * 60)
    all_passed = True
    for name, passed in results.items():
        icon = "✅" if passed else "❌"
        print(f"  {icon}  {name}")
        if not passed:
            all_passed = False

    print(f"\n  ⏱ 總耗時: {total_elapsed:.1f} 秒")

    if all_passed:
        print("\n🎉🎉🎉 全部測試通過！")
    else:
        failed = [k for k, v in results.items() if not v]
        print(f"\n⚠️ 以下測試失敗: {', '.join(failed)}")

    print("═" * 60)

    # ========================================
    # 測試後清理
    # ========================================
    if args.cleanup:
        from test_base import BaseApiTester
        BaseApiTester.cleanup_test_data()

    sys.exit(0 if all_passed else 1)


if __name__ == "__main__":
    main()
