/** 動物預約與試驗規劃頁共用格式化 helper。 */

/** YYYY-MM-DD → YYYY/M/D（清單顯示用）。 */
export function fmtDate(d: string | null): string {
  if (!d) return '—'
  const p = d.split('-')
  return p.length === 3 ? `${p[0]}/${Number(p[1])}/${Number(p[2])}` : d
}

/** YYYY-MM-DD → M/D（體重量測日期簡短顯示）。 */
export function fmtMd(d: string | null): string {
  if (!d) return ''
  const p = d.split('-')
  return p.length === 3 ? `${Number(p[1])}/${Number(p[2])}` : d
}
