# Security Policy

## 支援版本 (Supported Versions)

| 版本 | 支援狀態 |
|------|----------|
| `main`（最新） | ✅ 持續接收安全更新 |
| 其他分支 | ❌ 不支援 |

本系統為單一部署的內部系統，無多版本並行維護。

---

## 回報安全漏洞 (Reporting a Vulnerability)

**請勿**在公開 GitHub Issue 中回報安全漏洞，以避免尚未修補前遭到利用。

### 回報管道

**首選：GitHub Security Advisories（私下、加密）**

→ [https://github.com/delightening/ipig_system/security/advisories/new](https://github.com/delightening/ipig_system/security/advisories/new)

### 回報格式

請在 Advisory 中提供：

```text
漏洞類型：（例：SQL injection、XSS、IDOR、auth bypass、RCE）
影響元件：（例：backend/src/handlers/animal.rs、frontend/src/pages/...）
影響版本：（例：main branch as of 2026-05-XX）
嚴重程度（自評）：（Critical / High / Medium / Low）
重現步驟：（含 PoC 程式碼或截圖）
潛在影響：（資料洩漏？未授權操作？合規性影響？）
建議修復方向：（選填）
```

### 回應時間承諾

| 階段 | 目標時間 |
|------|----------|
| 初步確認收到 Advisory | 2 個工作日內 |
| 漏洞評估結果回覆 | 5 個工作日內 |
| 高危漏洞修補（CVSS ≥ 7.0） | 14 天內 |
| 中低危漏洞修補（CVSS < 7.0） | 30 天內 |

修補發布後，回報者優先獲知。我們對負責任的漏洞揭露表示感謝。

---

## 自動化安全掃描 (Automated Security Scanning)

本專案在 CI 中自動執行以下安全掃描（見 [.github/workflows/ci.yml](.github/workflows/ci.yml)）：

| 工具 | 掃描範圍 |
|------|----------|
| **cargo audit** | Rust 依賴 — RustSec Advisory DB |
| **cargo deny** | Rust 依賴 — CVE + License 合規 |
| **gitleaks** | Git history — Secret / credentials 洩漏 |
| **Trivy** | Container image — OS 套件與應用層 CVE |
| **Dependabot** | 依賴自動更新（每週，高危 major 單獨 PR） |

運行時防護：
- **ModSecurity CRS v4**（Nginx WAF，OWASP Core Rule Set）
- **Cloudflare Tunnel**（不對外直接暴露端口）
- **Rate limiting**（API 層 + Nginx 層）

---

## 合規安全要求 (Compliance Security Requirements)

本系統處理實驗動物資料與人事資料，須符合：

| 法規 | 安全要求 |
|------|----------|
| **GLP（良好實驗室規範）** | 稽核記錄完整性、資料不可竄改 |
| **21 CFR Part 11** | 電子簽章、存取控制、稽核追蹤 |
| **IACUC** | 動物實驗資料存取授權控制 |

相關文件：
- [docs/glp/traceability-matrix.md](docs/glp/traceability-matrix.md) — 條款雙向追溯

下列文件不隨本公開 repo 發布，可向維運者索取：

- `docs/security/THREAT_MODEL.md` — STRIDE 威脅模型
- `docs/security/ELECTRONIC_SIGNATURE_COMPLIANCE.md` — 電子簽章合規
- `docs/security/DATA_RETENTION_POLICY.md` — 資料保留政策

---

## 已知安全設定 (Security Configuration Notes)

- **JWT blacklist**：登出後 token 立即失效（Redis-backed blacklist）
- **TOTP 2FA**：電子簽章操作強制 2FA 重新驗證
- **GeoIP + Honeypot**：異常地區存取與探測行為記錄至 audit trail
- **CSP**：strict `script-src 'self'`（禁止 `unsafe-eval`），詳見 PROGRESS.md R58

