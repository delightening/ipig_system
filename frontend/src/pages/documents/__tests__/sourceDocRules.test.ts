import { describe, expect, it } from 'vitest'
import { carriesSourceDoc, sourceDocIdForPayload } from '../sourceDocRules'

describe('sourceDocIdForPayload', () => {
  const PO_ID = '00000000-0000-0000-0000-0000000000aa'

  it('入庫單保留來源採購單', () => {
    expect(sourceDocIdForPayload('GRN', PO_ID)).toBe(PO_ID)
  })

  it('入庫單的空字串／空白視為沒有來源', () => {
    expect(sourceDocIdForPayload('GRN', '')).toBeNull()
    expect(sourceDocIdForPayload('GRN', '   ')).toBeNull()
    expect(sourceDocIdForPayload('GRN', undefined)).toBeNull()
  })

  // #189 的重現路徑：先選入庫單挑了來源採購單，再切成其他類型，殘留值不得送出
  it.each(['PO', 'PR', 'SO', 'TR', 'ADJ', 'STK'])('%s 即使表單殘留來源 id 也送 null', (type) => {
    expect(sourceDocIdForPayload(type, PO_ID)).toBeNull()
  })

  it('類型未選時送 null', () => {
    expect(sourceDocIdForPayload(undefined, PO_ID)).toBeNull()
    expect(sourceDocIdForPayload('', PO_ID)).toBeNull()
  })
})

describe('carriesSourceDoc', () => {
  it('只有 GRN 帶來源單據', () => {
    expect(carriesSourceDoc('GRN')).toBe(true)
    expect(carriesSourceDoc('PR')).toBe(false)
  })
})
