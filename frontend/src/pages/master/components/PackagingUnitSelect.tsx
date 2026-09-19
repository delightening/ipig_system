import { useTranslation } from 'react-i18next'

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { UNITS, unitDisplayName } from '../constants'
import type { ProductFormData } from '../constants'

const ALL_UNITS = [...UNITS.outer, ...UNITS.inner, ...UNITS.base]

interface PackagingUnitSelectProps {
  value: string
  onChange: (value: string) => void
  formData: ProductFormData
  disabled?: boolean
  /** Whether to include base unit (true for reorder point, false for 2-layer current/safety stock) */
  includeBase?: boolean
  className?: string
}

interface UnitEntry {
  code: string
  name: string
  type: 'outer' | 'inner' | 'base'
}

function buildAvailableUnits(formData: ProductFormData, includeBase: boolean): UnitEntry[] {
  const units: UnitEntry[] = []

  if (formData.outerUnit) {
    const found = ALL_UNITS.find(u => u.name === formData.outerUnit || u.code === formData.outerUnit)
    if (found) units.push({ code: found.code, name: found.name, type: 'outer' })
  }

  if (formData.innerUnit) {
    const found = ALL_UNITS.find(u => u.name === formData.innerUnit || u.code === formData.innerUnit)
    if (found && !units.some(u => u.code === found.code)) {
      units.push({ code: found.code, name: found.name, type: 'inner' })
    }
  }

  if (includeBase && formData.baseUnit) {
    const found = ALL_UNITS.find(u => u.name === formData.baseUnit || u.code === formData.baseUnit)
    if (found && !units.some(u => u.code === found.code)) {
      units.push({ code: found.code, name: found.name, type: 'base' })
    }
  }

  return units
}

// 值是 i18n 鍵（模組頂層不存翻譯後字串），渲染時才 t()
const TYPE_LABEL_KEYS: Record<string, string> = {
  outer: 'erpMaster.packaging.typeShort.outer',
  inner: 'erpMaster.packaging.typeShort.inner',
  base: 'erpMaster.packaging.typeShort.base',
}

const TYPE_LABEL_LONG_KEYS: Record<string, string> = {
  outer: 'erpMaster.packaging.outerLayer',
  inner: 'erpMaster.packaging.innerLayer',
  base: 'erpMaster.packaging.baseLayer',
}

export function PackagingUnitSelect({
  value,
  onChange,
  formData,
  disabled = false,
  includeBase = false,
  className,
}: PackagingUnitSelectProps) {
  const { t } = useTranslation()
  const units = buildAvailableUnits(formData, includeBase)

  return (
    <Select value={value} onValueChange={onChange} disabled={disabled}>
      <SelectTrigger className={className}>
        <SelectValue placeholder={t('erpMaster.packaging.selectUnit')} />
      </SelectTrigger>
      <SelectContent>
        {units.length === 0
          ? UNITS.base.map((unit) => (
              <SelectItem key={unit.code} value={unit.name}>
                {unitDisplayName(unit.name)} ({unit.code})
              </SelectItem>
            ))
          : units.map((unit) => (
              <SelectItem key={unit.code} value={unit.name}>
                {unitDisplayName(unit.name)} ({unit.code}) - {includeBase ? t(TYPE_LABEL_LONG_KEYS[unit.type]) : t(TYPE_LABEL_KEYS[unit.type])}
              </SelectItem>
            ))
        }
      </SelectContent>
    </Select>
  )
}
