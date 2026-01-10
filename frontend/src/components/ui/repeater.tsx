import * as React from "react"
import { cn } from "@/lib/utils"
import { Plus, Trash2 } from "lucide-react"
import { Button } from "./button"

export interface RepeaterProps<T> {
  value: T[]
  onChange: (value: T[]) => void
  renderItem: (item: T, index: number, onChange: (item: T) => void) => React.ReactNode
  defaultItem: () => T
  addLabel?: string
  maxItems?: number
  minItems?: number
  className?: string
  disabled?: boolean
}

function Repeater<T>({
  value,
  onChange,
  renderItem,
  defaultItem,
  addLabel = "新增項目",
  maxItems = 20,
  minItems = 0,
  className,
  disabled = false,
}: RepeaterProps<T>) {
  const handleAdd = () => {
    if (value.length < maxItems && !disabled) {
      onChange([...value, defaultItem()])
    }
  }

  const handleRemove = (index: number) => {
    if (value.length > minItems && !disabled) {
      onChange(value.filter((_, i) => i !== index))
    }
  }

  const handleItemChange = (index: number, item: T) => {
    if (!disabled) {
      const newValue = [...value]
      newValue[index] = item
      onChange(newValue)
    }
  }

  return (
    <div className={cn("space-y-3", className)}>
      {value.map((item, index) => (
        <div key={index} className="flex items-start gap-2">
          <div className="flex-1">
            {renderItem(item, index, (newItem) => handleItemChange(index, newItem))}
          </div>
          {value.length > minItems && !disabled && (
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="h-10 w-10 shrink-0 text-red-500 hover:text-red-600 hover:bg-red-50"
              onClick={() => handleRemove(index)}
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          )}
        </div>
      ))}
      
      {value.length < maxItems && !disabled && (
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={handleAdd}
          className="w-full border-dashed"
        >
          <Plus className="h-4 w-4 mr-2" />
          {addLabel}
        </Button>
      )}
    </div>
  )
}

Repeater.displayName = "Repeater"

export { Repeater }
