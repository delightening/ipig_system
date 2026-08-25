// pii-scan 的回歸測試。
//
// 為什麼需要它：這支掃描器是 commit message 個資外洩的**唯一**防線——
// 推出去之後 force-push 收不回來（commit 物件仍可用 SHA 存取，得請
// GitHub Support 執行 GC）。沒有測試的話，任何一次重構都可能靜默地
// 把某條路徑關掉，而症狀要等到下一次外洩才會出現。
//
// 2026-08-25 建立，起因是一則列出 6 位同仁姓名與各自負責計畫份數的
// commit message 連續三次 push 全數通過。事後查出兩個獨立缺口：
//   (1) 掃描來源只有 git diff 的新增行，commit message 從來沒被掃過
//   (2) 沒有任何人名類的樣式
// 下面每一個 describe 都對應其中一個缺口，或對應一條「修的時候差點做錯」的邊界。
//
// 執行：node --test scripts/__tests__/pii-scan.test.mjs

import { test, describe } from 'node:test'
import assert from 'node:assert/strict'
import { execFileSync, execSync } from 'node:child_process'
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = path.dirname(fileURLToPath(import.meta.url))
const SCANNER = path.join(HERE, '..', 'pii-scan.mjs')
const FIXTURE = path.join(HERE, 'pii-names.fixture.txt')

/** 跑掃描器，回傳 { code, out }。code 0 = 通過，1 = 擋下。 */
function runScanner(argv, { names = FIXTURE, cwd } = {}) {
  const env = { ...process.env }
  if (names === null) delete env.PII_NAMES_FILE
  else env.PII_NAMES_FILE = names
  try {
    const out = execFileSync(process.execPath, [SCANNER, ...argv], {
      encoding: 'utf8',
      env,
      cwd: cwd ?? path.join(HERE, '..', '..'),
      stdio: ['pipe', 'pipe', 'pipe'],
    })
    return { code: 0, out }
  } catch (err) {
    return { code: err.status ?? 1, out: (err.stdout ?? '') + (err.stderr ?? '') }
  }
}

function withTempMessage(text, fn) {
  const dir = mkdtempSync(path.join(tmpdir(), 'pii-msg-'))
  const file = path.join(dir, 'COMMIT_EDITMSG')
  writeFileSync(file, text, 'utf8')
  try {
    return fn(file)
  } finally {
    rmSync(dir, { recursive: true, force: true })
  }
}

describe('缺口 1：commit message 必須被掃', () => {
  test('🔴 訊息含名單上的人名 → 擋下', () => {
    withTempMessage('fix: 調整權限\n\n陳測試 8 份、林範例 7 份\n', (f) => {
      const { code, out } = runScanner(['--commit-msg', f])
      assert.equal(code, 1, '應該擋下，實際通過了')
      assert.match(out, /系統內真實人名/)
    })
  })

  test('訊息只描述性質、不點名 → 通過', () => {
    withTempMessage('fix: 調整權限\n\n絕大多數現任 SD 沒有結案權限，具體數字留在私有 docs。\n', (f) => {
      const { code } = runScanner(['--commit-msg', f])
      assert.equal(code, 0)
    })
  })

  test('git 自動加的 # 註解行不算數（那些不會進最終訊息）', () => {
    withTempMessage('fix: 調整權限\n\n# 這行是 git 的說明：陳測試\n', (f) => {
      const { code } = runScanner(['--commit-msg', f])
      assert.equal(code, 0)
    })
  })

  test('訊息檔路徑缺漏 → fail-closed（擋下，不是靜默放行）', () => {
    const { code } = runScanner(['--commit-msg'])
    assert.equal(code, 1)
  })

  test('訊息檔讀不到 → fail-closed', () => {
    const { code } = runScanner(['--commit-msg', path.join(tmpdir(), 'no-such-file-xyz')])
    assert.equal(code, 1)
  })
})

describe('缺口 1b：--push 也要掃 message，不能只靠 commit-msg hook', () => {
  // amend／rebase／cherry-pick／--no-verify 產生的訊息都沒經過 commit-msg。
  // 當初外洩就是 amend 之後 force-push。
  test('🔴 range 內某個 commit 的 message 含人名 → 擋下', () => {
    const repoRoot = path.join(HERE, '..', '..')
    // ⚠️ 自帶 author/committer：CI runner 上沒有 git user.email／user.name，
    // 少了這組 `git commit-tree` 會直接失敗，而失敗訊息看起來像「測試壞了」
    // 而不是「環境缺設定」。不用 `git config` 是為了不污染 runner 的全域狀態。
    const gitEnv = {
      ...process.env,
      GIT_AUTHOR_NAME: 'pii-scan-test',
      GIT_AUTHOR_EMAIL: 'pii-scan-test@example.com',
      GIT_COMMITTER_NAME: 'pii-scan-test',
      GIT_COMMITTER_EMAIL: 'pii-scan-test@example.com',
    }
    const head = execSync('git rev-parse HEAD', { cwd: repoRoot, encoding: 'utf8', env: gitEnv }).trim()
    const tree = execSync('git write-tree', { cwd: repoRoot, encoding: 'utf8', env: gitEnv }).trim()
    // 游離 commit：不掛在任何分支上，測完隨 GC 消失
    const bad = execSync(`git commit-tree ${tree} -p ${head}`, {
      cwd: repoRoot,
      encoding: 'utf8',
      env: gitEnv,
      input: 'chore: 測試\n\n陳測試 3 份\n',
    }).trim()

    // 這裡不走 runScanner：--push 的 refs 要從 stdin 餵，需要 input 選項
    let result
    try {
      execFileSync(process.execPath, [SCANNER, '--push', 'origin'], {
        encoding: 'utf8',
        cwd: repoRoot,
        env: { ...process.env, PII_NAMES_FILE: FIXTURE },
        input: `refs/heads/t ${bad} refs/heads/t ${head}\n`,
      })
      result = 0
    } catch (err) {
      result = err.status ?? 1
      assert.match((err.stdout ?? '') + (err.stderr ?? ''), /系統內真實人名/)
    }
    assert.equal(result, 1, '--push 應該擋下含人名的 commit message')
  })
})

describe('缺口 2：人名比對本身', () => {
  test('沒有名單檔 → 不擋，但通過訊息要講明人名比對未啟用', () => {
    // CI 與新 clone 沒有名單是正常的。為此擋下所有 commit 會逼人
    // --no-verify，反而把整個掃描器關掉。
    withTempMessage('fix: 調整權限\n\n陳測試 8 份\n', (f) => {
      const { code, out } = runScanner(['--commit-msg', f], {
        names: path.join(tmpdir(), 'definitely-not-here.txt'),
      })
      assert.equal(code, 0, '沒有名單時不該擋')
      assert.match(out, /人名比對未啟用/)
    })
  })

  test('單字名字不收（否則滿地誤判）', () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'pii-names-'))
    const names = path.join(dir, 'names.txt')
    writeFileSync(names, '陳\n王\n', 'utf8')
    try {
      withTempMessage('fix: 修正陳述式與王道流程\n', (f) => {
        const { code } = runScanner(['--commit-msg', f], { names })
        assert.equal(code, 0, '單字名字若被收進字典，一般詞彙會被誤擋')
      })
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  test('人名遮罩保留首字，讓人看得出命中哪一個', () => {
    withTempMessage('fix: x\n\n陳測試\n', (f) => {
      const { out } = runScanner(['--commit-msg', f])
      assert.match(out, /陳\*\*/, '應顯示「陳**」而非整串 ***')
    })
  })
})

describe('既有偵測類別沒有被破壞', () => {
  // ⚠️ 刻意拼接而不是寫成字面量：完整寫出來會被這支掃描器自己抓到，
  // 於是每次改本測試檔都得 --no-verify。pii-scan.mjs 內部處理 DSN 範例時
  // 用的是同一招（見該檔 isUrlUserinfo 上方註解）。
  // 這不是繞過檢查——被擋的是「測試資料長得像個資」，不是真的有個資。
  const NON_WHITELISTED_EMAIL = ['someone', '@', 'realdomain', '.tw'].join('')

  test('Email 仍會被抓', () => {
    withTempMessage(`fix: x\n\n聯絡 ${NON_WHITELISTED_EMAIL}\n`, (f) => {
      const { code, out } = runScanner(['--commit-msg', f])
      assert.equal(code, 1)
      assert.match(out, /Email/)
    })
  })

  test('白名單網域仍豁免', () => {
    withTempMessage('fix: x\n\nCo-Authored-By: Claude <noreply@anthropic.com>\n', (f) => {
      const { code } = runScanner(['--commit-msg', f])
      assert.equal(code, 0)
    })
  })

  test('未知模式 → exit 2（用法錯誤，與「擋下」區分開）', () => {
    const { code } = runScanner(['--nonsense'])
    assert.equal(code, 2)
  })
})
