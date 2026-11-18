import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Input } from '@/components/ui/input'
import { FormField } from '@/components/ui/form-field'
import { formatArea, penAreaM2, type PenSizeError } from '../utils/penSize'

interface PenSizeFieldsProps {
  length: string
  width: string
  onLengthChange: (value: string) => void
  onWidthChange: (value: string) => void
  /** 由 validatePenSize 算出；null 表示合法（含兩格皆空） */
  error: PenSizeError | null
  /** 兩格皆為必填（套用到整個區時用）；預設選填 */
  required?: boolean
}

/** 欄位長、寬（公尺）輸入：一起填或一起留空，下方即時顯示換算後的面積 */
export function PenSizeFields({
  length, width, onLengthChange, onWidthChange, error, required = false,
}: PenSizeFieldsProps) {
  const { t } = useTranslation()
  const area = error === null ? penAreaM2(length, width) : null
  const lengthId = useId()
  const widthId = useId()
  // 必填模式（套用到整個區）不能說「或一起留空」，提示與單邊錯誤都改用必填措辭
  const errorKey = required && error === 'bothOrNone' ? 'bothRequired' : error

  return (
    <div className="space-y-1">
      <div className="grid grid-cols-2 gap-3">
        <FormField label={t('admin.penTab.fieldLength')} htmlFor={lengthId} required={required}>
          <Input
            id={lengthId}
            inputMode="decimal"
            value={length}
            onChange={e => onLengthChange(e.target.value)}
            aria-invalid={error !== null}
            placeholder="2.40"
          />
        </FormField>
        <FormField label={t('admin.penTab.fieldWidth')} htmlFor={widthId} required={required}>
          <Input
            id={widthId}
            inputMode="decimal"
            value={width}
            onChange={e => onWidthChange(e.target.value)}
            aria-invalid={error !== null}
            placeholder="3.00"
          />
        </FormField>
      </div>
      {errorKey !== null ? (
        <p className="text-sm text-destructive" role="alert">{t(`admin.penTab.size_${errorKey}`)}</p>
      ) : area !== null ? (
        <p className="text-sm text-muted-foreground tabular-nums">
          {t('admin.penTab.computedArea', { area: formatArea(area) })}
        </p>
      ) : (
        <p className="text-xs text-muted-foreground">
          {t(required ? 'admin.penTab.sizeHintRequired' : 'admin.penTab.sizeHint')}
        </p>
      )}
    </div>
  )
}
