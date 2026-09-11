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

/**
 * /ping/<job> 同一 job 最多這麼久才真的回寫一次 KV。心跳本來就是低頻事件
 * （backup 一天一次），這個節流是為了擋用戶端重試迴圈失控時打滿免費層每日
 * 1000 writes 額度——不影響逾期判斷的正確性，因為視窗遠小於 HEARTBEAT_JOBS
 * 的門檻（最短 26 小時）。
 *
 * 🔴 **這是 best-effort 的成本阻尼，不是安全控制。**
 * （2026-09-11 訂正；CodeRabbit 於 MR !8 以 CWE-400 指出，判斷正確。）
 *
 * 下面 fetch() 裡的判斷是 read-then-write，中間沒有任何互斥，而 Workers KV
 * **沒有 compare-and-set**。更關鍵的是 KV 為最終一致：同一個 key 寫進去要數十秒
 * 才傳播到各地副本。所以繞過它的不只是「同一瞬間的平行請求」，而是
 * **傳播完成前抵達的每一個請求都還讀到舊值**。
 *
 * 因此它擋得住的與擋不住的要分開講：
 *   - 用戶端重試迴圈失控 → **擋得住**。重試是序列的（前一個回來才發下一個），
 *     第二次讀到的就是第一次寫進去的值。這是本常數存在的理由。
 *   - PING_TOKEN 外洩後被平行濫用 → **擋不住**，攻擊者可以同時發。
 *
 * ⚠️ 本段原本把「擋 token 外洩」也寫成節流的存在理由——那是它結構上做不到的事。
 * 對付濫用的防線是**輪換 token** ＋ **Cloudflare 邊緣的 Rate Limiting 規則**
 * （在 Dashboard 對 /ping/* 設每 IP 上限）：那一層在請求進到本 Worker 之前就
 * 強制執行，才是真正原子的，而且不必在這支 Worker 裡多養一個有狀態元件。
 *
 * 刻意不用 Durable Object：它確實做得到原子，但本 Worker 是整套監控唯一跑在
 * 那台筆電外面的一層，**它自己壞掉沒有任何東西會通知你**（見 scheduled() 裡
 * 關於 KV 故障時看門狗會安靜地瞎掉的說明）。為了守一個免費額度而給這道最外層
 * 防線增加一個故障點，不划算。
 */
const PING_WRITE_MIN_INTERVAL_MS = 10 * 60 * 1000;

export default {
  async scheduled(_event, env, _ctx) {
    const now = Date.now();

    // ── 讀取階段 ────────────────────────────────────────────────────────────
    // 這一段只讀不寫，所有 KV 寫入都在後面的 commit()。因此 KV 讀不到時直接讓
    // KvUnavailable 往上拋，本輪就在「還沒動到任何狀態」的位置乾淨中止：
    // 不寄信、不回寫、下一輪（5 分鐘後）重來。
    //
    // ⚠️ 代價是 KV 持續故障時看門狗會**安靜地瞎掉**，而它自己不會告警。
    // 這是刻意選的：另一邊（沿用舊的 fail-open）是拿假資料做判斷，會同時製造
    // 誤報與漏報（見 KvUnavailable 的說明）。用一個「延後偵測 5 分鐘」換掉
    // 「講錯話」，對外部監控是划算的。KV 故障的可見性靠 Cloudflare 的 Workers
    // 錯誤率與 `wrangler tail`——這個 throw 就是為了讓它出現在那裡。
    let bootstrap;
    let health;
    const heartbeats = [];
    try {
      bootstrap = await ensureBootstrap(env, now);
      health = await checkHealth(env, now);
      for (const [job, maxAgeMs] of Object.entries(HEARTBEAT_JOBS)) {
        heartbeats.push(await checkHeartbeat(env, job, maxAgeMs, now, bootstrap.at));
      }
    } catch (e) {
      if (e instanceof KvUnavailable) {
        console.error(`watchdog: KV 不可用，跳過本輪（不寄信、不回寫）— ${e.message}`);
      }
      throw e;
    }

    // ── 判斷與寫入階段 ──────────────────────────────────────────────────────
    // 讀取階段全部成功，這裡才是第一個允許寫入的地方——bootstrap 的落地放最前面，
    // 因為它是純追蹤資料，不受後面送信結果影響（跟 fails/lastOkAt 同一類）。
    await bootstrap.commit();
    const alerts = [...health.alerts, ...heartbeats.flatMap((h) => h.alerts)];
    // 哪些機制在這一輪有話要說。只用來組主旨，不影響判斷或內文。
    const scopes = [
      ...(health.alerts.length > 0 ? ["系統"] : []),
      ...heartbeats.filter((h) => h.alerts.length > 0).map((h) => `心跳「${h.job}」`),
    ];
    let sendError = null;
    if (alerts.length > 0) {
      try {
        await sendAlert(env, alerts, now, scopes);
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
      const now = Date.now();
      let prevPing = null;
      try {
        prevPing = await kvGetJSON(env, pingKey(job));
      } catch (e) {
        if (!(e instanceof KvUnavailable)) throw e;
        // 讀不到舊值就當作沒有節流依據——心跳本身比節流精確度重要，照樣寫入
      }
      if (!prevPing?.at || now - prevPing.at >= PING_WRITE_MIN_INTERVAL_MS) {
        await env.WATCHDOG_KV.put(pingKey(job), JSON.stringify({ at: now }));
      }
      return new Response(null, { status: 204 });
    }

    if (url.pathname === "/status") {
      if (request.method !== "GET") {
        return new Response("method not allowed", { status: 405 });
      }
      if (!authorized(request, env)) {
        return new Response("forbidden", { status: 403 });
      }
      // /status 是「人在查看門狗還活著嗎」的入口。KV 讀不到時回 503 並說清楚
      // 是 KV 的問題，比丟一個空白 500 有用——後者會讓人以為是 Worker 掛了。
      try {
        return Response.json(await buildStatus(env, Date.now()));
      } catch (e) {
        if (e instanceof KvUnavailable) {
          return Response.json({ error: "kv_unavailable", detail: e.message }, { status: 503 });
        }
        throw e;
      }
    }

    return new Response("not found", { status: 404 });
  },
};

// ============================================================================
// 探測（主動）
// ============================================================================

/*
 * KV 的讀取→判斷→寫入不是原子的，兩輪重疊執行時會遺失更新。這裡**明確接受**
 * 這個競態，不改用 Durable Object。理由是把後果算清楚之後它很小：
 *
 *   - `fails` 計數遺失一次 → 告警延後一輪（5 分鐘）。門檻本來就是 3 次 ≒ 15 分鐘，
 *     多 5 分鐘不改變任何處置。
 *   - `alerted` 兩邊都讀到 false → 同一件事寄兩封信。吵，但不會漏。
 *   - `lastOkAt` 遺失一次回寫 → 只影響告警信裡「最後一次正常」這行的精度。
 *
 * 三種後果都是「慢一點」或「吵一點」，**沒有一種會讓系統掛掉而不告警**——
 * 而那是本 Worker 唯一要守住的事。Durable Object 要付的是一個常駐計費物件、
 * 一層新的失敗模式（DO 本身不可用時怎麼辦），換到的只有上面這三項的精度。
 *
 * ⚠️ 這個判斷有前提：`scheduled` 是唯一的寫入者，而 Cloudflare 的 Cron Trigger
 * 對同一個排程不會刻意併發（重疊只可能來自重試）。**若哪天新增了第二個寫入者**
 * （例如讓 /ping 也去改 `state:hb:*`，或加第二條 cron），上面的算式就不成立，
 * 屆時要重新評估而不是沿用本註解。
 *
 * 相關：讀取**失敗**與「沒有資料」的區分是另一回事，那個必須修，見 KvUnavailable。
 */

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
/**
 * @returns {Promise<{at: number, commit: () => Promise<void>}>}
 *   `at`：可以立即使用的 bootstrap 基準時間。
 *   `commit`：真正把它寫進 KV 的動作，**延後到讀取階段全部成功之後才呼叫**。
 *
 * 為什麼要延後：這支函式在 `scheduled` 的讀取階段（見該處註解）被呼叫，
 * 那段的保證是「只讀不寫、KV 讀不到就在還沒動到任何狀態的位置乾淨中止」。
 * 若這裡在讀取階段當場 `put`，而同一輪後面的 `checkHealth`／`checkHeartbeat`
 * 讀取又失敗拋出 `KvUnavailable`，`state:bootstrap` 已經被寫下且不會回滾——
 * 一個「中止」的輪次卻推進了心跳的寬限基準時間，違反上面那段保證。
 * 回傳 thunk 讓呼叫端自己決定何時真正落地。
 */
async function ensureBootstrap(env, now) {
  const rec = await kvGetJSON(env, KV_BOOTSTRAP);
  if (rec?.at) {
    return { at: rec.at, commit: async () => {} };
  }
  return {
    at: now,
    commit: () => env.WATCHDOG_KV.put(KV_BOOTSTRAP, JSON.stringify({ at: now })),
  };
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
    job,
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

async function sendAlert(env, lines, now, scopes) {
  const down = lines.some((l) => l.startsWith("🔴") || l.startsWith("🟠"));
  // 主旨必須帶機制別。2026-09-05 失敗演練實測：主動探測與心跳的告警主旨完全相同，
  // Gmail 依主旨把兩封摺進同一個 thread——真實故障若接在別的告警之後，新的那封會被
  // 埋在舊 thread 裡而不顯眼。這對一個「唯一的外部告警管道」是不能接受的失敗模式。
  const subject = `[iPig 看門狗] ${down ? "異常" : "已恢復"}：${scopes.join(" + ")}`;
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

/**
 * KV 讀取失敗時丟出，用來跟「key 真的不存在」（`null`）區分開。
 *
 * 這個區分是必要的，不是潔癖：舊版把讀取失敗吞成 `null`，**兩個方向都會錯**——
 *
 *   - 心跳側**誤報**。`ping:<job>` 讀失敗 → `lastAt` fallback 到 bootstrapAt →
 *     bootstrap 只要夠舊就判定 overdue；同一輪 `state:hb:<job>` 也讀失敗 →
 *     `alerted` 當成 false → 沒有任何東西擋住，直接寄出「備份可能正在靜默失敗」。
 *     KV 一次短暫抖動就能無中生有一封告警。
 *   - 探測側**漏報**。`state:probe` 讀失敗 → `prev.fails` 歸零 → 連續失敗計數
 *     被重置成 1，湊不到 FAIL_THRESHOLD。真的掛掉時反而不告警。
 *
 * 這兩個剛好是看門狗最不該犯的兩種錯，而且成因是同一行 `return null`。
 */
class KvUnavailable extends Error {
  constructor(key, cause) {
    super(`KV 讀取失敗 key=${key}: ${cause?.message ?? String(cause)}`);
    this.name = "KvUnavailable";
    this.key = key;
    this.cause = cause;
  }
}

/**
 * @returns 解析後的值，或 `null`——**`null` 只代表 key 不存在**。
 * @throws {KvUnavailable} KV 讀不到時。呼叫端必須跳過本輪，不得當成無資料。
 */
async function kvGetJSON(env, key) {
  try {
    return await env.WATCHDOG_KV.get(key, { type: "json" });
  } catch (e) {
    console.error(`watchdog: KV 讀取失敗 key=${key}`, e?.stack ?? String(e));
    throw new KvUnavailable(key, e);
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
