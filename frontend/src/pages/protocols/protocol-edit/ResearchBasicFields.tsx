import { useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import { useAuthStore } from '@/stores/auth'
import {
  PROJECT_TYPE_KEYS,
  PROJECT_CATEGORY_KEYS,
  PROJECT_TYPE_OTHER,
  PROJECT_CATEGORY_OTHER,
} from '@/lib/constants/aupProjectOptions'
import {
  fieldVisibleForVersion,
  type ProtocolFormVersion,
} from '@/lib/constants/protocolVersionManifests'
import type { ProtocolWorkingContent } from '@/types/protocol'

/**
 * 研究資料的非 PI 欄位（GLP / 計畫類型種類 / 資金來源 / 委託單位 / 試驗機構）。
 * 受控元件：`basic` 讀值、`onUpdate(path, value)` 寫值（path 同 updateWorkingContent 的 'basic' 段）。
 * 供 SectionBasic（編輯頁）與 ImportApprovedProtocolPage（匯入頁）共用，避免重複（R64-5c C2）。
 * `disabled` 為 true 時整段唯讀（編輯頁鎖定匯入計劃用）。
 */
export function ResearchBasicFields({
  basic,
  onUpdate,
  disabled,
  piOverride,
  formVersion,
}: {
  basic: ProtocolWorkingContent['basic']
  onUpdate: (path: string, value: unknown) => void
  disabled?: boolean
  /** 「同計畫主持人」帶入來源；匯入頁 PI 尚未寫入 basic.pi，由上層傳入即時 PI 值。
   *  未提供時 fallback 至 basic.pi（編輯頁）。 */
  piOverride?: { name: string; phone: string; email: string }
  /** 版本名冊：據此 gate 版本相依欄位（註冊機關/類型其他/委託地址/試驗單位…） */
  formVersion: ProtocolFormVersion
}) {
  const { t } = useTranslation()

  // 「同計畫主持人」帶入來源（匯入頁用 override，編輯頁用 basic.pi；含分機）
  const piSource = piOverride
    ? { ...piOverride, phone_ext: '' }
    : {
        name: basic.pi?.name ?? '',
        phone: basic.pi?.phone ?? '',
        email: basic.pi?.email ?? '',
        phone_ext: basic.pi?.phone_ext ?? '',
      }
  const contactSameAsPi = basic.sponsor?.contact_same_as_pi === true

  // 勾選「同計畫主持人」後，PI 改動即同步到聯絡人（live-sync）。
  // 用 latest-ref 持有 onUpdate（上層為 inline arrow，非穩定），effect 依賴只放 piSource 原始字串值，
  // 避免把不穩定 onUpdate 放進 deps 造成迴圈，同時不觸發 exhaustive-deps 告警。
  const onUpdateRef = useRef(onUpdate)
  onUpdateRef.current = onUpdate
  useEffect(() => {
    if (!contactSameAsPi) return
    onUpdateRef.current('sponsor.contact_person', piSource.name)
    onUpdateRef.current('sponsor.contact_phone', piSource.phone)
    onUpdateRef.current('sponsor.contact_phone_ext', piSource.phone_ext)
    onUpdateRef.current('sponsor.contact_email', piSource.email)
  }, [contactSameAsPi, piSource.name, piSource.phone, piSource.phone_ext, piSource.email])

  // 勾選/取消僅切旗標；帶入由上方 effect 處理（勾選後 + PI 變動皆同步）。
  const handleContactSameAsPi = (checked: boolean) => {
    onUpdate('sponsor.contact_same_as_pi', checked)
  }
  // 角色鍵與 ProtocolEditPage 對齊（含 SYSTEM_ADMIN），避免僅有 SYSTEM_ADMIN 者被鎖出設施欄位
  const isAdminOrIacuc =
    useAuthStore.getState().hasRole('admin') ||
    useAuthStore.getState().hasRole('SYSTEM_ADMIN') ||
    useAuthStore.getState().hasRole('IACUC_STAFF')

  const toggleArray = (path: string, current: string[], option: string, checked: boolean) =>
    onUpdate(path, checked ? [...current, option] : current.filter((s) => s !== option))

  return (
    <fieldset disabled={disabled} className="space-y-6 border-0 p-0 m-0 disabled:opacity-70">
      {/* GLP / non-GLP（單選） */}
      <div className="space-y-2">
        <Label>{t('aup.basic.glpAttribute')} *</Label>
        <div className="flex gap-4 pt-2">
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              name="is_glp"
              checked={basic.is_glp === true}
              onChange={() => onUpdate('is_glp', true)}
              className="w-4 h-4 text-status-purple-text"
            />
            <span className="text-sm">{t('aup.basic.glpCompliant')}</span>
          </label>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="radio"
              name="is_glp"
              checked={basic.is_glp === false}
              onChange={() => onUpdate('is_glp', false)}
              className="w-4 h-4 text-status-purple-text"
            />
            <span className="text-sm">{t('aup.basic.glpNonCompliant')}</span>
          </label>
        </div>
      </div>
      {basic.is_glp && fieldVisibleForVersion('basic.registrationAuthorities', formVersion) && (
        <div className="space-y-4">
          <Label className="text-base font-semibold">{t('aup.basic.registrationAuthorities')} *</Label>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 pl-4 text-sm">
            {['FDA', 'CE', 'TFDA', 'CFDA', 'other'].map((option) => (
              <div key={option} className="flex items-center space-x-3">
                <Checkbox
                  id={`reg_auth_${option}`}
                  checked={basic.registration_authorities.includes(option)}
                  onCheckedChange={(checked) =>
                    toggleArray('registration_authorities', basic.registration_authorities || [], option, !!checked)
                  }
                />
                <Label htmlFor={`reg_auth_${option}`} className="font-normal cursor-pointer">
                  {t(`aup.basic.registrationAuthorityOptions.${option}`)}
                </Label>
              </div>
            ))}
          </div>
          {basic.registration_authorities.includes('other') && (
            <div className="pl-4 pt-2 lg:w-1/2">
              <Input
                placeholder={t('aup.basic.specifyOther')}
                value={basic.registration_authority_other || ''}
                onChange={(e) => onUpdate('registration_authority_other', e.target.value)}
              />
            </div>
          )}
        </div>
      )}

      {/* 類型 / 種類（皆可複選） */}
      <div className="grid gap-4 md:grid-cols-2">
        <div className="space-y-2">
          <Label>{t('aup.basic.projectType')} * (複選)</Label>
          <div className="grid gap-3 pl-4 text-sm">
            {PROJECT_TYPE_KEYS.filter((v) => v !== PROJECT_TYPE_OTHER || fieldVisibleForVersion('basic.projectTypeOther', formVersion)).map((v) => (
              <div key={v} className="flex items-center space-x-3">
                <Checkbox
                  id={`project_type_${v}`}
                  checked={(basic.project_type || []).includes(v)}
                  onCheckedChange={(checked) =>
                    toggleArray('project_type', basic.project_type || [], v, !!checked)
                  }
                />
                <Label htmlFor={`project_type_${v}`} className="font-normal cursor-pointer">
                  {t(`aup.projectTypes.${v}`)}
                </Label>
              </div>
            ))}
          </div>
          {(basic.project_type || []).includes(PROJECT_TYPE_OTHER) && (
            <Input
              className="mt-2"
              placeholder={t('aup.basic.specifyOther')}
              value={basic.project_type_other || ''}
              onChange={(e) => onUpdate('project_type_other', e.target.value)}
            />
          )}
        </div>
        <div className="space-y-2">
          <Label>{t('aup.basic.projectCategory')} * (複選)</Label>
          <div className="grid gap-3 pl-4 text-sm sm:grid-cols-2 sm:grid-rows-6 sm:grid-flow-col">
            {PROJECT_CATEGORY_KEYS.map((v) => (
              <div key={v} className="flex items-center space-x-3">
                <Checkbox
                  id={`project_category_${v}`}
                  checked={(basic.project_category || []).includes(v)}
                  onCheckedChange={(checked) =>
                    toggleArray('project_category', basic.project_category || [], v, !!checked)
                  }
                />
                <Label htmlFor={`project_category_${v}`} className="font-normal cursor-pointer">
                  {t(`aup.projectCategories.${v}`)}
                </Label>
              </div>
            ))}
          </div>
          {(basic.project_category || []).includes(PROJECT_CATEGORY_OTHER) && (
            <Input
              className="mt-2"
              placeholder={t('aup.basic.specifyOther')}
              value={basic.project_category_other || ''}
              onChange={(e) => onUpdate('project_category_other', e.target.value)}
            />
          )}
        </div>
      </div>

      <div className="h-px bg-border my-4" />

      {/* 資金來源 */}
      <div className="space-y-4">
        <Label className="text-base font-semibold">{t('aup.basic.fundingSources')} (複選)</Label>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3 pl-4 text-sm">
          {['moa', 'mohw', 'nstc', 'moe', 'env', 'other'].map((option) => (
            <div key={option} className="flex items-center space-x-3">
              <Checkbox
                id={`funding_${option}`}
                checked={basic.funding_sources.includes(option)}
                onCheckedChange={(checked) =>
                  toggleArray('funding_sources', basic.funding_sources || [], option, !!checked)
                }
              />
              <Label htmlFor={`funding_${option}`} className="font-normal cursor-pointer">
                {t(`aup.basic.fundingSourceOptions.${option}`)}
              </Label>
            </div>
          ))}
        </div>
        {basic.funding_sources.includes('other') && (
          <div className="pl-4 pt-2 lg:w-1/2">
            <Input
              placeholder={t('aup.basic.specifyOther')}
              value={basic.funding_other || ''}
              onChange={(e) => onUpdate('funding_other', e.target.value)}
            />
          </div>
        )}
      </div>

      <div className="h-px bg-border my-4" />

      {/* 委託單位 */}
      <div className="space-y-4">
        <h3 className="font-semibold">{t('aup.basic.sponsor')}</h3>
        {/* 同計畫主持人：聯絡人 = PI 時直接繼承（姓名/電話/email 帶入並唯讀） */}
        <label className="flex items-center gap-2 text-sm cursor-pointer w-fit">
          <Checkbox
            checked={contactSameAsPi}
            onCheckedChange={(checked) => handleContactSameAsPi(!!checked)}
          />
          <span>{t('aup.basic.contactSameAsPi')}</span>
        </label>
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-2">
            <Label>{t('aup.basic.organizationName')} *</Label>
            <Input value={basic.sponsor.name} onChange={(e) => onUpdate('sponsor.name', e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label>{t('aup.basic.contactPerson')} *</Label>
            <Input
              value={basic.sponsor.contact_person}
              onChange={(e) => onUpdate('sponsor.contact_person', e.target.value)}
              placeholder={t('aup.basic.contactPersonPlaceholder')}
              disabled={contactSameAsPi}
            />
          </div>
          <div className="space-y-2">
            <Label>{t('aup.basic.contactPhone')} *</Label>
            <Input
              value={basic.sponsor.contact_phone}
              onChange={(e) => onUpdate('sponsor.contact_phone', e.target.value)}
              disabled={contactSameAsPi}
            />
          </div>
          <div className="space-y-2">
            <Label>{t('aup.basic.contactEmail')} *</Label>
            <Input
              value={basic.sponsor.contact_email}
              onChange={(e) => onUpdate('sponsor.contact_email', e.target.value)}
              disabled={contactSameAsPi}
            />
            <p className="text-xs text-muted-foreground">{t('aup.basic.contactEmailMultiHint')}</p>
          </div>
          {fieldVisibleForVersion('basic.sponsorAddress', formVersion) && (
            <div className="space-y-2">
              <Label>{t('aup.basic.sponsorAddress')}</Label>
              <Input
                value={basic.sponsor.address || ''}
                onChange={(e) => onUpdate('sponsor.address', e.target.value)}
              />
            </div>
          )}
        </div>
      </div>

      {/* 試驗單位 Test Unit 多站塊（C/D 版孤兒；E/F 已移除） */}
      {fieldVisibleForVersion('basic.testUnit', formVersion) && (
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="font-semibold">{t('aup.basic.testUnitTitle')}</h3>
            <button
              type="button"
              className="text-sm text-status-purple-text hover:underline"
              onClick={() =>
                onUpdate('test_units', [
                  ...(basic.test_units || []),
                  { stage: '', name: '', address: '', pi_name: '' },
                ])
              }
            >
              + {t('aup.basic.testUnitAdd')}
            </button>
          </div>
          {(basic.test_units || []).map((unit, idx) => {
            const patch = (field: 'stage' | 'name' | 'address' | 'pi_name', value: string) => {
              const next = [...(basic.test_units || [])]
              next[idx] = { ...next[idx], [field]: value }
              onUpdate('test_units', next)
            }
            return (
              <div key={idx} className="grid gap-3 md:grid-cols-2 border rounded-md p-3 relative">
                <div className="space-y-1">
                  <Label>{t('aup.basic.testUnitStage')}</Label>
                  <Input value={unit.stage || ''} onChange={(e) => patch('stage', e.target.value)} />
                </div>
                <div className="space-y-1">
                  <Label>{t('aup.basic.testUnitName')}</Label>
                  <Input value={unit.name || ''} onChange={(e) => patch('name', e.target.value)} />
                </div>
                <div className="space-y-1">
                  <Label>{t('aup.basic.testUnitAddress')}</Label>
                  <Input value={unit.address || ''} onChange={(e) => patch('address', e.target.value)} />
                </div>
                <div className="space-y-1">
                  <Label>{t('aup.basic.testUnitPi')}</Label>
                  <Input value={unit.pi_name || ''} onChange={(e) => patch('pi_name', e.target.value)} />
                </div>
                <button
                  type="button"
                  className="absolute top-1 right-2 text-destructive"
                  aria-label={t('common.delete')}
                  onClick={() => onUpdate('test_units', (basic.test_units || []).filter((_, i) => i !== idx))}
                >
                  ×
                </button>
              </div>
            )
          })}
        </div>
      )}

      {/* 試驗機構與設施 */}
      <div className="space-y-4">
        <h3 className="font-semibold">{t('aup.basic.facilityAndLocation')}</h3>
        <div className="grid gap-4 md:grid-cols-2">
          <div className="space-y-2">
            <Label>{t('aup.basic.facilityName')} *</Label>
            <Input
              value={basic.facility.title}
              onChange={(e) => onUpdate('facility.title', e.target.value)}
              disabled={!isAdminOrIacuc}
            />
          </div>
          <div className="space-y-2">
            <Label>{t('aup.basic.location')} *</Label>
            <Input
              value={basic.housing_location}
              onChange={(e) => onUpdate('housing_location', e.target.value)}
              disabled={!isAdminOrIacuc}
            />
          </div>
        </div>
      </div>
    </fieldset>
  )
}
