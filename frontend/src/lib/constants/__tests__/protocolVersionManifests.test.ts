import { describe, it, expect } from 'vitest'

import {
  fieldVisibleForVersion,
  normalizeFormVersion,
  LATEST_PROTOCOL_FORM_VERSION,
  type VersionGatedFieldKey,
} from '../protocolVersionManifests'

describe('版本名冊 gating（fieldVisibleForVersion）', () => {
  it('F 專有欄位：僅 F 顯示，C/D/E 隱藏（版本忠實）', () => {
    expect(fieldVisibleForVersion('design.painDistressSigns', 'F')).toBe(true)
    expect(fieldVisibleForVersion('design.painDistressSigns', 'C')).toBe(false)
    expect(fieldVisibleForVersion('design.painDistressSigns', 'E')).toBe(false)
    expect(fieldVisibleForVersion('guidelines.databases', 'F')).toBe(true)
    expect(fieldVisibleForVersion('guidelines.databases', 'D')).toBe(false)
    expect(fieldVisibleForVersion('purpose.reductionSpecialCare', 'F')).toBe(true)
    expect(fieldVisibleForVersion('purpose.reductionSpecialCare', 'C')).toBe(false)
  })

  it('C/D 孤兒欄位：僅 C/D 顯示，E/F 隱藏', () => {
    for (const key of ['basic.testUnit', 'basic.sponsorAddress', 'resultsAnalysis', 'documentArchiving', 'design.housingEnvironmentSop', 'items.testItemExtra'] as VersionGatedFieldKey[]) {
      expect(fieldVisibleForVersion(key, 'C')).toBe(true)
      expect(fieldVisibleForVersion(key, 'D')).toBe(true)
      expect(fieldVisibleForVersion(key, 'E')).toBe(false)
      expect(fieldVisibleForVersion(key, 'F')).toBe(false)
    }
  })

  it('C/D/E 欄位（F 移除）：F 隱藏、其餘顯示', () => {
    for (const key of ['basic.testItemType', 'basic.techCategories', 'basic.registrationAuthorities'] as VersionGatedFieldKey[]) {
      expect(fieldVisibleForVersion(key, 'C')).toBe(true)
      expect(fieldVisibleForVersion(key, 'E')).toBe(true)
      expect(fieldVisibleForVersion(key, 'F')).toBe(false)
    }
  })

  it('E 起欄位：C/D 隱藏，E/F 顯示', () => {
    for (const key of ['basic.sdSplit', 'basic.sponsorContactPerson', 'purpose.replacementPlatforms'] as VersionGatedFieldKey[]) {
      expect(fieldVisibleForVersion(key, 'C')).toBe(false)
      expect(fieldVisibleForVersion(key, 'E')).toBe(true)
      expect(fieldVisibleForVersion(key, 'F')).toBe(true)
    }
  })

  it('C=D：C 與 D 對每個 gated 欄位可見性一致（共用同一名冊）', () => {
    const keys: VersionGatedFieldKey[] = [
      'basic.testUnit', 'basic.sponsorAddress', 'resultsAnalysis', 'documentArchiving',
      'design.housingEnvironmentSop', 'items.testItemExtra', 'basic.testItemType',
      'basic.registrationAuthorities', 'purpose.replacementPlatforms', 'purpose.duplicateFourOptions',
      'design.painDistressSigns', 'guidelines.databases', 'basic.piPosition',
    ]
    for (const k of keys) {
      expect(fieldVisibleForVersion(k, 'C')).toBe(fieldVisibleForVersion(k, 'D'))
    }
  })

  it('normalizeFormVersion：未知/空 → 最新版；大小寫容錯', () => {
    expect(normalizeFormVersion(null)).toBe(LATEST_PROTOCOL_FORM_VERSION)
    expect(normalizeFormVersion(undefined)).toBe(LATEST_PROTOCOL_FORM_VERSION)
    expect(normalizeFormVersion('')).toBe(LATEST_PROTOCOL_FORM_VERSION)
    expect(normalizeFormVersion('X')).toBe(LATEST_PROTOCOL_FORM_VERSION)
    expect(normalizeFormVersion('c')).toBe('C')
    expect(normalizeFormVersion(' e ')).toBe('E')
    expect(normalizeFormVersion('F')).toBe('F')
  })
})
