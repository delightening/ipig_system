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
import { writeFileSync } from 'node:fs'
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

const PSQL_ARGS = ['psql', '-U', 'postgres', '-d', 'ipig_db', '-t', '-A', '-c', SQL]
const env = { ...process.env, MSYS_NO_PATHCONV: '1' }

// 先用固定容器名。
// ⚠️ 不先試 `docker compose exec`：worktree 裡沒有 `.env`（它是 gitignore 的，
// 只存在於主 checkout），compose 會在讀 env file 時就失敗。而容器名在
// docker-compose.yml 裡是寫死的 `container_name: ipig-db`，跨 worktree 都一樣。
const ATTEMPTS = [
  { label: 'docker exec ipig-db', argv: ['exec', '-i', 'ipig-db', ...PSQL_ARGS] },
  { label: 'docker compose exec db', argv: ['compose', 'exec', '-T', 'db', ...PSQL_ARGS] },
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

writeFileSync(OUT, header + names.join('\n') + '\n', 'utf8')
console.log(`已寫入 ${path.relative(repoRoot, OUT)}：${names.length} 個名字`)
