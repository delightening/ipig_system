// CI 鎖：前端 canonical 選項 key 必須與 print-pdf adapter 的比對 key 一致。
// 防止再發生 #608 那種「前端改了 key、Python adapter 沒同步 → AUP PDF 漏勾」漂移。
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, resolve } from 'node:path'
import { describe, it, expect } from 'vitest'
import { PROJECT_TYPE_KEYS, PROJECT_CATEGORY_KEYS } from './aupProjectOptions'

const here = dirname(fileURLToPath(import.meta.url))
// constants → lib → src → frontend → <repo root>
const adapterPath = resolve(here, '../../../../services/print-pdf/adapters/aup_protocol.py')

/** 抓 ProjectClassification 區塊內 `"<key>" in <varName>` 的 key 集合。 */
function pyKeys(src: string, varName: string): Set<string> {
  // 容單/雙引號（防 ruff 等格式化工具把 " 改成 '）
  const re = new RegExp(`["'](\\w+)["']\\s+in\\s+${varName}\\b`, 'g')
  const out = new Set<string>()
  let m: RegExpExecArray | null
  while ((m = re.exec(src)) !== null) out.add(m[1])
  return out
}

describe('AUP project options ↔ print-pdf adapter parity', () => {
  const src = readFileSync(adapterPath, 'utf8')

  it('project_type keys match python adapter (proj_types)', () => {
    expect(pyKeys(src, 'proj_types')).toEqual(new Set<string>(PROJECT_TYPE_KEYS))
  })

  it('project_category keys match python adapter (proj_cats)', () => {
    expect(pyKeys(src, 'proj_cats')).toEqual(new Set<string>(PROJECT_CATEGORY_KEYS))
  })
})
