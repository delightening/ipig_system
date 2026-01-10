# 豬博士 iPig 角色權限規格書

## 1. 帳號管理機制

### 1.1 私域註冊（非公開註冊）

本系統採用**私域註冊**機制，不提供公開註冊功能：

- ❌ 無公開註冊頁面
- ✅ 由公司內部管理員（SYSTEM_ADMIN）建立帳號
- ✅ 帳號建立後，系統自動寄送通知信至使用者 Email

---

### 1.2 帳號建立流程

```
管理員建立帳號
       │
       ▼
┌─────────────────────────────┐
│ 填寫使用者資料               │
│ • Email（必填，作為登入帳號）│
│ • 姓名                      │
│ • 角色指派                  │
│ • 所屬機構                  │
│ • 關聯計畫（外部人員適用）    │
└─────────────────────────────┘
       │
       ▼
┌─────────────────────────────┐
│ 系統產生初始密碼             │
│ （隨機產生或管理員指定）      │
└─────────────────────────────┘
       │
       ▼
┌─────────────────────────────┐
│ 寄送 Email 通知信            │
│ • 帳號（Email）              │
│ • 初始密碼                   │
│ • 登入網址                   │
│ • 首次登入須變更密碼提示      │
└─────────────────────────────┘
       │
       ▼
┌─────────────────────────────┐
│ 使用者首次登入               │
│ • 強制變更密碼               │
│ • 完成帳號啟用               │
└─────────────────────────────┘
```

---

### 1.3 Email 發送設定（Gmail SMTP）

**SMTP 設定：**

| 設定項目 | 值 |
|---------|---|
| SMTP Server | smtp.gmail.com |
| Port | 587 (TLS) 或 465 (SSL) |
| 帳號 | 從環境變數 `SMTP_USERNAME` 讀取 |
| 密碼 | 從環境變數 `SMTP_PASSWORD` 讀取（App Password） |
| 寄件者名稱 | 豬博士動物科技 |

**環境變數設定（.env）：**
```env
# Email SMTP Configuration
# ⚠️ 重要：所有敏感資訊請透過環境變數設定，不要直接寫在程式碼中
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password-here
SMTP_FROM_NAME=豬博士動物科技
SMTP_FROM_EMAIL=your-email@gmail.com
```

**注意事項：**
- 使用 Gmail 時，需要在 Google 帳號設定中產生「應用程式密碼」（App Password）
- 所有 SMTP 相關的敏感資訊都應透過 `.env` 檔案設定，不要硬編碼在程式碼中
- `.env` 檔案必須加入 `.gitignore`，避免提交到版本控制系統

**Rust 範例（使用 lettre）：**
```rust
use lettre::{
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub async fn send_registration_email(
    to_email: &str,
    to_name: &str,
    password: &str,
    login_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 從環境變數讀取 SMTP 設定（不要硬編碼）
    let smtp_user = std::env::var("SMTP_USERNAME")
        .expect("SMTP_USERNAME must be set in environment");
    let smtp_pass = std::env::var("SMTP_PASSWORD")
        .expect("SMTP_PASSWORD must be set in environment");
    let smtp_host = std::env::var("SMTP_HOST")
        .unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port: u16 = std::env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .unwrap_or(587);
    let from_email = std::env::var("SMTP_FROM_EMAIL")
        .expect("SMTP_FROM_EMAIL must be set in environment");
    let from_name = std::env::var("SMTP_FROM_NAME")
        .unwrap_or_else(|_| "豬博士動物科技".to_string());
    
    let email = Message::builder()
        .from(format!("{} <{}>", from_name, from_email).parse()?)
        .to(format!("{} <{}>", to_name, to_email).parse()?)
        .subject("豬博士 iPig 系統帳號開通通知")
        .body(format!(
            r#"您好 {}，

您的豬博士 iPig 系統帳號已開通：

【登入資訊】
帳號：{}
初始密碼：{}
登入網址：{}

請於首次登入後立即變更密碼。
如有任何問題，請聯繫工作人員（電話：037-433789）。

豬博士動物科技有限公司"#,
            to_name, to_email, password, login_url
        ))?;

    let creds = Credentials::new(smtp_user, smtp_pass);
    
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)?
        .port(smtp_port)
        .credentials(creds)
        .build();

    mailer.send(email).await?;
    Ok(())
}
```

**Node.js 範例（使用 nodemailer）：**
```javascript
const nodemailer = require('nodemailer');

// 從環境變數讀取 SMTP 設定（不要硬編碼）
const transporter = nodemailer.createTransport({
  host: process.env.SMTP_HOST || 'smtp.gmail.com',
  port: parseInt(process.env.SMTP_PORT || '587'),
  secure: false, // true for 465, false for other ports
  auth: {
    user: process.env.SMTP_USERNAME,
    pass: process.env.SMTP_PASSWORD,
  },
});

async function sendRegistrationEmail(toEmail, toName, password, loginUrl) {
  const fromName = process.env.SMTP_FROM_NAME || '豬博士動物科技';
  const fromEmail = process.env.SMTP_FROM_EMAIL || process.env.SMTP_USERNAME;
  
  await transporter.sendMail({
    from: `"${fromName}" <${fromEmail}>`,
    to: toEmail,
    subject: '豬博士 iPig 系統帳號開通通知',
    text: `您好 ${toName}，

您的豬博士 iPig 系統帳號已開通：

【登入資訊】
帳號：${toEmail}
初始密碼：${password}
登入網址：${loginUrl}

請於首次登入後立即變更密碼。
如有任何問題，請聯繫工作人員（電話：037-433789）。

豬博士動物科技有限公司`,
  });
}
```

---

### 1.4 Email 通知信範本

**主旨：** 豬博士 iPig 系統帳號開通通知

**內容：**
```
您好 {姓名}，

您的豬博士 iPig 系統帳號已開通：

【登入資訊】
帳號：{email}
初始密碼：{password}
登入網址：{login_url}

請於首次登入後立即變更密碼。
如有任何問題，請聯繫工作人員（電話：037-433789）。

豬博士動物科技有限公司
```

---

### 1.5 其他 Email 通知類型

| 通知類型 | 觸發時機 | 收件者 |
|---------|---------|-------|
| 帳號開通 | 管理員建立帳號 | 新使用者 |
| 密碼重設 | 使用者申請或管理員重設 | 使用者 |
| 密碼變更成功 | 使用者變更密碼 | 使用者 |
| 計畫審查通知 | 計畫狀態變更 | PI/相關人員 |
| 獸醫師建議通知 | 獸醫師新增建議 | 試驗工作人員 |

**密碼重設信範本：**
```
您好 {姓名}，

您的豬博士 iPig 系統密碼已重設：

【新密碼】
{new_password}

請於登入後立即變更密碼。

豬博士動物科技有限公司
```

---

### 1.6 機敏資料安全存放指南

#### 方法 1：本地開發 - `.env` 檔案

**步驟：**

1. 在專案根目錄建立 `.env` 檔案（複製 `backend/env.sample` 作為範本）：
```bash
# 複製範例檔案
cp backend/env.sample .env

# 編輯 .env 檔案，填入實際值
```

2. `.env` 檔案範例（**不要**提交真實密碼到 Git）：
```env
# .env（此檔案不可提交到 Git）
# 資料庫設定
POSTGRES_USER=postgres
POSTGRES_PASSWORD=CHANGE_THIS_STRONG_PASSWORD
POSTGRES_DB=ipig_db
POSTGRES_PORT=5433
DATABASE_URL=postgresql://postgres:CHANGE_THIS_STRONG_PASSWORD@localhost:5432/ipig_db

# JWT 安全設定
# 使用 PowerShell 生成安全的 JWT_SECRET：
# $jwt = [Convert]::ToBase64String((1..64 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
JWT_SECRET=CHANGE_THIS_JWT_SECRET_GENERATE_USING_POWERSHELL

# SMTP 郵件設定（可選）
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password-here
SMTP_FROM_EMAIL=your-email@gmail.com
SMTP_FROM_NAME=豬博士動物科技
```

3. 確保 `.gitignore` 中包含：
```gitignore
# 機敏資料
.env
.env.local
.env.production
*.pem
*.key
.env.*
!.env.example
```

4. 使用 `backend/env.sample` 作為範本（不含真實密碼），這個檔案可以提交到 Git。

**Rust 讀取環境變數：**
```rust
// Cargo.toml 加入：dotenv = "0.15"
use dotenv::dotenv;
use std::env;

fn main() {
    // 載入 .env 檔案
    dotenv().ok();
    
    // 讀取環境變數
    let smtp_user = env::var("SMTP_USER")
        .expect("SMTP_USER must be set");
    let smtp_pass = env::var("SMTP_PASS")
        .expect("SMTP_PASS must be set");
}
```

**Node.js 讀取環境變數：**
```javascript
// 安裝：npm install dotenv
require('dotenv').config();

const smtpUser = process.env.SMTP_USER;
const smtpPass = process.env.SMTP_PASS;

if (!smtpUser || !smtpPass) {
  throw new Error('SMTP credentials not configured');
}
```

---

#### 方法 2：Docker 部署 - Docker Secrets

**docker-compose.yml：**
```yaml
version: '3.8'

services:
  api:
    image: ipig-api:latest
    environment:
      - SMTP_HOST=smtp.gmail.com
      - SMTP_PORT=587
      - SMTP_USER_FILE=/run/secrets/smtp_user
      - SMTP_PASS_FILE=/run/secrets/smtp_pass
    secrets:
      - smtp_user
      - smtp_pass

secrets:
  smtp_user:
    file: ./secrets/smtp_user.txt
  smtp_pass:
    file: ./secrets/smtp_pass.txt
```

**secrets/smtp_user.txt：**
```
your-email@gmail.com
```

**secrets/smtp_pass.txt：**
```
your-app-password-here
```

**⚠️ 注意：上述檔案中的密碼都是範例，實際使用時請替換為真實的 SMTP 帳號和應用程式密碼。**

**程式中讀取 Docker Secrets：**
```rust
fn read_secret(name: &str) -> String {
    // 優先從檔案讀取（Docker Secrets）
    let file_path = format!("/run/secrets/{}", name);
    if let Ok(secret) = std::fs::read_to_string(&file_path) {
        return secret.trim().to_string();
    }
    // 否則從環境變數讀取
    std::env::var(name).expect(&format!("{} not configured", name))
}

let smtp_pass = read_secret("smtp_pass");
```

---

#### 方法 3：雲端部署 - 各平台 Secrets Manager

**AWS Secrets Manager：**
```rust
// Cargo.toml: aws-sdk-secretsmanager = "1.0"
use aws_sdk_secretsmanager::Client;

async fn get_smtp_credentials() -> (String, String) {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    
    let response = client
        .get_secret_value()
        .secret_id("ipig/smtp-credentials")
        .send()
        .await
        .expect("Failed to get secret");
    
    let secret_string = response.secret_string().unwrap();
    let credentials: serde_json::Value = serde_json::from_str(secret_string).unwrap();
    
    (
        credentials["SMTP_USER"].as_str().unwrap().to_string(),
        credentials["SMTP_PASS"].as_str().unwrap().to_string(),
    )
}
```

**GCP Secret Manager：**
```javascript
const { SecretManagerServiceClient } = require('@google-cloud/secret-manager');

async function getSmtpCredentials() {
  const client = new SecretManagerServiceClient();
  
  const [userSecret] = await client.accessSecretVersion({
    name: 'projects/your-project/secrets/smtp-user/versions/latest',
  });
  
  const [passSecret] = await client.accessSecretVersion({
    name: 'projects/your-project/secrets/smtp-pass/versions/latest',
  });
  
  return {
    user: userSecret.payload.data.toString(),
    pass: passSecret.payload.data.toString(),
  };
}
```

**Azure Key Vault：**
```javascript
const { SecretClient } = require("@azure/keyvault-secrets");
const { DefaultAzureCredential } = require("@azure/identity");

async function getSmtpCredentials() {
  const credential = new DefaultAzureCredential();
  const client = new SecretClient(
    "https://your-vault.vault.azure.net",
    credential
  );
  
  const smtpUser = await client.getSecret("smtp-user");
  const smtpPass = await client.getSecret("smtp-pass");
  
  return {
    user: smtpUser.value,
    pass: smtpPass.value,
  };
}
```

---

#### 方法 4：Kubernetes - K8s Secrets

**建立 Secret：**
```bash
# 方法 A：從指令建立
kubectl create secret generic smtp-credentials \
  --from-literal=smtp-user=your-email@gmail.com \
  --from-literal=smtp-pass=your-app-password-here

# 方法 B：從 YAML 建立（需 base64 編碼）
# 使用以下命令生成 base64 編碼：
# echo -n 'your-email@gmail.com' | base64
# echo -n 'your-app-password-here' | base64
```

**smtp-secret.yaml：**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: smtp-credentials
type: Opaque
data:
  # ⚠️ 以下為範例，實際使用時請替換為 base64 編碼後的真實值
  # echo -n 'your-email@gmail.com' | base64
  smtp-user: <BASE64_ENCODED_EMAIL>
  # echo -n 'your-app-password-here' | base64
  smtp-pass: <BASE64_ENCODED_PASSWORD>
```

**在 Deployment 中使用：**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ipig-api
spec:
  template:
    spec:
      containers:
      - name: api
        image: ipig-api:latest
        env:
        - name: SMTP_USER
          valueFrom:
            secretKeyRef:
              name: smtp-credentials
              key: smtp-user
        - name: SMTP_PASS
          valueFrom:
            secretKeyRef:
              name: smtp-credentials
              key: smtp-pass
```

---

#### ✅ 採用方案：Docker Secrets

本專案採用 **Docker Secrets** 管理機敏資料。

---

### 1.7 iPig 系統 Docker Secrets 完整配置

#### 目錄結構

```
ipig/
├── docker-compose.yml
├── docker-compose.prod.yml
├── secrets/                    # ⚠️ 加入 .gitignore
│   ├── smtp_user.txt
│   ├── smtp_pass.txt
│   ├── db_password.txt
│   └── jwt_secret.txt
├── .gitignore
└── ...
```

#### .gitignore

```gitignore
# 機敏資料目錄
secrets/

# 環境變數檔案
.env
.env.local
.env.production
```

---

#### Secrets 檔案建立

```bash
# 建立 secrets 目錄
mkdir -p secrets

# 建立各 secret 檔案
# ⚠️ 請替換為實際的值
echo -n "your-email@gmail.com" > secrets/smtp_user.txt
echo -n "your-app-password-here" > secrets/smtp_pass.txt
echo -n "your-strong-database-password" > secrets/db_password.txt
# 生成安全的 JWT Secret（PowerShell）
# $jwt = [Convert]::ToBase64String((1..64 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
# echo -n "generated-jwt-secret" > secrets/jwt_secret.txt
# 或使用 OpenSSL（Linux/Mac）
echo -n "$(openssl rand -base64 32)" > secrets/jwt_secret.txt

# 設定檔案權限（僅擁有者可讀）
chmod 600 secrets/*.txt
```

---

#### docker-compose.yml（完整配置）

```yaml
version: '3.8'

services:
  # PostgreSQL 資料庫
  db:
    image: postgres:15-alpine
    container_name: ipig-db
    restart: unless-stopped
    environment:
      POSTGRES_USER: ipig
      POSTGRES_DB: ipig
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
    secrets:
      - db_password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - ipig-network
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ipig"]
      interval: 10s
      timeout: 5s
      retries: 5

  # 後端 API（Rust）
  api:
    image: ipig-api:latest
    build:
      context: ./backend
      dockerfile: Dockerfile
    container_name: ipig-api
    restart: unless-stopped
    depends_on:
      db:
        condition: service_healthy
    environment:
      # 非機敏設定直接寫
      APP_ENV: production
      APP_HOST: 0.0.0.0
      APP_PORT: 8080
      DATABASE_HOST: db
      DATABASE_PORT: 5432
      DATABASE_NAME: ipig
      DATABASE_USER: ipig
      SMTP_HOST: smtp.gmail.com
      SMTP_PORT: 587
      SMTP_FROM_NAME: 豬博士動物科技
      # 機敏資料從 secrets 讀取
      DATABASE_PASSWORD_FILE: /run/secrets/db_password
      SMTP_USER_FILE: /run/secrets/smtp_user
      SMTP_PASS_FILE: /run/secrets/smtp_pass
      JWT_SECRET_FILE: /run/secrets/jwt_secret
    secrets:
      - db_password
      - smtp_user
      - smtp_pass
      - jwt_secret
    ports:
      - "8080:8080"
    networks:
      - ipig-network

  # 前端（React）
  web:
    image: ipig-web:latest
    build:
      context: ./frontend
      dockerfile: Dockerfile
    container_name: ipig-web
    restart: unless-stopped
    depends_on:
      - api
    ports:
      - "80:80"
      - "443:443"
    networks:
      - ipig-network

# Secrets 定義
secrets:
  db_password:
    file: ./secrets/db_password.txt
  smtp_user:
    file: ./secrets/smtp_user.txt
  smtp_pass:
    file: ./secrets/smtp_pass.txt
  jwt_secret:
    file: ./secrets/jwt_secret.txt

# 網路
networks:
  ipig-network:
    driver: bridge

# 持久化儲存
volumes:
  postgres_data:
```

---

#### 後端程式讀取 Secrets（Rust）

**src/config.rs：**
```rust
use std::fs;
use std::env;

/// 讀取設定值：優先從檔案（Docker Secret），否則從環境變數
pub fn get_config(key: &str) -> String {
    // 檢查是否有對應的 _FILE 環境變數
    let file_key = format!("{}_FILE", key);
    
    if let Ok(file_path) = env::var(&file_key) {
        // 從 Docker Secret 檔案讀取
        match fs::read_to_string(&file_path) {
            Ok(content) => return content.trim().to_string(),
            Err(e) => eprintln!("Warning: Cannot read {}: {}", file_path, e),
        }
    }
    
    // 從環境變數讀取
    env::var(key).unwrap_or_else(|_| {
        panic!("{} or {} must be set", key, file_key)
    })
}

/// 應用程式設定結構
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from_name: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let db_host = get_config("DATABASE_HOST");
        let db_port = get_config("DATABASE_PORT");
        let db_name = get_config("DATABASE_NAME");
        let db_user = get_config("DATABASE_USER");
        let db_pass = get_config("DATABASE_PASSWORD");
        
        Self {
            database_url: format!(
                "postgresql://{}:{}@{}:{}/{}",
                db_user, db_pass, db_host, db_port, db_name
            ),
            jwt_secret: get_config("JWT_SECRET"),
            smtp_host: get_config("SMTP_HOST"),
            smtp_port: get_config("SMTP_PORT").parse().unwrap_or(587),
            smtp_user: get_config("SMTP_USER"),
            smtp_pass: get_config("SMTP_PASS"),
            smtp_from_name: env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "豬博士動物科技".to_string()),
        }
    }
}
```

**src/main.rs：**
```rust
mod config;

use config::AppConfig;

#[tokio::main]
async fn main() {
    // 載入設定
    let config = AppConfig::from_env();
    
    println!("SMTP User: {}", config.smtp_user);
    println!("Database connected to: {}", config.database_url.split('@').last().unwrap());
    
    // 啟動伺服器...
}
```

---

#### 部署指令

```bash
# 開發環境
docker-compose up -d

# 查看日誌
docker-compose logs -f api

# 重新建置並部署
docker-compose up -d --build

# 停止服務
docker-compose down

# 停止並清除資料
docker-compose down -v
```

---

#### 驗證 Secrets 是否正確載入

```bash
# 進入容器檢查
docker exec -it ipig-api sh

# 在容器內查看 secrets（檔案存在於 /run/secrets/）
cat /run/secrets/smtp_user
cat /run/secrets/smtp_pass

# 離開容器
exit
```

---

#### 本地開發（不使用 Docker）

本地開發時，建立 `.env` 檔案：

```env
# .env（本地開發用）
DATABASE_HOST=localhost
DATABASE_PORT=5432
DATABASE_NAME=ipig
DATABASE_USER=ipig
DATABASE_PASSWORD=CHANGE_THIS_PASSWORD

SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password-here
SMTP_FROM_NAME=豬博士動物科技
SMTP_FROM_EMAIL=your-email@gmail.com

# ⚠️ 使用 PowerShell 生成安全的 JWT_SECRET：
# $jwt = [Convert]::ToBase64String((1..64 | ForEach-Object { Get-Random -Minimum 0 -Maximum 256 }))
JWT_SECRET=CHANGE_THIS_JWT_SECRET_GENERATE_USING_POWERSHELL
```

程式碼會自動判斷：有 `_FILE` 環境變數就讀檔案，沒有就讀環境變數。

---

#### 最佳實踐

1. ✅ `secrets/` 目錄加入 `.gitignore`
2. ✅ Secret 檔案權限設為 600
3. ✅ 正式環境不使用 `.env` 檔案
4. ✅ 定期輪換密碼（建議每 90 天）
5. ✅ 備份 secrets 到安全位置（非 Git）

---

## 2. 角色分類

### 2.1 內部 vs 外部人員

| 分類 | 定義 | 可存取系統 |
|-----|------|-----------|
| **內部人員** | 豬博士公司員工 | 全部三個系統 |
| **外部人員** | 委託單位、計畫主持人等 | 僅計畫相關系統 |

---

### 2.2 內部角色（公司員工）

| 角色代碼 | 角色名稱 | 說明 | 可存取系統 |
|---------|---------|------|-----------|
| SYSTEM_ADMIN | 系統管理員 | 全系統最高權限、使用者管理 | AUP ✓ / 進銷存 ✓ / 動物管理 ✓ |
| WAREHOUSE_MANAGER | 倉庫管理員 | 專責 ERP 進銷存 | AUP ✗ / 進銷存 ✓ / 動物管理 ✗ |
| PROGRAM_ADMIN | 程式管理員 | 系統維運 | AUP ✓ / 進銷存 ✓ / 動物管理 ✓ |
| VET | 獸醫師 | 動物健康管理 | AUP ○ / 進銷存 ✗ / 動物管理 ✓ |
| EXPERIMENT_STAFF | 試驗工作人員 | 執行實驗、記錄數據 | AUP ✗ / 進銷存 ○ / 動物管理 ✓ |
| IACUC_STAFF | 執行秘書 | IACUC 行政、管理所有計劃進度 | AUP ✓ / 進銷存 ✗ / 動物管理 ✓ |
| CHAIR | IACUC 主席 | 審查決策 | AUP ✓ / 進銷存 ✗ / 動物管理 ○ |
| REVIEWER | 審查委員 | 計畫審查 | AUP ○ / 進銷存 ✗ / 動物管理 ✗ |

> ✓ 完整存取 ｜ ○ 部分存取 ｜ ✗ 無權限

---

### 2.3 外部角色（委託方/研究人員）

| 角色代碼 | 角色名稱 | 說明 | 可存取系統 |
|---------|---------|------|-----------|
| PI | 計畫主持人 | 提交並管理計畫 | AUP ○ / 進銷存 ✗ / 動物管理 ○ |
| CLIENT | 委託人 | 查看委託案（同單位可多人多計劃） | AUP ○ / 進銷存 ✗ / 動物管理 ○ |

**外部角色限制：**
- 只能看到**自己相關的計畫**
- 只能看到**計畫下的豬隻紀錄**
- 無法看到其他計畫或全部豬隻
- 無法存取進銷存系統

### 2.4 委託單位（CLIENT）詳細權限

| 權限項目 | 說明 |
|---------|------|
| ✅ 查看計畫詳情 | 可檢視自己委託的計畫申請表與豬隻清單 |
| ✅ 下載病歷 | 可下載病歷總表、觀察試驗紀錄、手術紀錄 |
| ✅ 查看獸醫師建議 | 可見紀錄中的獸醫師建議內容 |
| ✅ 結案後存取 | 永久有效，帳號存續期間皆可查閱歷史資料 |
| ✅ 多人帳號 | 同一委託單位可建立多個獨立帳號 |
| ✗ 新增/編輯紀錄 | 僅可檢視，不可修改任何資料 |
| ✗ 進銷存系統 | 完全不可見 |

---

## 3. 系統存取控制

### 3.1 Sidebar 動態顯示

系統依據登入角色動態顯示可用的 Sidebar 項目：

**內部人員（完整 Sidebar）：**
```
├── Dashboard
├── 我的計畫（若有指派）
├── 計畫管理（管理員）
├── 豬隻管理
├── 進銷存管理 ←（僅內部可見）
│   ├── 採購
│   ├── 庫存
│   └── 報表
├── 系統操作說明
├── 資源管理（管理員）
│   ├── 使用者
│   ├── 角色
│   └── ...
└── 登出
```

**外部人員（精簡 Sidebar）：**
```
├── Dashboard
├── 我的計畫 ←（僅顯示自己的計畫）
├── 系統操作說明
└── 登出
```

---

### 3.2 資料範圍控制

| 角色類型 | 計畫範圍 | 豬隻範圍 |
|---------|---------|---------|
| 內部管理員 | 全部計畫 | 全部豬隻 |
| 獸醫師 | 全部計畫（審查） | 全部豬隻（健康管理） |
| 試驗工作人員 | 被指派的計畫 | 被指派計畫的豬隻 |
| PI | 自己的計畫 | 自己計畫的豬隻 |
| CLIENT | 委託的計畫 | 委託計畫的豬隻 |

---

## 4. 詳細權限矩陣

### 4.1 AUP 提交與審查系統

| 功能 | SYSTEM_ADMIN | CHAIR | IACUC_STAFF | REVIEWER | VET | PI | CLIENT |
|-----|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| 查看所有計畫 | ✓ | ✓ | ✓ | ○ | ○ | ✗ | ✗ |
| 查看自己計畫 | ✓ | ✓ | ✓ | ○ | ○ | ✓ | ✓ |
| 建立計畫 | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| 編輯草稿 | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| 提交計畫 | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ | ✗ |
| 指派審查人員 | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| 新增審查意見 | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
| 核准/否決 | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| 變更狀態 | ✓ | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |

> ✓ 可執行 ｜ ○ 部分/條件限制 ｜ ✗ 無權限

---

### 4.2 進銷存管理系統

| 功能 | SYSTEM_ADMIN | WAREHOUSE_MANAGER | PROGRAM_ADMIN | EXPERIMENT_STAFF |
|-----|:---:|:---:|:---:|:---:|
| 查看庫存 | ✓ | ✓ | ✓ | ✓（唯讀） |
| 建立採購單 | ✓ | ✓ | ○ | ✗ |
| 核准採購單 | ✓ | ✓ | ✗ | ✗ |
| 入庫/出庫 | ✓ | ✓ | ○ | ✗ |
| 盤點 | ✓ | ✓ | ○ | ✗ |
| 調撥 | ✓ | ✓ | ○ | ✗ |
| 報表匯出 | ✓ | ✓ | ✓ | ✗ |
| 系統設定 | ✓ | ✗ | ✗ | ✗ |

**說明：** 進銷存系統僅限**內部人員**存取，外部角色完全不可見。

---

### 4.3 實驗動物管理系統

| 功能 | SYSTEM_ADMIN | IACUC_STAFF | VET | EXPERIMENT_STAFF | PI | CLIENT |
|-----|:---:|:---:|:---:|:---:|:---:|:---:|
| 查看所有豬隻 | ✓ | ✓ | ✓ | ○ | ✗ | ✗ |
| 查看計畫豬隻 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| 新增豬隻 | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| 編輯豬隻資料 | ✓ | ✓ | ○ | ✗ | ✗ | ✗ |
| 分配豬隻至計畫 | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| 新增觀察紀錄 | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
| 新增手術紀錄 | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
| 新增體重紀錄 | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
| 新增疫苗紀錄 | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
| 新增犧牲紀錄 | ✓ | ✓ | ✓ | ✓ | ✗ | ✗ |
| 獸醫師建議 | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ |
| 標記已讀 | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ |
| 下載病歷 | ✓ | ✓ | ✓ | ○ | ✓ | ✓ |
| 匯入資料 | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |

---

### 4.4 使用者管理權限

| 功能 | SYSTEM_ADMIN | PROGRAM_ADMIN | 其他 |
|-----|:---:|:---:|:---:|
| 查看使用者列表 | ✓ | ✓ | ✗ |
| 建立使用者帳號 | ✓ | ✗ | ✗ |
| 編輯使用者資料 | ✓ | ✗ | ✗ |
| 停用/啟用帳號 | ✓ | ✗ | ✗ |
| 重設密碼 | ✓ | ✗ | ✗ |
| 指派角色 | ✓ | ✗ | ✗ |
| 管理角色定義 | ✓ | ✗ | ✗ |
| 管理權限定義 | ✓ | ✗ | ✗ |

---

## 5. 資料表設計

### 5.1 使用者表（users）

| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| email | VARCHAR(255) | 登入帳號（唯一） |
| password_hash | VARCHAR(255) | 密碼雜湊 |
| display_name | VARCHAR(100) | 顯示名稱 |
| phone | VARCHAR(20) | 聯絡電話 |
| organization | VARCHAR(200) | 所屬單位（PI/CLIENT 適用） |
| is_internal | BOOLEAN | 是否為內部人員 |
| is_active | BOOLEAN | 帳號是否啟用 |
| must_change_password | BOOLEAN | 是否需要變更密碼 |
| last_login_at | TIMESTAMP | 最後登入時間 |
| created_at | TIMESTAMP | 建立時間 |
| updated_at | TIMESTAMP | 更新時間 |

---

### 5.2 角色表（roles）

| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| code | VARCHAR(50) | 角色代碼（唯一） |
| name | VARCHAR(100) | 角色名稱 |
| description | TEXT | 說明 |
| is_internal | BOOLEAN | 是否為內部角色 |
| is_system | BOOLEAN | 是否為系統預設（不可刪除） |
| created_at | TIMESTAMP | |

---

### 5.3 使用者角色關聯（user_roles）

| 欄位 | 類型 | 說明 |
|-----|------|------|
| user_id | UUID | FK → users.id |
| role_id | UUID | FK → roles.id |
| assigned_at | TIMESTAMP | 指派時間 |
| assigned_by | UUID | 指派者 |

> 主鍵：(user_id, role_id)

---

### 5.4 權限表（permissions）

| 欄位 | 類型 | 說明 |
|-----|------|------|
| id | UUID | 主鍵 |
| code | VARCHAR(100) | 權限代碼（唯一） |
| name | VARCHAR(100) | 權限名稱 |
| module | VARCHAR(50) | 所屬模組（aup/erp/animal） |
| description | TEXT | 說明 |

---

### 5.5 角色權限關聯（role_permissions）

| 欄位 | 類型 | 說明 |
|-----|------|------|
| role_id | UUID | FK → roles.id |
| permission_id | UUID | FK → permissions.id |

> 主鍵：(role_id, permission_id)

---

### 5.6 使用者計畫關聯（user_protocols）

用於控制外部人員可存取的計畫範圍：

| 欄位 | 類型 | 說明 |
|-----|------|------|
| user_id | UUID | FK → users.id |
| protocol_id | UUID | FK → protocols.id |
| role_in_protocol | ENUM | 在該計畫的角色（PI/CLIENT） |
| granted_at | TIMESTAMP | 授權時間 |
| granted_by | UUID | 授權者 |

---

## 6. 預設權限代碼

### 6.1 AUP 系統權限

| 權限代碼 | 說明 |
|---------|------|
| `aup.protocol.view_all` | 查看所有計畫 |
| `aup.protocol.view_own` | 查看自己的計畫 |
| `aup.protocol.create` | 建立計畫 |
| `aup.protocol.edit` | 編輯計畫 |
| `aup.protocol.submit` | 提交計畫 |
| `aup.protocol.review` | 審查計畫 |
| `aup.protocol.approve` | 核准/否決 |
| `aup.protocol.change_status` | 變更狀態 |
| `aup.review.assign` | 指派審查人員 |
| `aup.review.comment` | 新增審查意見 |

---

### 6.2 進銷存系統權限

| 權限代碼 | 說明 |
|---------|------|
| `erp.inventory.view` | 查看庫存 |
| `erp.purchase.create` | 建立採購單 |
| `erp.purchase.approve` | 核准採購單 |
| `erp.stock.in` | 入庫操作 |
| `erp.stock.out` | 出庫操作 |
| `erp.stock.adjust` | 庫存調整 |
| `erp.stock.transfer` | 調撥 |
| `erp.stocktake.create` | 盤點 |
| `erp.report.view` | 查看報表 |
| `erp.report.export` | 匯出報表 |

---

### 6.3 實驗動物管理系統權限

| 權限代碼 | 說明 |
|---------|------|
| `animal.pig.view_all` | 查看所有豬隻 |
| `animal.pig.view_project` | 查看計畫內豬隻 |
| `animal.pig.create` | 新增豬隻 |
| `animal.pig.edit` | 編輯豬隻資料 |
| `animal.pig.assign` | 分配豬隻至計畫 |
| `animal.pig.import` | 匯入豬隻資料 |
| `animal.record.create` | 新增紀錄 |
| `animal.record.edit` | 編輯紀錄 |
| `animal.record.delete` | 刪除紀錄 |
| `animal.vet.recommend` | 新增獸醫師建議 |
| `animal.vet.read` | 標記獸醫師已讀 |
| `animal.export.medical` | 匯出病歷 |

---

### 6.4 系統管理權限

| 權限代碼 | 說明 |
|---------|------|
| `admin.user.view` | 查看使用者 |
| `admin.user.create` | 建立使用者 |
| `admin.user.edit` | 編輯使用者 |
| `admin.user.delete` | 停用使用者 |
| `admin.user.reset_password` | 重設密碼 |
| `admin.role.manage` | 管理角色 |
| `admin.permission.manage` | 管理權限 |
| `admin.audit.view` | 查看稽核紀錄 |

---

## 7. API 端點

### 7.1 使用者管理

| 端點 | 方法 | 說明 | 權限 |
|-----|------|------|------|
| `/admin/users` | GET | 取得使用者列表 | `admin.user.view` |
| `/admin/users` | POST | 建立使用者（觸發 Email 通知） | `admin.user.create` |
| `/admin/users/{id}` | GET | 取得單一使用者 | `admin.user.view` |
| `/admin/users/{id}` | PATCH | 更新使用者 | `admin.user.edit` |
| `/admin/users/{id}/deactivate` | POST | 停用帳號 | `admin.user.delete` |
| `/admin/users/{id}/activate` | POST | 啟用帳號 | `admin.user.edit` |
| `/admin/users/{id}/reset-password` | POST | 重設密碼（觸發 Email） | `admin.user.reset_password` |
| `/admin/users/{id}/roles` | PUT | 指派角色 | `admin.role.manage` |

---

### 7.2 角色管理

| 端點 | 方法 | 說明 | 權限 |
|-----|------|------|------|
| `/admin/roles` | GET | 取得角色列表 | `admin.role.manage` |
| `/admin/roles` | POST | 建立角色 | `admin.role.manage` |
| `/admin/roles/{id}` | PATCH | 更新角色 | `admin.role.manage` |
| `/admin/roles/{id}/permissions` | PUT | 設定角色權限 | `admin.permission.manage` |

---

### 7.3 權限查詢

| 端點 | 方法 | 說明 |
|-----|------|------|
| `/auth/me` | GET | 取得當前使用者資訊與權限 |
| `/auth/me/permissions` | GET | 取得當前使用者所有權限代碼 |
| `/auth/me/accessible-systems` | GET | 取得可存取的子系統列表 |

---

## 8. 前端權限控制

### 8.1 路由守衛

```javascript
// 檢查是否可存取子系統
const canAccessSystem = (system) => {
  const accessibleSystems = user.accessibleSystems; // ['aup', 'animal']
  return accessibleSystems.includes(system);
};

// 檢查是否有特定權限
const hasPermission = (permissionCode) => {
  return user.permissions.includes(permissionCode);
};
```

### 8.2 UI 元件顯示控制

```jsx
// 依權限顯示按鈕
{hasPermission('animal.pig.create') && (
  <Button>＋新增豬隻</Button>
)}

// 依角色顯示 Sidebar 項目
{user.isInternal && (
  <SidebarItem>進銷存管理</SidebarItem>
)}
```

---

## 9. 補充說明

### 9.1 角色可疊加

- 一個使用者可以同時擁有多個角色
- 權限取聯集（擁有任一角色的權限即可執行）
- 例：某人同時是 VET 和 PI，則可審查計畫也可提交計畫

### 9.2 外部人員計畫關聯

- 外部人員必須關聯至少一個計畫才能登入查看資料
- 計畫結案後，外部人員仍可查看歷史紀錄（唯讀）
- 新計畫需重新關聯授權

### 9.3 密碼政策

- 最小長度：8 字元
- 必須包含：大小寫字母 + 數字
- 首次登入強制變更
- 建議 90 天定期變更（可設定提醒）

