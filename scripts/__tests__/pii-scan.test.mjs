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

  // ⚠️ 這支原本斷言相反的行為（「# 開頭的行不算數」），是錯的。
  // CodeRabbit 於 PR #24 指出：commit-msg hook 拿到的是 **git cleanup 之前**的
  // 緩衝內容，而 cleanup mode 為 whitespace／scissors／verbatim 時，`#` 行會
  // 原封不動留在最終 message 裡——hook 無從得知會用哪個 mode。
  test('🔴 `#` 開頭的行照樣要掃——cleanup mode 可能保留它們', () => {
    withTempMessage('fix: 調整權限\n\n# 陳測試 8 份\n', (f) => {
      const { code, out } = runScanner(['--commit-msg', f])
      assert.equal(code, 1, '`#` 行被跳過的話，改用 --cleanup=verbatim 就能繞過')
      assert.match(out, /系統內真實人名/)
    })
  })

  test('🔴 已提交的訊息裡 `#` 行是真實內容，不是註解', () => {
    // git commit -m $'fix: x\n\n# <個資>' 產生的 message 就長這樣
    withTempMessage('fix: x\n\n# 林範例 3 份\n', (f) => {
      const { code } = runScanner(['--commit-msg', f])
      assert.equal(code, 1)
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

  // CodeRabbit 於 #24 指出：git 會把 message 重編碼成 `i18n.logOutputEncoding`
  // 指定的編碼，而掃描器一律以 UTF-8 解碼 stdout。兩者不一致 → 整段變亂碼 →
  // 人名一個都對不上 → 印「通過」。
  //
  // ⚠️ 這支測試用 **Big5**，不是 ISO-8859-1。原因是實測發現：
  // 中文轉 ISO-8859-1 時 iconv 失敗，而 git 在 iconv 失敗時**原封不動輸出**，
  // 於是恰好是安全的——拿它當測資會得到「沒問題」的假結論。
  // 會出事的是「轉得過去」的編碼，Big5 對台灣的專案來說正是最可能被設的那個。
  test('🔴 i18n.logOutputEncoding=Big5 時仍要掃得到人名', () => {
    const dir = mkdtempSync(path.join(tmpdir(), 'pii-enc-'))
    const gitEnv = {
      ...process.env,
      GIT_AUTHOR_NAME: 'pii-scan-test',
      GIT_AUTHOR_EMAIL: 'pii-scan-test@example.com',
      GIT_COMMITTER_NAME: 'pii-scan-test',
      GIT_COMMITTER_EMAIL: 'pii-scan-test@example.com',
    }
    const g = (cmd, opts = {}) =>
      execSync(cmd, { cwd: dir, encoding: 'utf8', env: gitEnv, ...opts })
    try {
      g('git init -q .')
      g('git commit --allow-empty -q -m "base"')
      const head = g('git rev-parse HEAD').trim()
      const tree = g('git write-tree').trim()
      const bad = g(`git commit-tree ${tree} -p ${head}`, {
        input: 'chore: 測試\n\n陳測試 3 份\n',
      }).trim()
      // 🔴 關鍵設定：讓 git 把 message 轉成 Big5 才輸出
      g('git config i18n.logOutputEncoding Big5')

      let result
      try {
        execFileSync(process.execPath, [SCANNER, '--push', 'origin'], {
          encoding: 'utf8',
          cwd: dir,
          env: { ...gitEnv, PII_NAMES_FILE: FIXTURE },
          input: `refs/heads/t ${bad} refs/heads/t ${head}\n`,
        })
        result = 0
      } catch (err) {
        result = err.status ?? 1
      }
      assert.equal(result, 1, 'Big5 輸出編碼下仍應擋下——不能因為亂碼就當作乾淨')
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })
})

describe('git cleanup mode：`#` 行在各模式下都要被掃到', () => {
  // CodeRabbit 於 PR #24 要求「add coverage for the relevant Git cleanup modes」。
  // 這裡不去真的跑 git commit（那會污染 repo 且慢），而是驗證掃描器對
  // 「cleanup 後可能留下 `#` 行」的那幾種訊息形狀都不放過。
  // 掃描器不看 cleanup mode——它一律全掃，這正是要守住的行為。
  const SHAPES = [
    ['verbatim：整份原樣保留', 'fix: x\n\n# 陳測試\n'],
    ['scissors：剪刀線之前的 # 行保留', 'fix: x\n\n# 林範例\n# ------------------------ >8 ------------------------\n# 以下不會進 message\n'],
    ['whitespace：只去空白，# 行保留', 'fix: x\n\n#   王假名   \n'],
    ['# 出現在行首以外（不受任何 cleanup 影響）', 'fix: x\n\n備註 # 甲乙丙\n'],
  ]
  for (const [label, msg] of SHAPES) {
    test(label, () => {
      withTempMessage(msg, (f) => {
        const { code } = runScanner(['--commit-msg', f])
        assert.equal(code, 1, `這個形狀應被擋下：${JSON.stringify(msg)}`)
      })
    })
  }
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

  // --full 是拿來做全庫稽核的。「掃完 0 命中」若被讀成「全庫乾淨」，
  // 而人名維度其實根本沒跑，那個稽核結論就是錯的。
  test('🔴 --full 沒有名單時要明講人名比對未啟用', () => {
    const { code, out } = runScanner(['--full'], {
      names: path.join(tmpdir(), 'definitely-not-here.txt'),
    })
    assert.equal(code, 0, '--full 永遠不阻擋')
    assert.match(out, /人名比對未啟用/)
  })

  test('--full 有名單時不印那則警告', () => {
    const { code, out } = runScanner(['--full'])
    assert.equal(code, 0)
    assert.doesNotMatch(out, /人名比對未啟用/)
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

// CodeRabbit 於 PR #24 指出：超長的單行被 `slice(0, MAX_LINE_SCAN_CHARS)`
// 默默截斷，第 200,000 字元之後的個資看不到，而檢查照樣印「通過」。
//
// 這是最糟的失敗方式——有一道閘、閘說沒事、其實根本沒看完。
// 修法不是拿掉上限（那會把「漏掉個資」換成「hook 掛住」），
// 而是**阻擋模式下把超長行本身當成 finding**。
describe('超長行：阻擋模式必須 fail closed，不得默默截斷', () => {
  const LIMIT = 200_000

  test('🔴 人名藏在上限之後 → 仍被擋下', () => {
    const msg = `fix: something\n\n${'x'.repeat(LIMIT + 10)}王大明`
    withTempMessage(msg, (f) => {
      const { code, out } = runScanner(['--commit-msg', f])
      assert.equal(code, 1, `應被擋下，實得 exit ${code}\n${out}`)
      assert.match(out, /超過 200000 上限/)
    })
  })

  test('恰好等於上限的行不算超長（邊界：<= 不是 <）', () => {
    const msg = `fix: something\n\n${'x'.repeat(LIMIT)}`
    withTempMessage(msg, (f) => {
      const { code, out } = runScanner(['--commit-msg', f])
      assert.equal(code, 0, `剛好到上限應通過，實得 exit ${code}\n${out}`)
      assert.doesNotMatch(out, /超過 200000 上限/)
    })
  })

  test('超長但完全乾淨的行也擋——無法完整掃描時不賭它乾淨', () => {
    const msg = `fix: something\n\n${'x'.repeat(LIMIT + 1)}`
    withTempMessage(msg, (f) => {
      const { code } = runScanner(['--commit-msg', f])
      assert.equal(code, 1)
    })
  })

  // ⚠️ 長度必須出現在**沒被遮蔽**的地方。report() 會把 `matched` 遮成星號
  // （為了不回顯個資），實作時我一度把長度放在那裡，結果訊息只剩一排 `*`，
  // 看的人完全不知道這行多長——等於報告了但沒說出唯一有用的資訊。
  test('報告要看得出這行多長，且長度沒被遮蔽掉', () => {
    const msg = `fix: something\n\n${'x'.repeat(LIMIT + 5)}`
    withTempMessage(msg, (f) => {
      const { out } = runScanner(['--commit-msg', f])
      assert.match(out, new RegExp(`單行 ${LIMIT + 5} 字元`))
    })
  })
})
