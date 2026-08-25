// 從資料庫產生 pii-scan 用的人名字典。
//
// ⚠️ 產出的 scripts/pii-names.local.txt **是個資，已 gitignore，不得進版控**。
// 每台開發機各自產生一份；CI 與新 clone 沒有它是正常的（掃描器會退化成
// 不做人名比對，並在通過訊息裡註明）。
//
// 用法：
//   node scripts/gen-pii-names.mjs            從 docker compose 的 db 服務讀
//   node scripts/gen-pii-names.mjs --dry-run  只印筆數，不寫檔（確認連得上）
//
// 為什麼不讓 pii-scan 自己連 DB：hook 要在每次 commit 跑，連線失敗會變成
// 「資料庫沒開就不能 commit」。名單變動很慢，週期性重新產生就夠。

import { execFileSync } from 'node:child_process'
import { renameSync, writeFileSync } from 'node:fs'
import path from 'node:path'

const dryRun = process.argv.includes('--dry-run')
const repoRoot = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim()
const OUT = path.join(repoRoot, 'scripts/pii-names.local.txt')

// 只取顯示名稱。email／帳號已由既有的 email 樣式涵蓋，不必重複。
const SQL = `
SELECT DISTINCT display_name
FROM users
WHERE display_name IS NOT NULL
  AND length(btrim(display_name)) >= 2
ORDER BY display_name;
`.trim()

// ⚠️ 不寫死 `-U postgres -d ipig_db`：Compose 可以用 POSTGRES_USER／POSTGRES_DB
// 覆寫這兩個值，寫死的話在覆寫過的環境上會直接連不上。
// 改成在容器內展開容器自己的環境變數（`sh -c` 讓 $VAR 由容器的 shell 解析，
// 不是由本機 shell），沒設時才退回預設值。
const PSQL_IN_CONTAINER =
  'psql -U "${POSTGRES_USER:-postgres}" -d "${POSTGRES_DB:-ipig_db}" -t -A -c "$1"'
const env = { ...process.env, MSYS_NO_PATHCONV: '1' }

// 先用固定容器名。
// ⚠️ 不先試 `docker compose exec`：worktree 裡沒有 `.env`（它是 gitignore 的，
// 只存在於主 checkout），compose 會在讀 env file 時就失敗。而容器名在
// docker-compose.yml 裡是寫死的 `container_name: ipig-db`，跨 worktree 都一樣。
// `sh -c '<script>' sh "<sql>"`：SQL 當位置參數 $1 傳進去，不做字串拼接，
// 避免 SQL 內容被容器的 shell 再解析一次。
const ATTEMPTS = [
  {
    label: 'docker exec ipig-db',
    argv: ['exec', '-i', 'ipig-db', 'sh', '-c', PSQL_IN_CONTAINER, 'sh', SQL],
  },
  {
    label: 'docker compose exec db',
    argv: ['compose', 'exec', '-T', 'db', 'sh', '-c', PSQL_IN_CONTAINER, 'sh', SQL],
  },
]

let raw = null
const errors = []
for (const a of ATTEMPTS) {
  try {
    raw = execFileSync('docker', a.argv, { encoding: 'utf8', cwd: repoRoot, env })
    break
  } catch (err) {
    errors.push(`  ${a.label}：${String(err.message).split('\n')[0]}`)
  }
}
if (raw === null) {
  console.error('讀不到資料庫，兩種連法都失敗：')
  console.error(errors.join('\n'))
  console.error('確認資料庫容器有起來（docker ps | grep ipig-db）。')
  process.exit(1)
}

const names = [...new Set(raw.split('\n').map((l) => l.trim()).filter((l) => l.length >= 2))].sort()

if (!names.length) {
  console.error('查出 0 個名字——不覆蓋現有檔案，先確認查詢是否正確。')
  process.exit(1)
}

if (dryRun) {
  console.log(`[dry-run] 查到 ${names.length} 個名字，未寫檔。`)
  process.exit(0)
}

const header = [
  '# pii-scan 人名字典（自動產生，請勿手動編輯）',
  '# 產生指令：node scripts/gen-pii-names.mjs',
  '# ⚠️ 本檔含個資，已 gitignore，不得進版控。',
  '',
].join('\n')

// ⚠️ 原子寫入：先寫暫存檔再 rename。
// 直接 writeFileSync 到 OUT 的話，中途被打斷（Ctrl-C、磁碟滿、行程被殺）
// 會留下一份**截斷的字典**，而 pii-scan 讀到它照樣正常運作——
// 只是少了後半段的名字，於是那些人的姓名就悄悄通過檢查。
// rename 在同一個檔案系統上是原子操作：要嘛舊的完整檔、要嘛新的完整檔。
const tmp = OUT + '.tmp'
writeFileSync(tmp, header + names.join('\n') + '\n', 'utf8')
renameSync(tmp, OUT)
console.log(`已寫入 ${path.relative(repoRoot, OUT)}：${names.length} 個名字`)
