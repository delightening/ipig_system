/**
 * 看門狗的判讀正確性測試（R93-2）。
 *
 * 為什麼這幾支測試值得存在：這個 Worker 是「系統掛了」的唯一外部偵測管道，
 * 而它在此之前**一支測試都沒有**（`package.json` 的 `check` 只做語法檢查）。
 * R93-2 那個 bug——KV 讀取失敗被吞成 `null`——正是沒有網子才活得下來的形狀：
 * 程式看起來有錯誤處理（有 try/catch、有 console.error），只是處理錯了方向。
 *
 * 跑法：`npm test`（或 `node --import ./test/register.mjs --test test/`）。
 * 不需要 wrangler、不連網、零 devDependency。
 */
import { test } from "node:test";
import assert from "node:assert/strict";

import worker from "../src/worker.js";

const HOUR = 60 * 60 * 1000;

/**
 * 🔴 **必須是執行當下的時間，不能寫死日期。**
 *
 * 這裡原本是 `Date.UTC(2026, 8, 3, 12, 0, 0)`，看起來像「固定時間讓測試可重現」，
 * 實際上是顆定時炸彈：`worker.scheduled()` 內部用的是真實的 `Date.now()`，沒有可注入的
 * 時鐘。所以寫死的 NOW 只固定了「餵進 KV 的資料」，比較對象仍是真實時間——
 * 兩者的差距每過一天就多一天。
 *
 * 具體引爆點：下面「連續失敗達門檻」那支測試餵 `state:bootstrap = NOW`，而該測試沒有
 * `ping:backup`，心跳的 `lastAt` 就 fallback 到 bootstrap。一旦真實時間超過
 * NOW + 26h（BACKUP_MAX_AGE），第一次 `scheduled()` 就會判定心跳逾期而寄出告警，
 * 於是「第 1 次失敗不該告警」的斷言被一封**心跳**告警打掛——與健康檢查門檻無關。
 * 實測：把 NOW 往前推 12 小時（模擬過了引爆點），7 支測試裡剛好只有那一支轉紅。
 *
 * 改成 `Date.now()` 之後，所有測試資料都是相對於執行當下，永遠不會過期。
 * 斷言本來就只依賴相對時間差、不依賴絕對時間值（沒有任何一支斷言格式化後的時間字串），
 * 所以不損失可重現性。**不要為了「看起來比較確定」再改回寫死的日期。**
 */
const NOW = Date.now();
/** 與 worker.js 的 HEARTBEAT_JOBS.backup 一致。 */
const BACKUP_MAX_AGE = 26 * HOUR;

/**
 * 假的 KV。`data` 是 key → 值；`failOn` 列出讀取要丟例外的 key。
 * `puts` 記錄所有寫入，讓「本輪不該動到任何狀態」變成可斷言的事實。
 */
function makeKv({ data = {}, failOn = [] } = {}) {
  const puts = [];
  return {
    puts,
    async get(key) {
      if (failOn.includes(key)) {
        throw new Error(`simulated KV outage on ${key}`);
      }
      return Object.hasOwn(data, key) ? data[key] : null;
    },
    async put(key, value) {
      puts.push({ key, value });
      data[key] = JSON.parse(value);
    },
  };
}

function makeEnv(kv, { healthOk = true } = {}) {
  const sent = [];
  return {
    sent,
    WATCHDOG_KV: kv,
    HEALTH_URL: "https://example.invalid/api/health",
    ALERT_FROM: "watchdog@example.invalid",
    ALERT_TO: "ops@example.invalid",
    PING_TOKEN: "t".repeat(32),
    FAIL_THRESHOLD: "3",
    ALERT_EMAIL: {
      async send(msg) {
        sent.push(msg);
      },
    },
    _healthOk: healthOk,
  };
}

/** worker.js 用全域 fetch 打 HEALTH_URL；這裡換掉它，測試不連網。 */
function withHealth(ok, fn) {
  const original = globalThis.fetch;
  globalThis.fetch = async () =>
    ok
      ? new Response(JSON.stringify({ status: "healthy" }), { status: 200 })
      : new Response("boom", { status: 503 });
  return fn().finally(() => {
    globalThis.fetch = original;
  });
}

// ---------------------------------------------------------------------------
// R93-2：KV 讀取失敗 ≠ 沒有資料
// ---------------------------------------------------------------------------

test("KV 讀取失敗時整輪中止：不寄信、不回寫任何狀態", async () => {
  const kv = makeKv({
    data: { "state:bootstrap": { at: NOW - 10 * 24 * HOUR } },
    failOn: ["ping:backup"],
  });
  const env = makeEnv(kv);

  await withHealth(true, async () => {
    await assert.rejects(
      () => worker.scheduled({}, env, {}),
      (e) => e.name === "KvUnavailable" && e.key === "ping:backup",
      "KV 讀不到必須往上拋，不能當成 key 不存在",
    );
  });

  assert.deepEqual(env.sent, [], "本輪不得寄出任何信——資料是假的，判斷不算數");
  assert.deepEqual(kv.puts, [], "本輪不得回寫任何狀態，下一輪要能從乾淨的位置重來");
});

test("bootstrap 缺席時，後面的讀取失敗不得讓 bootstrap 先落地", async () => {
  // 與上一支的差異：上一支 state:bootstrap 已存在（ensureBootstrap 讀到既有值，
  // 不會嘗試寫入），測不到 ensureBootstrap 自己的寫入時機。這支刻意留空，
  // 讓 ensureBootstrap 判定「首次啟動」而準備寫入——寫入是否真的被延後到
  // 全部讀取成功之後，只有在這個情境下才驗證得到。
  const kv = makeKv({
    data: {}, // state:bootstrap 不存在
    failOn: ["ping:backup"], // ensureBootstrap 之後的讀取才失敗
  });
  const env = makeEnv(kv);

  await withHealth(true, async () => {
    await assert.rejects(
      () => worker.scheduled({}, env, {}),
      (e) => e.name === "KvUnavailable" && e.key === "ping:backup",
    );
  });

  assert.ok(
    !kv.puts.some((p) => p.key === "state:bootstrap"),
    "讀取階段中止後，本該延後的 bootstrap 落地必須真的沒發生——" +
      "否則一個被判定為「中止」的輪次，會安靜地把心跳寬限基準往前推",
  );
  assert.deepEqual(kv.puts, [], "本輪不得回寫任何狀態");
});

test("舊語意重現：同一情境下，把讀取失敗吞成 null 會直接產生一封誤報", () => {
  // 這裡不是測 worker，而是把**舊版的錯誤語意**原樣寫下來並排比較，
  // 讓「這次到底修掉了什麼」變成可執行的斷言，而不是只寫在 commit message 裡。
  // 舊版：async function kvGetJSON(env, key) { try { ... } catch { return null } }
  const oldKvGetJSON = (thrown) => {
    try {
      if (thrown) throw new Error("simulated KV outage");
      return null;
    } catch {
      return null; // ← 就是這一行
    }
  };

  const bootstrapAt = NOW - 10 * 24 * HOUR; // 看門狗已上線 10 天
  const ping = oldKvGetJSON(true); // ping:backup 讀失敗 → null
  const state = oldKvGetJSON(true) ?? { alerted: false }; // state:hb:backup 也讀失敗
  const lastAt = ping?.at ?? bootstrapAt; // fallback 到 bootstrap
  const overdue = NOW - lastAt > BACKUP_MAX_AGE;

  assert.ok(
    overdue && !state.alerted,
    "舊語意下，KV 一次抖動就滿足「逾期且尚未告警」→ 無中生有一封「備份可能正在靜默失敗」",
  );
});

test("key 不存在（真的沒有資料）仍照舊運作，不受本次修正影響", async () => {
  const kv = makeKv(); // 全空，所有 get 回 null
  const env = makeEnv(kv);

  await withHealth(true, () => worker.scheduled({}, env, {}));

  assert.deepEqual(env.sent, [], "第一次跑不該告警——bootstrap 就是為了避免部署當天誤報");
  assert.ok(
    kv.puts.some((p) => p.key === "state:bootstrap"),
    "首輪應寫入 bootstrap 基準時間",
  );
});

test("/status 在 KV 壞掉時回 503 並指明是 KV，不是丟一個空白 500", async () => {
  const kv = makeKv({ failOn: ["state:probe"] });
  const env = makeEnv(kv);
  const req = new Request("https://w.invalid/status", {
    headers: { authorization: `Bearer ${env.PING_TOKEN}` },
  });

  const res = await worker.fetch(req, env);

  assert.equal(res.status, 503);
  const body = await res.json();
  assert.equal(body.error, "kv_unavailable");
  assert.match(body.detail, /state:probe/);
});

// ---------------------------------------------------------------------------
// 迴歸護欄：確認修正沒有把正常的告警路徑一起關掉
// ---------------------------------------------------------------------------

test("連續失敗達門檻才告警，未達門檻不吵", async () => {
  const kv = makeKv({ data: { "state:bootstrap": { at: NOW } } });
  const env = makeEnv(kv);

  await withHealth(false, () => worker.scheduled({}, env, {})); // 第 1 次
  assert.deepEqual(env.sent, [], "第 1 次失敗不該告警（門檻 3）");

  await withHealth(false, () => worker.scheduled({}, env, {})); // 第 2 次
  assert.deepEqual(env.sent, [], "第 2 次失敗仍不該告警");

  await withHealth(false, () => worker.scheduled({}, env, {})); // 第 3 次
  assert.equal(env.sent.length, 1, "第 3 次達門檻，應寄出一封告警");
});

test("心跳逾期會告警——KvUnavailable 的修正沒有把這條路一起擋掉", async () => {
  const kv = makeKv({
    data: {
      "state:bootstrap": { at: NOW - 10 * 24 * HOUR },
      "ping:backup": { at: NOW - 30 * HOUR }, // 超過 26 小時門檻
    },
  });
  const env = makeEnv(kv);

  await withHealth(true, () => worker.scheduled({}, env, {}));

  assert.equal(env.sent.length, 1, "真的逾期時必須告警");
  assert.ok(
    kv.puts.some((p) => p.key === "state:hb:backup"),
    "告警送出後應把 alerted 狀態寫回，避免每 5 分鐘重寄",
  );
});
