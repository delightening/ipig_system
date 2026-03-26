# Code Review 推薦工具 — 2026-03-26

根據 iPig System Code Review 發現的問題，推薦以下線上工具分類導入。

---

## 1. 自動化 Code Review（CI 整合）

| 工具 | 特點 | 適合解決 |
|------|------|---------|
| **[CodeRabbit](https://www.coderabbit.ai/)** | GitHub PR 自動逐行 review，2M+ repos | 整體程式碼品質 |
| **[DeepSource](https://deepsource.com/)** | Rust + TS 支持，自動修復 + 165+ secrets 偵測 | 死代碼、`any` 型別、clone 過多 |
| **[SonarQube](https://www.sonarsource.com/products/sonarqube/)** | v25.5 起支援 Rust（85 rules + Clippy 整合） | 品質閘門、技術債追蹤 |

## 2. 安全掃描（SAST）

| 工具 | 特點 | 適合解決 |
|------|------|---------|
| **[ZeroPath](https://zeropath.com/)** | 專精 Rust SAST，100萬+ 行 | Rust 後端深度安全掃描 |
| **[Semgrep](https://semgrep.dev/)** | 40+ 語言、20K-100K 行/秒，自訂規則 | SQL injection、CSRF 邏輯 |
| **[Aikido](https://www.aikido.dev/)** | SAST + DAST + 容器 + IaC 一站式 | Docker 安全 + 依賴漏洞 |
| **[Snyk](https://snyk.io/)** | IDE 即時 + Cargo.toml/package.json SCA | CVE 依賴漏洞 |

## 3. 資料庫 Schema Review

| 工具 | 特點 | 適合解決 |
|------|------|---------|
| **[Bytebase](https://www.bytebase.com/)** | SQL review 自動檢查命名、缺索引、危險操作 | FK 索引缺失、CASCADE 風險 |
| **[pgroll](https://github.com/xataio/pgroll)** | expand/contract 零停機 migration | 安全 schema 演進 |

## 4. React / 前端專用

| 工具 | 特點 | 適合解決 |
|------|------|---------|
| **[DeepScan](https://deepscan.io/)** | 專精 JS/TS/React，GitHub PR review | hooks 問題、競態條件 |
| **[ESLint + typescript-eslint](https://typescript-eslint.io/)** | 嚴格模式抓 `any` | 型別安全 |

---

## 建議導入順序

1. **Semgrep** — 免費開源，自訂規則抓 CSRF/SQL pattern，CI 快速整合
2. **DeepSource** — Rust + TS 雙語支持，自動修復省時間
3. **Bytebase** — 解決資料庫 review 盲區
4. **CodeRabbit** — PR 自動 review 補人工審查缺口

> 專案已有 Trivy + cargo-audit + npm audit in CI，加上以上工具可覆蓋本次 review 大部分問題。
