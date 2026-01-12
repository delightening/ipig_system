import * as React from "react"
import { cn } from "@/lib/utils"
import { Check } from "lucide-react"

export interface CheckboxProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'onChange'> {
  label?: string
  onCheckedChange?: (checked: boolean) => void
}

const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, label, id, onCheckedChange, ...props }, ref) => {
    const inputId = id || `checkbox-${React.useId()}`

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      onCheckedChange?.(e.target.checked)
    }

    return (
      <div className="flex items-center space-x-2">
        <div className="relative">
          <input
            type="checkbox"
            id={inputId}
            className={cn(
              "peer h-4 w-4 shrink-0 rounded-sm border border-slate-300 appearance-none",
              "checked:bg-purple-600 checked:border-purple-600",
              "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2",
              "disabled:cursor-not-allowed disabled:opacity-50",
              className
            )}
            ref={ref}
            onChange={handleChange}
            {...props}
          />
          <Check className="absolute top-0 left-0 h-4 w-4 text-white pointer-events-none opacity-0 peer-checked:opacity-100" />
        </div>
        {label && (
          <label
            htmlFor={inputId}
            className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
          >
            {label}
          </label>
        )}
      </div>
    )
  }
)
Checkbox.displayName = "Checkbox"

export { Checkbox }
