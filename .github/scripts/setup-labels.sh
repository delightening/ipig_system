#!/usr/bin/env bash
# 初始化 GitHub Labels 標準集
# 執行前提：gh CLI 已登入（gh auth login）且有 repo 管理權限
# 用法：bash .github/scripts/setup-labels.sh <owner>/<repo>
# 例：bash .github/scripts/setup-labels.sh delightening/ipig_system

set -euo pipefail

REPO="${1:-delightening/ipig_system}"

create_label() {
  local name="$1" color="$2" description="$3"
  if gh label list --repo "$REPO" --limit 100 --json name --jq '.[].name' | grep -qx "$name"; then
    gh label edit "$name" --repo "$REPO" --color "$color" --description "$description"
    echo "  ✔ updated: $name"
  else
    gh label create "$name" --repo "$REPO" --color "$color" --description "$description"
    echo "  ✔ created: $name"
  fi
}

echo "=== 設定 $REPO 的 GitHub Labels ==="

echo ""
echo "── 類型 (Type) ──"
create_label "bug"            "d73a4a" "回報問題與錯誤"
create_label "enhancement"   "a2eeef" "新功能或現有功能改善"
create_label "documentation" "0075ca" "文件新增或修改"
create_label "refactor"      "e4e669" "重構（無功能變更）"
create_label "security"      "b60205" "資安修補"
create_label "compliance"    "6f42c1" "GLP / 21 CFR Part 11 相關"
create_label "infrastructure" "cfd3d7" "CI/CD、Docker、監控、依賴更新"
create_label "performance"   "0e8a16" "效能優化"
create_label "migration"     "c2e0c6" "資料庫 migration"

echo ""
echo "── 優先級 (Priority) ──"
create_label "P0-critical"   "ee0701" "系統無法使用 / 資料遺失 / 立即修復"
create_label "P1-high"       "d73a4a" "核心功能異常，高優先"
create_label "P2-medium"     "fbca04" "中優先，月內處理"
create_label "P3-low"        "0075ca" "低優先，季內處理"

echo ""
echo "── 狀態 (Status) ──"
create_label "needs-triage"  "e4e4e4" "待評估與分類"
create_label "in-progress"   "1d76db" "進行中"
create_label "blocked"       "e11d48" "被阻擋，等待外部依賴"
create_label "wont-fix"      "ffffff" "確認不修復"
create_label "duplicate"     "cfd3d7" "與現有 issue 重複"
create_label "stale"         "fef2c0" "長時間無活動"

echo ""
echo "── 範圍 (Area) ──"
create_label "area:backend"  "c5def5" "後端 Rust/Axum"
create_label "area:frontend" "bfd4f2" "前端 React/TypeScript"
create_label "area:database" "d4edda" "資料庫 schema / migration"
create_label "area:ci-cd"    "fff3cd" "CI/CD、GitHub Actions"
create_label "area:monitoring" "e2d9f3" "Prometheus / Grafana / 告警"
create_label "area:deployment" "fce8e6" "部署、Docker、Cloudflare"

echo ""
echo "=== 完成 ==="
