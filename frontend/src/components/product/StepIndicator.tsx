import * as React from 'react'
import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'

export interface Step {
  id: string
  label: string
  description?: string
}

interface StepIndicatorProps {
  steps: Step[]
  currentStep: number
  completedSteps?: number[]
  onStepClick?: (stepIndex: number) => void
  allowNavigation?: boolean
  className?: string
}

export function StepIndicator({
  steps,
  currentStep,
  completedSteps = [],
  onStepClick,
  allowNavigation = false,
  className,
}: StepIndicatorProps) {
  return (
    <div className={cn("flex items-center justify-center", className)}>
      {steps.map((step, index) => {
        const isCompleted = completedSteps.includes(index)
        const isCurrent = currentStep === index
        const isClickable = allowNavigation && (isCompleted || index < currentStep)

        return (
          <React.Fragment key={step.id}>
            {/* Step Circle + Label */}
            <div className="flex flex-col items-center">
              <button
                type="button"
                onClick={() => isClickable && onStepClick?.(index)}
                disabled={!isClickable}
                className={cn(
                  "step-circle relative w-10 h-10 rounded-full flex items-center justify-center font-semibold text-sm transition-all",
                  isCompleted
                    ? "bg-success text-success-foreground"
                    : isCurrent
                    ? "bg-primary text-primary-foreground ring-4 ring-primary/20"
                    : "bg-slate-100 dark:bg-slate-800 text-slate-400 border-2 border-slate-200 dark:border-slate-700",
                  isClickable && "cursor-pointer hover:scale-105",
                  !isClickable && "cursor-default"
                )}
              >
                {isCompleted ? (
                  <Check className="w-5 h-5" />
                ) : (
                  <span>{index + 1}</span>
                )}
              </button>
              
              {/* Label */}
              <span
                className={cn(
                  "mt-2 text-sm font-medium transition-colors",
                  isCurrent
                    ? "text-primary"
                    : isCompleted
                    ? "text-slate-700 dark:text-slate-300"
                    : "text-slate-400"
                )}
              >
                {step.label}
              </span>
              
              {/* Description */}
              {step.description && (
                <span className="text-xs text-slate-400 mt-0.5 hidden sm:block">
                  {step.description}
                </span>
              )}
            </div>

            {/* Connector Line */}
            {index < steps.length - 1 && (
              <div
                className={cn(
                  "step-line w-16 sm:w-24 h-0.5 mx-2 transition-colors",
                  index < currentStep || completedSteps.includes(index)
                    ? "bg-success"
                    : "bg-slate-200 dark:bg-slate-700"
                )}
              />
            )}
          </React.Fragment>
        )
      })}
    </div>
  )
}

// Compact Step Indicator for mobile
interface CompactStepIndicatorProps {
  steps: Step[]
  currentStep: number
  className?: string
}

export function CompactStepIndicator({
  steps,
  currentStep,
  className,
}: CompactStepIndicatorProps) {
  return (
    <div className={cn("flex items-center justify-between", className)}>
      <span className="text-sm font-medium text-slate-700 dark:text-slate-300">
        步驟 {currentStep + 1} / {steps.length}
      </span>
      <span className="text-sm text-slate-500">
        {steps[currentStep]?.label}
      </span>
    </div>
  )
}
