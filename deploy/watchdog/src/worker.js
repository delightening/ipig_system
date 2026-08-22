/**
 * ipig-watchdog — 跑在 Cloudflare 上的外部看門狗
 *
 * 存在理由：Prometheus / Alertmanager / Grafana 全部跑在那台 prod 筆電的 Docker 裡。
 * 筆電掛掉的那一刻監控跟著掛，`backup_last_success_timestamp_seconds` 這條 alert
 * 永遠不會送出——能偵測「系統掛了」的唯一機制，跟系統掛在同一台機器上。
 * 本 Worker 是唯一跑在筆電外面的那一層，因此它的通知路徑**不得**經過筆電
 * （不可複用 Alertmanager SMTP 或 R22 SecurityNotifier，否則就退化成自己監控自己）。
 *
 * 兩個職責：
 *   (a) 主動探測 — cron 每 5 分鐘打 /api/health，連續 FAIL_THRESHOLD 次失敗即告警。
 *   (b) 被動心跳（dead man's switch）— 收 POST /ping/<job>，逾期未 ping 即告警。
 *       這半邊才抓得到「靜默失敗」：2026-05-09 那次 pg_backup.sh DB_NAME 打錯、
 *       cron 數週失敗而 /backups/ 空無一物，主動探測完全看不出來（網站好好的）。
 *
 * 刻意不做：degraded（HTTP 200 但 status != "healthy"）不告警。
 * 那代表筆電還活著，內部 Alertmanager 本來就看得到、也管得比這裡細，
 * 在這裡重複一份只會製造兩邊都要維護的告警規則。本 Worker 只管「還在不在」。
 */

import { EmailMessage } from "cloudflare:email";

/** 心跳 job → 逾期門檻（毫秒）。key 同時是 /ping/<job> 的合法值白名單。 */
const HEARTBEAT_JOBS = {
  // 備份 cron 排 02:00 daily，給 2 小時寬限
  backup: 26 * 60 * 60 * 1000,
};

const KV_PROBE = "state:probe";
const KV_BOOTSTRAP = "state:bootstrap";
const pingKey = (job) => `ping:${job}`;
const hbStateKey = (job) => `state:hb:${job}`;

/** 探測成功時，lastOkAt 最多這麼久才回寫一次 KV（免費層每日 1000 writes）。 */
const OK_WRITE_INTERVAL_MS = 60 * 60 * 1000;

export default {
  async scheduled(_event, env, _ctx) {
    const now = Date.now();
    const bootstrapAt = await ensureBootstrap(env, now);

    const health = await checkHealth(env, now);
    const heartbeats = [];
    for (const [job, maxAgeMs] of Object.entries(HEARTBEAT_JOBS)) {
      heartbeats.push(await checkHeartbeat(env, job, maxAgeMs, now, bootstrapAt));
    }

    const alerts = [...health.alerts, ...heartbeats.flatMap((h) => h.alerts)];
    let sendError = null;
    if (alerts.length > 0) {
      try {
        await sendAlert(env, alerts, now);
      } catch (e) {
        sendError = e;
      }
    }

    // 送信失敗時不推進 alerted 狀態，讓下一輪重新嘗試通知；
    // fails/lastOkAt 等純追蹤資料仍照常回寫，不受送信結果影響。
    const delivered = sendError === null;
    await health.commit(delivered);
    for (const h of heartbeats) await h.commit(delivered);

    if (sendError) throw sendError;
  },

  async fetch(request, env) {
    const url = new URL(request.url);

    const ping = url.pathname.match(/^\/ping\/([a-z0-9_-]{1,32})$/);
    if (ping) {
      if (request.method !== "POST") {
        return new Response("method not allowed", { status: 405 });
      }
      if (!authorized(request, env)) {
        return new Response("forbidden", { status: 403 });
      }
      const job = ping[1];
      if (!Object.hasOwn(HEARTBEAT_JOBS, job)) {
        return new Response("unknown job", { status: 404 });
      }
      await env.WATCHDOG_KV.put(pingKey(job), JSON.stringify({ at: Date.now() }));
      return new Response(null, { status: 204 });
    }

    if (url.pathname === "/status") {
      if (request.method !== "GET") {
        return new Response("method not allowed", { status: 405 });
      }
      if (!authorized(request, env)) {
        return new Response("forbidden", { status: 403 });
      }
      return Response.json(await buildStatus(env, Date.now()));
    }

    return new Response("not found", { status: 404 });
  },
};

// ============================================================================
// 探測（主動）
// ============================================================================

/** @returns {Promise<{alerts: string[], commit: (delivered: boolean) => Promise<void>}>} */
async function checkHealth(env, now) {
  const probe = await probeHealth(env);
  const prev = (await kvGetJSON(env, KV_PROBE)) ?? { fails: 0, alerted: false, lastOkAt: null };
  const threshold = positiveInt(env.FAIL_THRESHOLD, 3);
  const alerts = [];

  let pending;
  if (probe.ok) {
    if (prev.alerted) {
      const downSince = prev.lastOkAt ? fmt(prev.lastOkAt) : "(不明)";
      alerts.push(`✅ 系統已恢復：${env.HEALTH_URL} 回應正常（最後一次正常：${downSince}）`);
    }
    pending = { fails: 0, alerted: false, lastOkAt: now };
  } else {
    const fails = (prev.fails ?? 0) + 1;
    const shouldAlert = fails >= threshold && !prev.alerted;
    if (shouldAlert) {
      alerts.push(
        `🔴 系統無回應：${env.HEALTH_URL} 連續 ${fails} 次探測失敗（${probe.detail}）。` +
          `最後一次正常：${prev.lastOkAt ? fmt(prev.lastOkAt) : "(不明)"}。` +
          `處置見 docs/runbooks/cold-start.md`,
      );
    }
    pending = {
      fails,
      alerted: prev.alerted || shouldAlert,
      lastOkAt: prev.lastOkAt ?? null,
      lastDetail: probe.detail,
    };
  }

  return {
    alerts,
    async commit(delivered) {
      // 送信失敗時 alerted 維持前一輪的值（下一輪重新嘗試通知）；
      // 其餘純追蹤欄位（fails/lastOkAt）不受送信結果影響，照常回寫。
      const next = delivered ? pending : { ...pending, alerted: prev.alerted };
      // 只在狀態真的變動、或 lastOkAt 過期時回寫，避免每 5 分鐘吃掉一次 KV write 額度
      const stateChanged = next.fails !== prev.fails || next.alerted !== prev.alerted;
      const okStale = probe.ok && (!prev.lastOkAt || now - prev.lastOkAt >= OK_WRITE_INTERVAL_MS);
      if (stateChanged || okStale) {
        await env.WATCHDOG_KV.put(KV_PROBE, JSON.stringify(next));
      }
    },
  };
}

async function probeHealth(env) {
  const timeoutMs = positiveInt(env.PROBE_TIMEOUT_MS, 10_000);
  try {
    const res = await fetch(env.HEALTH_URL, {
      method: "GET",
      signal: AbortSignal.timeout(timeoutMs),
      headers: { "user-agent": "ipig-watchdog", "cache-control": "no-cache" },
    });
    if (!res.ok) {
      return { ok: false, detail: `HTTP ${res.status}` };
    }
    // body 解析失敗不影響「活著」的判定——回得了 200 就代表 API 在
    let status = "200";
    try {
      status = (await res.json())?.status ?? "200";
    } catch {
      /* 忽略 */
    }
    return { ok: true, detail: status };
  } catch (e) {
    return { ok: false, detail: `unreachable: ${e?.message ?? String(e)}` };
  }
}

// ============================================================================
// 心跳（被動 / dead man's switch）
// ============================================================================

/**
 * 沒有任何 ping 紀錄時，用 Worker 自己第一次跑的時間當基準——
 * 否則部署當天就會誤報「backup 從未執行」。
 * 但基準一旦超過門檻仍會告警，所以「腳本改了卻沒生效」這種失敗也抓得到。
 */
async function ensureBootstrap(env, now) {
  const rec = await kvGetJSON(env, KV_BOOTSTRAP);
  if (rec?.at) return rec.at;
  await env.WATCHDOG_KV.put(KV_BOOTSTRAP, JSON.stringify({ at: now }));
  return now;
}

/** @returns {Promise<{alerts: string[], commit: (delivered: boolean) => Promise<void>}>} */
async function checkHeartbeat(env, job, maxAgeMs, now, bootstrapAt) {
  const ping = await kvGetJSON(env, pingKey(job));
  const state = (await kvGetJSON(env, hbStateKey(job))) ?? { alerted: false };
  const lastAt = ping?.at ?? bootstrapAt;
  const overdue = now - lastAt > maxAgeMs;
  const alerts = [];

  if (overdue && !state.alerted) {
    const seen = ping?.at ? fmt(ping.at) : "從未收到（以看門狗啟用時間計算）";
    alerts.push(
      `🟠 心跳逾期：job「${job}」已 ${hours(now - lastAt)} 小時未回報成功（門檻 ${hours(maxAgeMs)} 小時）。` +
        `最後一次成功：${seen}。備份可能正在靜默失敗——請查 db-backup 容器 log。`,
    );
  } else if (!overdue && state.alerted) {
    alerts.push(`✅ 心跳恢復：job「${job}」已回報成功（${fmt(lastAt)}）`);
  }

  return {
    alerts,
    async commit(delivered) {
      // 送信失敗時保留原本的 alerted，下一輪重新嘗試通知
      const nextAlerted = delivered ? overdue : state.alerted;
      if (nextAlerted !== state.alerted) {
        await env.WATCHDOG_KV.put(hbStateKey(job), JSON.stringify({ alerted: nextAlerted }));
      }
    },
  };
}

// ============================================================================
// 通知
// ============================================================================

async function sendAlert(env, lines, now) {
  const down = lines.some((l) => l.startsWith("🔴") || l.startsWith("🟠"));
  const subject = down ? "[iPig 看門狗] 系統異常" : "[iPig 看門狗] 已恢復";
  const text = [
    ...lines,
    "",
    `偵測時間：${fmt(now)}`,
    "來源：Cloudflare Workers（跑在 prod 筆電之外，不受筆電狀態影響）",
    "設定：deploy/watchdog/",
  ].join("\n");

  const from = env.ALERT_FROM;
  const to = env.ALERT_TO;
  try {
    await env.ALERT_EMAIL.send(new EmailMessage(from, to, buildMime({ from, to, subject, text })));
  } catch (e) {
    // 通知送不出去是最糟的失敗模式——一定要留在 wrangler tail 看得到的地方
    console.error("watchdog: 告警寄送失敗", e?.stack ?? String(e), "| 內容:", text);
    throw e;
  }
}

function buildMime({ from, to, subject, text }) {
  // 中文主旨要走 RFC 2047，否則部分 MTA 會收到亂碼
  const body = b64(text).replace(/(.{76})/g, "$1\r\n");
  return [
    `From: ${from}`,
    `To: ${to}`,
    `Subject: =?UTF-8?B?${b64(subject)}?=`,
    `Message-ID: <${crypto.randomUUID()}@ipig-watchdog>`,
    `Date: ${new Date().toUTCString().replace(/GMT$/, "+0000")}`,
    "MIME-Version: 1.0",
    'Content-Type: text/plain; charset="utf-8"',
    "Content-Transfer-Encoding: base64",
    "",
    body,
  ].join("\r\n");
}

// ============================================================================
// 小工具
// ============================================================================

async function buildStatus(env, now) {
  const probe = (await kvGetJSON(env, KV_PROBE)) ?? null;
  const bootstrap = await kvGetJSON(env, KV_BOOTSTRAP);
  const heartbeats = {};
  for (const [job, maxAgeMs] of Object.entries(HEARTBEAT_JOBS)) {
    const ping = await kvGetJSON(env, pingKey(job));
    const state = await kvGetJSON(env, hbStateKey(job));
    heartbeats[job] = {
      last_ping: ping?.at ? fmt(ping.at) : null,
      overdue_after_hours: hours(maxAgeMs),
      alerted: state?.alerted ?? false,
    };
  }
  return {
    now: fmt(now),
    health_url: env.HEALTH_URL,
    probe: probe && { ...probe, lastOkAt: probe.lastOkAt ? fmt(probe.lastOkAt) : null },
    watching_since: bootstrap?.at ? fmt(bootstrap.at) : null,
    heartbeats,
  };
}

function authorized(request, env) {
  if (!env.PING_TOKEN) return false;
  const header = request.headers.get("authorization") ?? "";
  const match = header.match(/^Bearer\s+(.+)$/i);
  return Boolean(match) && timingSafeEqual(match[1].trim(), env.PING_TOKEN);
}

/** 長度會外洩，但這個 token 只保護一支心跳端點，可接受。 */
function timingSafeEqual(a, b) {
  if (typeof a !== "string" || typeof b !== "string" || a.length !== b.length) return false;
  let diff = 0;
  for (let i = 0; i < a.length; i++) diff |= a.charCodeAt(i) ^ b.charCodeAt(i);
  return diff === 0;
}

async function kvGetJSON(env, key) {
  try {
    return await env.WATCHDOG_KV.get(key, { type: "json" });
  } catch (e) {
    console.error(`watchdog: KV 讀取失敗 key=${key}`, e?.stack ?? String(e));
    return null;
  }
}

function positiveInt(raw, fallback) {
  const n = Number(raw);
  return Number.isFinite(n) && n > 0 ? n : fallback;
}

/** 使用者在 GMT+8；不依賴 Intl 時區資料，直接算偏移。 */
function fmt(ts) {
  return `${new Date(ts + 8 * 60 * 60 * 1000).toISOString().slice(0, 19).replace("T", " ")} (GMT+8)`;
}

function hours(ms) {
  return Math.round(ms / 3_600_000);
}

function b64(s) {
  const bytes = new TextEncoder().encode(s);
  let bin = "";
  for (const byte of bytes) bin += String.fromCharCode(byte);
  return btoa(bin);
}
