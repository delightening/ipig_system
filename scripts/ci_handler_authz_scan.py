#!/usr/bin/env python3
"""Handler Object-Level Authorization Scan — ADVISORY（非阻擋）。R75-P4 / R66-D2 Phase 1。

定位（2026-06-19 使用者裁定方向 1）：純文字掃描在本架構**無法成為乾淨硬 gate**——
object-level 授權合法地分散在 handler(`access::require_*`) / service(`*Service::ensure_/check_`) /
角色型 by-design 三層，文字掃描跟不進 service。故本掃描**僅產 triage worklist，永遠 exit 0**，
不阻擋 CI。硬 gate 由 `ci_handler_security_scan.sh`（窄 `_current_user` 偵測）承擔；
真正的編譯期根治由 Phase 2 型別化（`Scoped<T>`）承擔。

原理（per-function 切分、放寬 object-level pattern、排除內部 by-design 模組）：
  routes/*.rs → handler→{HTTP method}；handlers/**/*.rs 逐 `pub async fn` 切分，
  旗標「帶 Path 物件 id 的 mutation/單筆讀 + body 無 object-level 守衛 + 非內部模組」。

用法：python3 scripts/ci_handler_authz_scan.py   (一律回 0；輸出供人工 triage / Phase 2 取材)
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HANDLERS_DIR = ROOT / "backend" / "src" / "handlers"
ROUTES_DIR = ROOT / "backend" / "src" / "routes"

# object-level 守衛（放寬版）：handler 層 access::require_* + service 層慣例
# (*Service::ensure_/check_/can_ + *_access( + check_is_pi + is_assigned*)。
# 命中任一即視為「該 handler 某層有做物件級把關」，不再旗標。
OBJECT_LEVEL_RE = re.compile(
    r"access::|::ensure_|::check_|::can_|_access\(|check_is_pi|::is_assigned"
    r"|require_.*_access|require_protocol|require_animal|require_amendment"
    r"|require_vet_patrol|require_iacuc|require_reauth"
)

# 內部 by-design 模組（R75-0/3/6 釘死：角色 gated、非外部 CLIENT/PI 可達、非跨客戶）。
# 排除以維持 advisory 信號聚焦在「外部可達 / 物件擁有權有意義」的 family。
# 以「模組主檔名」（不含 .rs）比對，與 module_name() 的 rs.stem 一致；
# 同時相容未來巢狀化（handlers/glp_compliance/record.rs → parent dir "glp_compliance"）。
INTERNAL_BY_DESIGN = {
    "glp_compliance", "equipment", "facility", "qa_plan", "warehouse",
    "storage_location", "product", "sku", "partner", "audit", "role",
    "ip_blocklist", "treatment_drug", "notification_routing", "mcp_keys",
    "calendar", "ai", "user", "invitation", "training", "notification",
    "user_preferences",
}

PATH_RE = re.compile(r"Path\s*[<(]")
FUNC_RE = re.compile(r"pub\s+async\s+fn\s+([a-zA-Z0-9_]+)")
MUTATION_METHODS = {"post", "put", "patch", "delete"}
CALL_RE = re.compile(r"\.?(get|post|put|patch|delete)\(\s*(?:handlers::)?([a-zA-Z0-9_:]+)")


def build_route_method_map() -> dict[str, set[str]]:
    # 以 bare fn 名為 key。少數跨模組同名 handler（如 download_attachment）會合併 method 集合，
    # 但合併僅「增加」method → 至多造成過度旗標（advisory 已知偽陽性多、供人工 triage），
    # 絕不會把 mutation 漏成未檢查而藏住真漏洞。故維持 bare-name，不為此複雜化（CodeRabbit nitpick）。
    mapping: dict[str, set[str]] = {}
    for rs in ROUTES_DIR.glob("*.rs"):
        for method, target in CALL_RE.findall(rs.read_text(encoding="utf-8")):
            mapping.setdefault(target.split("::")[-1], set()).add(method)
    return mapping


def split_functions(text: str) -> list[tuple[str, str]]:
    matches = list(FUNC_RE.finditer(text))
    return [
        (m.group(1), text[m.start():(matches[i + 1].start() if i + 1 < len(matches) else len(text))])
        for i, m in enumerate(matches)
    ]


def module_name(rs: Path) -> str:
    return rs.stem if rs.parent.name == "handlers" else rs.parent.name


def scan() -> int:
    route_methods = build_route_method_map()
    findings: dict[str, list[str]] = {}
    checked = 0

    for rs in sorted(HANDLERS_DIR.rglob("*.rs")):
        mod = module_name(rs)
        for name, block in split_functions(rs.read_text(encoding="utf-8")):
            methods = route_methods.get(name, set())
            if not methods or not PATH_RE.search(block):
                continue
            if not (methods & MUTATION_METHODS or "get" in methods):
                continue
            checked += 1
            if OBJECT_LEVEL_RE.search(block) or mod in INTERNAL_BY_DESIGN:
                continue
            key = rs.relative_to(HANDLERS_DIR).as_posix()
            findings.setdefault(key, []).append(name)

    total = sum(len(v) for v in findings.values())
    print("=== Handler Object-Level Authorization Scan (ADVISORY · 非阻擋) ===\n")
    print(f"檢查 {checked} 個帶 Path 物件 id 的 mutation/讀單筆 handler；排除內部 by-design 模組。")
    print(f"殘留待人工 triage（手動讀 service 判定 real-gap vs delegated）：{total} 個\n")
    for key in sorted(findings, key=lambda k: -len(findings[k])):
        fns = findings[key]
        print(f"  {len(fns):2d}  {key}: {', '.join(fns[:6])}{' …' if len(fns) > 6 else ''}")
    print(
        "\n注意：本清單含大量 service-delegated 偽陽性（如 hr 假單/messaging 按 user/thread scope、"
        "R75-11 resolve_review_comment 守衛在 service）。**非待修清單**，僅供 Phase 2 型別化取材與人工複查。"
    )
    return 0  # ADVISORY：永不阻擋 CI


if __name__ == "__main__":
    sys.exit(scan())
