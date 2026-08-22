// 個資（PII）掃描器 — 阻擋含疑似個資的 git commit/push。
//
// 偵測類別：台灣身分證/居留證字號（含檢查碼驗證）、手機/市話號碼、Email、
// 具體門牌地址（路/街+號）。regex + heuristic，會有誤判，遇到誤判用
// `git commit --no-verify` / `git push --no-verify` 略過（git 內建機制）。
//
// 用法：
//   node scripts/pii-scan.mjs --staged         掃 staged 內容新增的行（pre-commit 用）
//   node scripts/pii-scan.mjs --push <remote> <url>  掃即將 push 的 commit range（pre-push 用，refs 從 stdin 讀）
//   node scripts/pii-scan.mjs --full            掃整個目前 tracked 檔案（一次性歷史稽核用，不阻擋）
//
// Exit code：--staged / --push 有命中 → 1（阻擋）；--full 永遠 0（只回報）。

import { execFileSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import path from 'node:path'

const EXCLUDE_PATH_RE = /(^|\/)(node_modules|dist|target|__pycache__|\.git|scripts\/trivy_bin|coverage|\.venv|venv)(\/|$)/
const EXCLUDE_FILENAMES = new Set(['pnpm-lock.yaml', 'package-lock.json', 'Cargo.lock'])
const BINARY_EXT_RE = /\.(png|jpe?g|gif|ico|bmp|webp|svg|pdf|docx?|xlsx?|pptx?|zip|7z|tar|gz|rar|exe|dll|so|dylib|wasm|woff2?|eot|ttf|otf|pyc|db|sqlite3?|lock|ipynb|mp[34]|mov|avi|class|jar)$/i
const MAX_FULL_SCAN_BYTES = 2 * 1024 * 1024
const MAX_LINE_SCAN_CHARS = 200_000
const OID_RE = /^[0-9a-f]{40}$/i

function shouldSkipPath(relPath) {
  const normalized = relPath.replace(/\\/g, '/')
  if (EXCLUDE_PATH_RE.test(normalized)) return true
  if (EXCLUDE_FILENAMES.has(path.basename(normalized))) return true
  if (BINARY_EXT_RE.test(normalized)) return true
  return false
}

// 台灣身分證/居留證字號檢查碼（標準演算法：字母轉兩位數代碼 + 加權總和 mod 10）
const LETTER_CODES = {
  A: 10, B: 11, C: 12, D: 13, E: 14, F: 15, G: 16, H: 17, I: 34, J: 18,
  K: 19, L: 20, M: 21, N: 22, O: 35, P: 23, Q: 24, R: 25, S: 26, T: 27,
  U: 28, V: 29, W: 32, X: 30, Y: 31, Z: 33,
}
function isValidTwId(id) {
  const code = LETTER_CODES[id[0]]
  if (code === undefined) return false
  const y1 = Math.floor(code / 10)
  const y2 = code % 10
  const d = id.slice(1).split('').map(Number)
  if (d.length !== 9 || d.some(Number.isNaN)) return false
  const weights = [8, 7, 6, 5, 4, 3, 2, 1, 1]
  const sum = y1 * 1 + y2 * 9 + d.reduce((acc, digit, i) => acc + digit * weights[i], 0)
  return sum % 10 === 0
}

const SAFE_EMAIL_DOMAINS = [
  'anthropic.com', 'github.com', 'githubusercontent.com',
  'example.com', 'example.org', 'example.net', 'test.com', 'localhost',
  'sentry.io', 'w3.org', 'w3c.org',
]
// 連線字串的 IP host 會被上面的 email regex 誤判：
// `postgres://user:password@127.0.0.1:5432/db` 會被吃成「userinfo 的密碼段 + IP host」
// （`:` 不在字元類內，比對停在最後一段數字），導致每次備份含 DSN 的文件都被擋，
// 逼人用 --no-verify 繞過整個掃描器。
//
// 豁免條件是「合法 IPv4」**且**「出現在 URL 的 userinfo 位置」，兩者缺一不可：
//   postgres://user:password@127.0.0.1/db  → 豁免
//   同樣形狀但前面沒有 scheme://（獨立出現）→ 照樣回報
// （本註解刻意不寫出獨立形式的實例——寫了會被這支掃描器自己抓到，
//   每次修改本檔都要 --no-verify，那正是本次要消除的行為。）
//
// 沿革（PR #148，三輪收窄，每輪的洞都是 bot review 抓出來的）：
//   一版 /^\d+(\.\d+)*$/ —— 任何點分隔數字串都放行（CodeAnt Major + pr-agent）
//   二版 嚴格 IPv4       —— 合法 IPv4 仍無條件放行、與出現位置無關，屬 repo 範圍的
//                           false negative（CodeAnt [security] + Qodo Rule 2611600
//                           兩家獨立指出，且 Qodo 兩輪結論一致）
//   三版 IPv4 + URL 上下文（本版）
//
// IPv6 不需處理——URL 裡是 `[::1]` 形式，方括號與冒號都不在 email regex 的字元類內。
function isIpLikeHost(domain) {
  const parts = domain.split('.')
  if (parts.length !== 4) return false
  return parts.every((p) => /^\d{1,3}$/.test(p) && Number(p) <= 255)
}
// match 是否位於 URL 的 authority 段（scheme://user:pass@host）。往前找最近的 `://`，
// 要求它與 match 起點之間不含任何「已經離開 authority」的字元：
//   空白／括號／引號 → 那個 `://` 屬於同一行稍早的另一段文字，不是同一個 URL
//   `/` `?` `#`      → authority 已結束，後面是 path／query／fragment。少了這三個，
//                      出現在 path 或 query 裡的 `<user>@<ipv4>` 會被誤判成 userinfo
//                      （CodeAnt + Qodo 於 PR #148 第四輪各自獨立指出）。
//                      此處刻意用佔位符：寫出實例會被本掃描器自己抓到——
//                      同一支 PR 內已經因此絆倒四次。
// 刻意不限制往前看的距離：加上 `/` 邊界後，再遠的 `://` 也不會誤判（中間必然先撞到 `/`），
// 而固定回看長度反而會讓「username 很長的合法 DSN」找不到 scheme 被誤擋
// （Qodo 同輪的 correctness finding：原本寫死 200 字元）。
function isUrlUserinfo(text, index) {
  const before = text.slice(0, index)
  const schemePos = before.lastIndexOf('://')
  if (schemePos === -1) return false
  return !/[\s"'`<>()[\]{},;\/?#]/.test(before.slice(schemePos + 3))
}
function isSafeEmail(addr) {
  const domain = addr.split('@')[1]?.toLowerCase()
  if (!domain) return false
  return SAFE_EMAIL_DOMAINS.some((d) => domain === d || domain.endsWith('.' + d))
}
// 兩個條件同時成立才算連線字串片段，缺一即照常回報。
function isDsnCredential(addr, ctx) {
  const domain = addr.split('@')[1]?.toLowerCase()
  if (!domain || !isIpLikeHost(domain)) return false
  return isUrlUserinfo(ctx.text, ctx.index)
}

const PATTERNS = [
  {
    key: 'tw_id',
    label: '台灣身分證/居留證字號',
    regex: /\b[A-Z][1289]\d{8}\b/g,
    validate: (m) => isValidTwId(m),
  },
  {
    key: 'phone_mobile',
    label: '手機號碼',
    regex: /\b09\d{2}[-\s]\d{3}[-\s]?\d{3}\b|\b09\d{8}\b/g,
  },
  {
    key: 'phone_landline',
    label: '市話（含區碼）',
    regex: /\(0[2-9]\)[-\s]?\d{3,4}[-\s]?\d{3,4}\b|\b0[2-9][-\s]\d{3,4}[-\s]\d{3,4}\b/g,
  },
  {
    key: 'email',
    label: 'Email',
    regex: /\b[\w.+-]+@[\w-]+\.[\w.-]+\b/g,
    exclude: (m, ctx) => isSafeEmail(m) || isDsnCredential(m, ctx),
  },
  {
    key: 'address',
    label: '具體門牌地址',
    regex: /[一-龥]{2,6}(?:路|街|大道|巷|弄)[一-龥0-9]{0,12}\d+號/g,
  },
]

function mask(s) {
  if (s.length <= 4) return '*'.repeat(s.length)
  return s.slice(0, 2) + '*'.repeat(s.length - 4) + s.slice(-2)
}

function scanLine(text) {
  const hits = []
  for (const p of PATTERNS) {
    p.regex.lastIndex = 0
    let m
    while ((m = p.regex.exec(text))) {
      const matched = m[0]
      if (p.validate && !p.validate(matched)) continue
      // 傳入整行與 match 起點：exclude 需要上下文才能分辨「URL 裡的 host」與
      // 「獨立出現的 user@ip」——只看 matched 字串兩者長得一模一樣。
      if (p.exclude && p.exclude(matched, { text, index: m.index })) continue
      hits.push({ key: p.key, label: p.label, matched })
    }
  }
  return hits
}

function git(argv) {
  return execFileSync('git', argv, { encoding: 'utf8', maxBuffer: 1024 * 1024 * 200 })
}

// 解析 `git diff -U0` 輸出，只取新增（+）的行，附帶新檔行號。
// 只在緊接在 `--- ` 之後才把 `+++ ` 視為檔案標頭——否則新增內容若本身以
// `++ ` 開頭（diff 呈現為 `+++ ...`）會被誤判成標頭而跳過掃描，等於留了個繞過口。
function parseAddedLines(diffText) {
  const results = []
  let currentFile = null
  let newLineNum = null
  let sawOldHeader = false
  for (const line of diffText.split('\n')) {
    if (line.startsWith('--- ')) {
      sawOldHeader = true
      continue
    }
    if (sawOldHeader && line.startsWith('+++ ')) {
      sawOldHeader = false
      const p = line.slice(4).trim()
      currentFile = p === '/dev/null' ? null : p.replace(/^b\//, '')
      newLineNum = null
      continue
    }
    sawOldHeader = false
    if (line.startsWith('@@')) {
      const m = /\+(\d+)/.exec(line)
      newLineNum = m ? parseInt(m[1], 10) : null
      continue
    }
    if (!currentFile || newLineNum === null) continue
    if (line.startsWith('+')) {
      results.push({ file: currentFile, line: newLineNum, text: line.slice(1) })
      newLineNum++
    }
    // '-' 開頭的移除行不佔新檔行號，其餘（-U0 下理論上不會有 context 行）忽略
  }
  return results
}

function findingsFromAddedLines(addedLines) {
  const findings = []
  for (const { file, line, text } of addedLines) {
    if (shouldSkipPath(file)) continue
    const hits = scanLine(text.slice(0, MAX_LINE_SCAN_CHARS))
    if (hits.length) findings.push({ file, line, hits })
  }
  return findings
}

function report(findings, { blocking }) {
  if (!findings.length) {
    if (blocking) console.log('[pii-scan] 未發現疑似個資，通過。')
    return
  }
  console.error(`\n[pii-scan] 發現 ${findings.length} 處疑似個資：\n`)
  for (const f of findings) {
    for (const h of f.hits) {
      console.error(`  ${f.file}:${f.line}  [${h.label}]  ${mask(h.matched)}`)
    }
  }
  if (blocking) {
    console.error(`
這些內容符合台灣個資格式（身分證字號／電話／Email／門牌地址），已阻擋本次操作。
- 若確實是真實個資 → 從變更中移除該內容（改用去識別化的假資料）再重新 commit/push。
- 若確定是誤判（測試假資料、範例字串等）→ 可用 \`--no-verify\` 略過這次檢查
  （git commit --no-verify / git push --no-verify），但請先確認真的是誤判。
`)
  }
}

function cmdStaged() {
  // --no-renames：rename+改內容時強制拆成 delete+add，內容一定會落在可掃的一側
  // --diff-filter=ACMR：R 是防禦性保留（例如未來拿掉 --no-renames），非主要防線
  let diff
  try {
    diff = git(['diff', '--cached', '-U0', '--no-color', '--no-renames', '--diff-filter=ACMR'])
  } catch (err) {
    console.error(`[pii-scan] 無法取得 staged diff，為安全起見擋下：${err.message}`)
    process.exit(1)
  }
  const findings = findingsFromAddedLines(parseAddedLines(diff))
  report(findings, { blocking: true })
  process.exit(findings.length ? 1 : 0)
}

async function readStdin() {
  const chunks = []
  for await (const chunk of process.stdin) chunks.push(chunk)
  return Buffer.concat(chunks).toString('utf8')
}

// 找不到可信 base 回傳 null，交由呼叫端 fail-closed（不可靜默放行 push）
function resolveBase(localOid, remoteOid, remote) {
  const ZERO = '0'.repeat(40)
  if (remoteOid && remoteOid !== ZERO) return remoteOid
  try {
    const ref = git(['symbolic-ref', '-q', `refs/remotes/${remote}/HEAD`]).trim()
    if (ref) {
      const base = git(['merge-base', localOid, ref]).trim()
      if (base) return base
    }
  } catch {
    // remote 沒有已知的預設分支（例如剛加的 remote 還沒 fetch 過）
  }
  return null
}

async function cmdPush() {
  const remote = process.argv[3] || 'origin'
  const stdin = await readStdin()
  const ZERO = '0'.repeat(40)
  let allFindings = []
  const blockers = []
  for (const line of stdin.split('\n')) {
    const parts = line.trim().split(/\s+/)
    if (parts.length < 4) continue
    const [, localOid, , remoteOid] = parts
    if (localOid === ZERO) continue // 刪除 ref，不需掃描
    if (!OID_RE.test(localOid) || (remoteOid !== ZERO && !OID_RE.test(remoteOid))) {
      blockers.push(`ref 資訊格式異常，無法安全解析，為安全起見擋下：${line}`)
      continue
    }
    const base = resolveBase(localOid, remoteOid, remote)
    if (base === null) {
      blockers.push(`${localOid.slice(0, 8)} 無法推導掃描起點（remote "${remote}" 沒有已知的預設分支），為安全起見擋下。確認乾淨後可用 --no-verify 略過，或先跑 \`pnpm run scan:pii\` 全庫確認。`)
      continue
    }
    let commitCount
    try {
      commitCount = parseInt(git(['rev-list', '--count', `${base}..${localOid}`]).trim(), 10)
    } catch {
      blockers.push(`${base.slice(0, 8)}..${localOid.slice(0, 8)} 無法計算 commit range，為安全起見擋下。`)
      continue
    }
    if (commitCount > 500) {
      blockers.push(`${base.slice(0, 8)}..${localOid.slice(0, 8)} 共 ${commitCount} 個 commit，範圍過大無法即時全掃，為安全起見擋下。確認乾淨後可用 --no-verify 略過，或先跑 \`pnpm run scan:pii\` 全庫確認。`)
      continue
    }
    let diff
    try {
      diff = git(['diff', '-U0', '--no-color', '--no-renames', '--diff-filter=ACMR', `${base}..${localOid}`])
    } catch {
      blockers.push(`${base.slice(0, 8)}..${localOid.slice(0, 8)} 無法取得 diff，為安全起見擋下。`)
      continue
    }
    allFindings = allFindings.concat(findingsFromAddedLines(parseAddedLines(diff)))
  }
  report(allFindings, { blocking: true })
  for (const b of blockers) console.error(`[pii-scan] ${b}`)
  process.exit(allFindings.length || blockers.length ? 1 : 0)
}

function cmdFull() {
  const files = git(['ls-files']).split('\n').filter(Boolean)
  const findings = []
  for (const file of files) {
    if (shouldSkipPath(file)) continue
    let stat
    try {
      stat = readFileSync(file)
    } catch {
      continue
    }
    if (stat.length > MAX_FULL_SCAN_BYTES) continue
    if (stat.includes(0)) continue // 二進位檔（含 null byte）跳過
    const text = stat.toString('utf8')
    const lines = text.split('\n')
    for (let i = 0; i < lines.length; i++) {
      const hits = scanLine(lines[i].slice(0, MAX_LINE_SCAN_CHARS))
      if (hits.length) findings.push({ file, line: i + 1, hits })
    }
  }
  report(findings, { blocking: false })
  console.log(`\n[pii-scan --full] 掃描 ${files.length} 個 tracked 檔案，命中 ${findings.length} 處（此模式僅回報，不阻擋）。`)
  process.exit(0)
}

const mode = process.argv[2]
if (mode === '--staged') cmdStaged()
else if (mode === '--push') cmdPush()
else if (mode === '--full') cmdFull()
else {
  console.error('用法：node scripts/pii-scan.mjs --staged | --push <remote> <url> | --full')
  process.exit(2)
}
