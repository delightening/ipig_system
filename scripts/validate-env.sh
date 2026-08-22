#!/bin/bash
set -euo pipefail

# iPig 環境變數驗證腳本
# 在 Docker 啟動前或 CI/CD pipeline 中執行

ERRORS=0
WARNINGS=0

check_required() {
  local var_name="$1"
  local description="${2:-}"
  if [ -z "${!var_name:-}" ]; then
    echo "ERROR: ${var_name} is not set${description:+ ($description)}"
    ERRORS=$((ERRORS + 1))
  fi
}

check_optional() {
  local var_name="$1"
  local description="${2:-}"
  if [ -z "${!var_name:-}" ]; then
    echo "WARN:  ${var_name} is not set${description:+ ($description)}"
    WARNINGS=$((WARNINGS + 1))
  fi
}

echo "=== iPig Environment Validation ==="
echo ""

# Database (required)
check_required "DATABASE_URL" "PostgreSQL connection string"

# JWT ES256 金鑰（required）
# 可直接設定 PEM 內容，或使用 _FILE 指向檔案路徑（Docker Secrets 推薦）
check_required "JWT_EC_PRIVATE_KEY" "ES256 EC P-256 private key PEM (or JWT_EC_PRIVATE_KEY_FILE)"
check_required "JWT_EC_PUBLIC_KEY"  "ES256 EC P-256 public key PEM (or JWT_EC_PUBLIC_KEY_FILE)"

# Application
check_optional "RUST_LOG" "Logging level (default: info)"
check_optional "SERVER_PORT" "API server port (default: 8000)"

# SMTP (optional but recommended)
check_optional "SMTP_HOST" "SMTP server for sending emails"
check_optional "SMTP_USERNAME" "SMTP authentication"
check_optional "SMTP_PASSWORD" "SMTP authentication"
check_optional "SMTP_FROM_EMAIL" "Sender email address"

# Google Calendar (optional)
check_optional "GOOGLE_SERVICE_ACCOUNT_KEY" "Google Calendar integration"
check_optional "GOOGLE_CALENDAR_ID" "Google Calendar ID"

# Security (recommended)
check_optional "AUDIT_HMAC_KEY" "HMAC key for audit log integrity (min 16 chars)"
if [ -n "${AUDIT_HMAC_KEY:-}" ] && [ ${#AUDIT_HMAC_KEY} -lt 16 ]; then
  echo "WARN:  AUDIT_HMAC_KEY is shorter than recommended 16 characters"
  WARNINGS=$((WARNINGS + 1))
fi

check_optional "TOTP_ENCRYPTION_KEY" "TOTP secret encryption key"

echo ""
echo "=== Results ==="
echo "Errors:   ${ERRORS}"
echo "Warnings: ${WARNINGS}"

if [ $ERRORS -gt 0 ]; then
  echo ""
  echo "FAILED: ${ERRORS} required variable(s) missing."
  exit 1
fi

echo ""
echo "PASSED: All required variables are set."
exit 0
