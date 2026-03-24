# NFC 電子名片 — 使用說明

## 概述

此功能提供兩個獨立的靜態網頁：

| 檔案 | 用途 |
|------|------|
| `index.html` | 電子名片展示頁 — NFC 掃描後使用者看到的頁面 |
| `nfc-writer.html` | NFC 寫入工具 — 管理員將 URL 寫入 NFC 標籤的工具 |

## 架構決策

- **獨立靜態頁面**：放在 `frontend/public/nfc-card/` 下，不依賴 React 應用，確保 NFC 掃描後能瞬間載入
- **零外部依賴**：所有 CSS 與 JS 內嵌，無需載入任何 CDN 資源
- **響應式設計**：支援手機、平板、桌面瀏覽器
- **Web NFC API**：使用瀏覽器原生 NFC API（目前僅 Android Chrome 89+ 支援）

## 功能清單

### 名片展示頁 (`index.html`)

- 顯示姓名、職稱、公司
- 頭像（支援圖片 URL 或姓名首字）
- 電話（可直接撥打）、Email（可直接發信）、網站、地址（Google Maps）
- 社群連結（LinkedIn、GitHub、LINE、Facebook）
- **儲存聯絡人**：一鍵下載 vCard (.vcf) 檔案
- **QR Code**：產生當前頁面的 QR Code 供掃描

### NFC 寫入工具 (`nfc-writer.html`)

- **URL 模式**：寫入名片頁面 URL 至 NFC 標籤（推薦）
- **vCard 模式**：直接將 vCard 資料寫入 NFC 標籤
- **驗證功能**：讀取 NFC 標籤內容，確認寫入正確

## 使用方式

### 步驟 1：自訂名片內容

編輯 `index.html` 中的 `CARD_DATA` 物件：

```javascript
const CARD_DATA = {
  name: '你的姓名',
  jobTitle: '你的職稱',
  company: '你的公司名',
  avatarUrl: '',  // 頭像圖片 URL（留空則顯示姓名首字）
  phone: '+886-xxx-xxx-xxx',
  email: 'you@example.com',
  website: 'https://example.com',
  address: '你的地址',
  social: {
    linkedin: 'https://linkedin.com/in/xxx',
    github: 'https://github.com/xxx',
    line: 'https://line.me/ti/p/xxx',
    facebook: 'https://facebook.com/xxx',
  },
};
```

### 步驟 2：部署名片頁面

將整個 `nfc-card/` 目錄部署至可公開存取的網址，例如：
- `https://your-domain.com/nfc-card/`

### 步驟 3：寫入 NFC 標籤

1. 使用 **Android 手機的 Chrome 瀏覽器**開啟 `nfc-writer.html`
2. 確認 URL 欄位已填入正確的名片網址
3. 點擊「開始寫入 NFC」
4. 將空白 NFC 標籤靠近手機背面
5. 等待寫入成功提示

### 步驟 4：驗證

1. 在寫入工具頁面點擊「讀取 NFC 標籤」
2. 靠近 NFC 標籤確認內容正確
3. 也可以直接用手機靠近 NFC 標籤測試，應自動開啟名片頁面

## NFC 標籤建議

| 類型 | 容量 | 建議用途 |
|------|------|----------|
| NTAG213 | 144 bytes | URL 模式（推薦，便宜） |
| NTAG215 | 504 bytes | URL 模式 |
| NTAG216 | 888 bytes | vCard 模式（需較大容量） |

- **推薦**：使用 NTAG213 + URL 模式，成本低且相容性最好
- NFC 標籤可貼在名片、手機殼、鑰匙圈等物品上

## 瀏覽器相容性

| 功能 | Chrome (Android) | Safari (iOS) | 其他 |
|------|-------------------|--------------|------|
| 名片展示 | ✅ | ✅ | ✅ |
| vCard 下載 | ✅ | ✅ | ✅ |
| QR Code | ✅ | ✅ | ✅ |
| NFC 寫入 | ✅ (89+) | ❌ | ❌ |
| NFC 讀取 | ✅ (89+) | ❌ | ❌ |

> iOS 裝置掃描 NFC 標籤後可自動開啟 URL，但無法使用 Web NFC API 寫入。
> 如需在 iOS 上寫入 NFC，可使用第三方 App（如 NFC Tools）。
