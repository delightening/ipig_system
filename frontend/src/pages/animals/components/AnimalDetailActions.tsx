import React from 'react'

import { AnimalStatus } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { GuestHide } from '@/components/ui/guest-hide'
import { Can } from '@/components/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
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
    <GuestHide>
      <div className="flex gap-2">
        {status === 'in_experiment' && (
          <>
            {/* \u7DCA\u6025\u7D66\u85E5\u7368\u7ACB\u65BC\u4E00\u822C record \u6B0A\u9650\uFF08\u5F8C\u7AEF observation.rs:105 \u5728 is_emergency \u6642
                \u53E6\u5916\u8981\u6C42 animal.record.emergency\uFF09\uFF0C\u6545\u9019\u88E1\u4E5F\u7528\u8A72\u78BC\u800C\u975E record.create\u3002 */}
            <Can permission={PERMISSIONS.ANIMAL_RECORD_EMERGENCY}>
              <Button
                variant="outline"
                className="border-status-warning-border text-status-warning-text hover:bg-status-warning-bg"
                onClick={onEmergencyMedication}
              >
                <AlertTriangle className="h-4 w-4 mr-2" />
                {'\u7DCA\u6025\u7D66\u85E5'}
              </Button>
            </Can>
            {/* \u5F8C\u7AEF euthanasia.rs \u7684 create_order \u6AA2 animal.euthanasia.create\uFF08\u5DF2\u6388\u4E88 VET\uFF0C
                \u4E26\u62FF\u6389 has_role(ROLE_VET) fallback\uFF09\uFF0C\u524D\u7AEF\u6539\u7528\u540C\u4E00\u78BC\u3002 */}
            <Can permission={PERMISSIONS.ANIMAL_EUTHANASIA_CREATE}>
              <Button
                variant="outline"
                className="border-destructive text-destructive hover:bg-status-error-bg"
                onClick={onEuthanasiaOrder}
              >
                <AlertOctagon className="h-4 w-4 mr-2" />
                {'\u958B\u7ACB\u5B89\u6A02\u6B7B\u55AE'}
              </Button>
            </Can>
          </>
        )}
        {/* \u767B\u8A18\u731D\u6B7B\u8D70 sudden_death.rs:36 \u7684 animal.record.create */}
        <Can permission={PERMISSIONS.ANIMAL_RECORD_CREATE}>
          <Button
            variant="outline"
            className="border-destructive text-destructive hover:bg-status-error-bg"
            onClick={onSuddenDeath}
          >
            <Zap className="h-4 w-4 mr-2" />
            {'\u767B\u8A18\u731D\u6B7B'}
          </Button>
        </Can>
      </div>
    </GuestHide>
  )
}
