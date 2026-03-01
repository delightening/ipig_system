import { useState } from 'react'
import { ChevronDown, ChevronUp } from 'lucide-react'
import { Checkbox } from '@/components/ui/checkbox'
import { Input } from '@/components/ui/input'

/** 可折疊區塊組件 */
export function CollapsibleSection({
  title,
  children,
  defaultOpen = true,
}: {
  title: string
  children: React.ReactNode
  defaultOpen?: boolean
}) {
  const [isOpen, setIsOpen] = useState(defaultOpen)

  return (
    <div className="border rounded-lg">
      <button
        type="button"
        className="flex items-center justify-between w-full px-4 py-3 text-left font-medium bg-slate-50 hover:bg-slate-100 rounded-t-lg"
        onClick={() => setIsOpen(!isOpen)}
      >
        {title}
        {isOpen ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
      </button>
      {isOpen && <div className="p-4 space-y-4">{children}</div>}
    </div>
  )
}

/** 誘導麻醉藥物項目 */
export interface AnesthesiaDrug {
  name: string
  dose: string
  enabled: boolean
}

/** 藥物勾選輸入組件 */
export function DrugCheckInput({
  label,
  drug,
  onChange,
}: {
  label: string
  drug: AnesthesiaDrug
  onChange: (drug: AnesthesiaDrug) => void
}) {
  return (
    <div className="flex items-center gap-3">
      <Checkbox
        label={label}
        checked={drug.enabled}
        onCheckedChange={(checked) => onChange({ ...drug, enabled: checked === true })}
      />
      {drug.enabled && (
        <Input
          className="w-32"
          placeholder="劑量"
          value={drug.dose}
          onChange={(e) => onChange({ ...drug, dose: e.target.value })}
        />
      )}
    </div>
  )
}
