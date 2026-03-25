import React from 'react'

import { AnimalStatus } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { AlertTriangle, AlertOctagon, Zap } from 'lucide-react'

interface AnimalDetailActionsProps {
  status: AnimalStatus
  onEmergencyMedication: () => void
  onEuthanasiaOrder: () => void
  onSuddenDeath: () => void
}

export function AnimalDetailActions({
  status,
  onEmergencyMedication,
  onEuthanasiaOrder,
  onSuddenDeath,
}: AnimalDetailActionsProps) {
  if (status !== 'in_experiment' && status !== 'completed') return null

  return (
    <div className="flex gap-2">
      {status === 'in_experiment' && (
        <>
          <Button
            variant="outline"
            className="border-status-warning-border text-status-warning-text hover:bg-status-warning-bg"
            onClick={onEmergencyMedication}
          >
            <AlertTriangle className="h-4 w-4 mr-2" />
            {'\u7DCA\u6025\u7D66\u85E5'}
          </Button>
          <Button
            variant="outline"
            className="border-destructive text-destructive hover:bg-status-error-bg"
            onClick={onEuthanasiaOrder}
          >
            <AlertOctagon className="h-4 w-4 mr-2" />
            {'\u958B\u7ACB\u5B89\u6A02\u6B7B\u55AE'}
          </Button>
        </>
      )}
      <Button
        variant="outline"
        className="border-destructive text-destructive hover:bg-status-error-bg"
        onClick={onSuddenDeath}
      >
        <Zap className="h-4 w-4 mr-2" />
        {'\u767B\u8A18\u731D\u6B7B'}
      </Button>
    </div>
  )
}
