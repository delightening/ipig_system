/**
 * iPig 系統 - k6 壓力測試腳本
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
 *   BASE_URL    - API 基底路徑（預設 http://localhost:8000）
 *   TEST_EMAIL  - 測試帳號 email
 *   TEST_PASS   - 測試帳號密碼
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
        'api_duration': ['p(95)<500'],
        'report_duration': ['p(95)<2000'],
        'login_duration': ['p(95)<1000'],
        'errors': ['rate<0.05'],
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

function authGet(url, token, metricTrend) {
    const params = {
        headers: { 'Authorization': `Bearer ${token}` },
    };
    const res = http.get(url, params);
    if (metricTrend) metricTrend.add(res.timings.duration);
    check(res, {
        'HTTP 200': (r) => r.status === 200,
    }) || errorRate.add(1);
    return res;
}

// ============================================
// setup(): 登入一次，所有 VU 共用 token
// ============================================
export function setup() {
    const res = http.post(
        `${BASE_URL}/api/auth/login`,
        JSON.stringify({ email: TEST_EMAIL, password: TEST_PASS }),
        { headers: { 'Content-Type': 'application/json' } }
    );

    loginDuration.add(res.timings.duration);

    const ok = check(res, {
        '登入成功 (200)': (r) => r.status === 200,
    });

    if (!ok) {
        console.error(`Setup 登入失敗 [${res.status}]: ${res.body}`);
        return { token: null };
    }

    const body = JSON.parse(res.body);
    console.log(`✅ Setup 登入成功, token 長度: ${body.access_token.length}`);
    return { token: body.access_token };
}

// ============================================
// 主測試流程（每個 VU 重複執行）
// ============================================
export default function (data) {
    if (!data.token) {
        console.error('無 token，跳過本次迭代');
        errorRate.add(1);
        sleep(1);
        return;
    }

    const token = data.token;

    // 1. 健康檢查（公開端點）
    group('健康檢查', () => {
        const res = http.get(`${BASE_URL}/api/health`);
        apiDuration.add(res.timings.duration);
        check(res, { '健康回應': (r) => r.status === 200 });
    });

    sleep(0.3);

    // 2. 一般 API 端點
    group('一般 API', () => {
        authGet(`${BASE_URL}/api/animals`, token, apiDuration);
        sleep(0.2);
        authGet(`${BASE_URL}/api/protocols`, token, apiDuration);
        sleep(0.2);
        authGet(`${BASE_URL}/api/users`, token, apiDuration);
        sleep(0.2);
        authGet(`${BASE_URL}/api/notifications`, token, apiDuration);
    });

    sleep(0.5);

    // 3. 報表 API（允許較長回應時間）
    group('報表 API', () => {
        authGet(`${BASE_URL}/api/reports/stock-on-hand`, token, reportDuration);
        sleep(0.3);
        authGet(`${BASE_URL}/api/reports/stock-ledger`, token, reportDuration);
        sleep(0.3);
        authGet(`${BASE_URL}/api/reports/purchase-lines`, token, reportDuration);
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
        stdout: JSON.stringify(data.metrics, null, 2),
    };
}
