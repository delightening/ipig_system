import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { UNITS } from '../constants'
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

const TYPE_LABELS: Record<string, string> = {
  outer: '外',
  inner: '內',
  base: '基礎',
}

const TYPE_LABELS_LONG: Record<string, string> = {
  outer: '外層',
  inner: '內層',
  base: '基礎',
}

export function PackagingUnitSelect({
  value,
  onChange,
  formData,
  disabled = false,
  includeBase = false,
  className,
}: PackagingUnitSelectProps) {
  const units = buildAvailableUnits(formData, includeBase)

  return (
    <Select value={value} onValueChange={onChange} disabled={disabled}>
      <SelectTrigger className={className}>
        <SelectValue placeholder="選擇單位" />
      </SelectTrigger>
      <SelectContent>
        {units.length === 0
          ? UNITS.base.map((unit) => (
              <SelectItem key={unit.code} value={unit.name}>
                {unit.name} ({unit.code})
              </SelectItem>
            ))
          : units.map((unit) => (
              <SelectItem key={unit.code} value={unit.name}>
                {unit.name} ({unit.code}) - {includeBase ? `${TYPE_LABELS_LONG[unit.type]}包裝` : TYPE_LABELS[unit.type]}
              </SelectItem>
            ))
        }
      </SelectContent>
    </Select>
  )
}
