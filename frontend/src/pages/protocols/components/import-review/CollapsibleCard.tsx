import { useState, type ReactNode } from 'react'
import { ChevronDown, ChevronRight } from 'lucide-react'

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

/**
 * 補登作業可收合區塊：標題列可點擊收合/展開（執秘意見 / 獸醫師評比 / 委員意見）。
 * 收合時隱藏內容，方便逐段補登時聚焦單一區塊。
 */
export function CollapsibleCard({
  title,
  defaultOpen = true,
  children,
}: {
  title: string
  defaultOpen?: boolean
  children: ReactNode
}) {
  const [open, setOpen] = useState(defaultOpen)
  return (
    <Card>
      <CardHeader className="py-4">
        <button
          type="button"
          onClick={() => setOpen((v) => !v)}
          aria-expanded={open}
          className="flex w-full items-center justify-between text-left"
        >
          <CardTitle className="text-base">{title}</CardTitle>
          {open ? (
            <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
          ) : (
            <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
          )}
        </button>
      </CardHeader>
      {open && <CardContent className="space-y-3">{children}</CardContent>}
    </Card>
  )
}
