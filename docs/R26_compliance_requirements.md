# R26 — 合規要求總覽（2026-04-20）

> 對應 TODO.md R26（Nice-to-have 資安強化）。本文件彙整主要合規框架對 R26 四項議題的具體條款要求，作為排程與驗收依據。

## 0. 適用決策樹

```
客戶是美國 FDA 受管機構（藥廠、CRO、GLP/GMP 實驗室）？ → 21 CFR Part 11 必要
客戶處理歐盟公民個資？                                → GDPR 必要
客戶處理美國醫療資料（PHI）？                          → HIPAA 必要
客戶要求 SaaS 供應商稽核報告？                         → SOC 2 Type II
客戶要求 ISO 認證？                                    → ISO/IEC 27001
收信用卡支付？                                         → PCI DSS
```

本專案（ipig_system）目前主要場景為 **動物試驗 / 畜牧管理**，若客戶為藥廠 GLP 實驗室 → 21 CFR Part 11 優先；若僅本地實驗 → SOC 2 Type II 作為最通用背書。

---

## 1. 法規／框架摘要

### 1.1 21 CFR Part 11（美國 FDA 電子紀錄／電子簽章）
- **適用對象**：受 FDA 監管的電子紀錄、電子簽章系統（藥廠、CRO、GLP/GCP 臨床研究）。
- **核心要求**：
  - §11.10 — 系統驗證（validation）、稽核軌跡（audit trail）、資料保留、存取控制。
  - §11.10(c) — 有能力產生準確、完整的紀錄副本，可還原供 FDA 稽核。
  - §11.100 / §11.200 — 電子簽章唯一性、不可否認性、至少兩個獨立身份驗證元件（雙因素）。
  - §11.300 — 身份識別碼／密碼管理（定期更換、失效鎖定）。
- **對應 R26 項目**：
  - R26-1（管理員強制 2FA）→ §11.200(a)(1)(ii)。
  - R26-3（備份還原演練）→ §11.10(c) 資料可還原能力。

### 1.2 SOC 2（AICPA Trust Services Criteria）
- **適用對象**：SaaS 供應商常見的第三方稽核標準，Type I（設計）/ Type II（一段期間持續運作證據）。
- **五大 TSC**：Security（必選）、Availability、Processing Integrity、Confidentiality、Privacy。
- **常用對應條款**：
  - CC6.1 — Logical access controls（帳號、權限、MFA）。
  - CC6.6 — 邊界安全、加密傳輸。
  - CC6.7 — 資料加密 at-rest。
  - CC7.1 / CC7.2 — 弱點管理、威脅偵測（含 pentest）。
  - A1.2 — 備份與還原演練。
- **對應 R26 項目**：
  - R26-1 → CC6.1。
  - R26-2 → CC6.7。
  - R26-3 → A1.2。
  - R26-4 → CC7.1 / CC7.2。

### 1.3 ISO/IEC 27001（資訊安全管理系統 ISMS）
- **適用對象**：國際通用 ISMS 認證；常被亞洲／歐洲客戶要求。
- **Annex A 控制項（2022 版）對應**：
  - A.5.15 — 存取控制政策。
  - A.8.24 — 加密使用（相當於舊版 A.10.1.1）。
  - A.8.13 — 資訊備份。
  - A.8.8 — 技術弱點管理（含 pentest）。
  - A.5.17 — 身份驗證資訊管理（MFA）。
- **對應 R26 項目**：
  - R26-1 → A.5.15 / A.5.17。
  - R26-2 → A.8.24。
  - R26-3 → A.8.13。
  - R26-4 → A.8.8。

### 1.4 GDPR（歐盟一般資料保護規則）
- **適用對象**：處理歐盟自然人個資的任何組織（不論位置）。
- **核心條款**：
  - Art. 5 — 處理原則（合法、限於目的、最小化、準確、保留限期、完整與保密）。
  - Art. 25 — Privacy by Design / by Default。
  - Art. 32 — 技術與組織措施（加密、匿名化、還原能力、定期測試）。
  - Art. 33 / 34 — 外洩通報（72 小時內）。
- **對應 R26 項目**：
  - R26-2（欄位加密）→ Art. 32(1)(a)。
  - R26-3（備份演練）→ Art. 32(1)(c) 還原能力。
  - R26-4（pentest）→ Art. 32(1)(d) 定期測試有效性。
- **額外需注意**：資料主體權利（存取／更正／刪除／可攜），本系統目前 TODO 未涵蓋，若擴展到歐盟客戶需補 DSAR 流程。

### 1.5 HIPAA（美國健康資訊保護法）
- **適用對象**：處理 PHI（Protected Health Information）的醫療機構與其業務夥伴。本專案若擴展到人類醫療資料才需要。
- **Security Rule 核心**：
  - §164.308 — 管理性保護措施（風險分析、授權管理、員工訓練）。
  - §164.310 — 實體保護措施（機房、工作站）。
  - §164.312 — 技術保護措施（存取控制、稽核、完整性、傳輸加密）。
  - §164.312(a)(2)(iv) — Encryption at rest（addressable）。
  - §164.312(e)(2)(ii) — Encryption in transit（addressable）。
- **對應 R26 項目**：
  - R26-1 → §164.312(d) Person or entity authentication。
  - R26-2 → §164.312(a)(2)(iv)。

### 1.6 PCI DSS（支付卡產業資料安全標準）
- **適用對象**：儲存、處理或傳輸信用卡卡號的系統。本專案目前**不適用**（無支付功能），列入參考。
- **若未來整合金流**：
  - Req. 3 — 儲存的卡資料加密。
  - Req. 8 — 強認證（含 MFA）。
  - Req. 11.3 — 年度外部 pentest + 重大變更後再測。

---

## 2. R26 項目 × 合規矩陣

| R26 | 21 CFR 11 | SOC 2 | ISO 27001 | GDPR | HIPAA | PCI DSS |
|-----|-----------|-------|-----------|------|-------|---------|
| R26-1 強制 2FA | §11.200(a)(1)(ii) | CC6.1 | A.5.17 | Art. 32 | §164.312(d) | Req. 8.3 |
| R26-2 欄位加密 | — | CC6.7 | A.8.24 | Art. 32(1)(a) | §164.312(a)(2)(iv) | Req. 3.4 |
| R26-3 備份演練 | §11.10(c) | A1.2 | A.8.13 | Art. 32(1)(c) | §164.308(a)(7) | Req. 12.10.1 |
| R26-4 外部 pentest | — | CC7.1/7.2 | A.8.8 | Art. 32(1)(d) | §164.308(a)(8) | Req. 11.3 |

---

## 3. 建議排程觸發條件

| 觸發情境 | 建議啟動 R26 項目 |
|----------|-------------------|
| 客戶為 CRO / 藥廠 GLP 實驗室 | R26-1、R26-3（21 CFR Part 11 必要） |
| 簽大型 SaaS 合約要求稽核 | R26-1、R26-3、R26-4（SOC 2 Type II 準備） |
| 擴展歐盟市場 | R26-2、R26-3、R26-4 + 補 DSAR 流程（GDPR） |
| 接受政府標案要求 ISO 認證 | R26 全部四項（ISO 27001 ISMS 認證前準備） |
| 年度預算允許且客戶詢問 | R26-4（年度 pentest 作為信任背書） |

---

## 4. 參考資料

- FDA 21 CFR Part 11: https://www.accessdata.fda.gov/scripts/cdrh/cfdocs/cfcfr/CFRSearch.cfm?CFRPart=11
- AICPA SOC 2 / TSC: https://www.aicpa.org/interestareas/frc/assuranceadvisoryservices/aicpasoc2report.html
- ISO/IEC 27001:2022 標準: https://www.iso.org/standard/27001
- GDPR 全文: https://gdpr-info.eu/
- HIPAA Security Rule: https://www.hhs.gov/hipaa/for-professionals/security/laws-regulations/
- PCI DSS v4.0: https://www.pcisecuritystandards.org/

---

## 5. 變更紀錄

- 2026-04-20 初版建立（隨 R26 TODO 新增）
