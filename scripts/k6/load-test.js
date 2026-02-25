/**
 * iPig 系統 - k6 壓力測試腳本 (P1-5)
 *
 * 安裝 k6：
 *   Windows: choco install k6 -y (管理員) 或 https://dl.k6.io/msi/k6-latest-amd64.msi
 *   macOS:   brew install k6
 *   Linux:   snap install k6
 *
 * 執行方式：
 *   k6 run scripts/k6/load-test.js
 *   k6 run --vus 50 --duration 60s scripts/k6/load-test.js
 *
 * 環境變數：
 *   BASE_URL    - API 基底路徑（預設 http://localhost:8080）
 *   TEST_USER   - 測試帳號（預設 admin）
 *   TEST_PASS   - 測試密碼（預設 changeme）
 */

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// ============================================
// 自訂指標
// ============================================
const errorRate = new Rate('errors');
const loginDuration = new Trend('login_duration', true);
const apiDuration = new Trend('api_duration', true);
const reportDuration = new Trend('report_duration', true);

// ============================================
// 測試設定
// ============================================
export const options = {
    stages: [
        { duration: '10s', target: 10 },  // 暖機：逐步增加至 10 VU
        { duration: '30s', target: 30 },  // 負載：增加至 30 VU
        { duration: '20s', target: 50 },  // 高峰：增加至 50 VU
        { duration: '10s', target: 0 },   // 降溫：逐步歸零
    ],
    thresholds: {
        // P95 < 500ms（一般 API）
        'api_duration': ['p(95)<500'],
        // P95 < 2000ms（報表 API）
        'report_duration': ['p(95)<2000'],
        // 登入 P95 < 1000ms
        'login_duration': ['p(95)<1000'],
        // 錯誤率 < 5%
        'errors': ['rate<0.05'],
        // HTTP 請求失敗率 < 10%
        'http_req_failed': ['rate<0.1'],
    },
};

// ============================================
// 環境變數
// ============================================
const BASE_URL = __ENV.BASE_URL || 'http://localhost:8000';
const TEST_EMAIL = __ENV.TEST_EMAIL || 'jason4617987@gmail.com';
const TEST_PASS = __ENV.TEST_PASS || 'kfknxJH6AjSvJh6?';

// ============================================
// 輔助函式
// ============================================

/**
 * 登入並取得 JWT token
 */
function login() {
    const res = http.post(
        `${BASE_URL}/api/auth/login`,
        JSON.stringify({ email: TEST_EMAIL, password: TEST_PASS }),
        { headers: { 'Content-Type': 'application/json' } }
    );

    if (res.status !== 200) {
        console.log(`登入失敗 [${res.status}]: ${res.body}`);
        errorRate.add(1);
        return null;
    }

    const body = JSON.parse(res.body);
    loginDuration.add(res.timings.duration);
    check(res, {
        '登入成功 (200)': (r) => r.status === 200,
    });

    return body.access_token;
}

/**
 * 發送已認證的 GET 請求
 */
function authGet(url, token, metricTrend) {
    const params = {
        headers: {
            'Authorization': `Bearer ${token}`,
        },
    };
    const res = http.get(url, params);
    if (metricTrend) metricTrend.add(res.timings.duration);
    check(res, {
        'HTTP 200': (r) => r.status === 200,
    }) || errorRate.add(1);
    return res;
}

// 每個 VU 初始化時會執行一次
export function setup() {
    return {};
}

// 儲存每個 VU 的 token，避免重複登入觸發 Rate Limit
let vuToken = null;

// ============================================
// 主測試流程
// ============================================
export default function () {
    // 1. 登入 (如果該 VU 還沒登入)
    if (!vuToken) {
        group('登入', () => {
            vuToken = login();
        });
        // 如果登入失敗，跳過本次循環並稍候再試
        if (!vuToken) {
            sleep(1);
            return;
        }
    }

    sleep(0.5);

    // 2. 健康檢查（公開）
    group('健康檢查', () => {
        const res = http.get(`${BASE_URL}/api/health`);
        apiDuration.add(res.timings.duration);
        check(res, { '健康回應': (r) => r.status === 200 });
    });

    sleep(0.3);

    // 3. 一般 API 端點
    group('一般 API', () => {
        authGet(`${BASE_URL}/api/animals`, vuToken, apiDuration);
        sleep(0.2);
        authGet(`${BASE_URL}/api/protocols`, vuToken, apiDuration);
        sleep(0.2);
        authGet(`${BASE_URL}/api/users`, vuToken, apiDuration);
        sleep(0.2);
        authGet(`${BASE_URL}/api/notifications`, vuToken, apiDuration);
    });

    sleep(0.5);

    // 4. 報表 API（允許較長回應時間）
    group('報表 API', () => {
        authGet(`${BASE_URL}/api/reports/stock-on-hand`, vuToken, reportDuration);
        sleep(0.3);
        authGet(`${BASE_URL}/api/reports/stock-ledger`, vuToken, reportDuration);
        sleep(0.3);
        authGet(`${BASE_URL}/api/reports/purchase-lines`, vuToken, reportDuration);
    });

    sleep(1);
}

// ============================================
// 測試摘要
// ============================================
export function handleSummary(data) {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
    return {
        [`tests/results/k6_${timestamp}.json`]: JSON.stringify(data, null, 2),
        stdout: textSummary(data, { indent: '  ', enableColors: true }),
    };
}

// k6 內建文字摘要
function textSummary(data, opts) {
    // k6 v0.30+ 有內建 handleSummary，此處為相容性回退
    return JSON.stringify(data.metrics, null, 2);
}
