import React, { useState, useCallback } from 'react'
import { useParams, Link, useNavigate, useSearchParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { ArrowLeft, Loader2, AlertCircle, Download } from 'lucide-react'
import { useConfirmDialog } from '@/hooks/useConfirmDialog'
import { ConfirmDialog } from '@/components/ui/confirm-dialog'
import { ExportDialog } from '@/components/animal/ExportDialog'
import { EmergencyMedicationDialog } from '@/components/animal/EmergencyMedicationDialog'
import { EuthanasiaOrderDialog } from '@/components/animal/EuthanasiaOrderDialog'
import { useAuthStore } from '@/stores/auth'
import { useUIPreferences } from '@/stores/uiPreferences'

import { parseTabFromUrl } from './constants'
import type { TabType } from './constants'
import { useAnimalDetailQueries } from './hooks/useAnimalDetailQueries'
import { useAnimalDetailMutations } from './hooks/useAnimalDetailMutations'
import { AnimalHeaderCard } from './components/AnimalHeaderCard'
import { AnimalDetailActions } from './components/AnimalDetailActions'
import { TabBar, TabContent } from './components/AnimalDetailTabContent'
import { SuddenDeathDialog } from './components/SuddenDeathDialog'

export function AnimalDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [searchParams, setSearchParams] = useSearchParams()
  const { dialogState, confirm } = useConfirmDialog()
  const { t } = useTranslation()
  const animalId = id!

  const { hasRole } = useAuthStore()
  const { developerMode, toggleDeveloperMode } = useUIPreferences()

  const tabParam = searchParams.get('tab')

  const [showExportDialog, setShowExportDialog] = useState(false)
  const [showEmergencyMedicationDialog, setShowEmergencyMedicationDialog] = useState(false)
  const [showEuthanasiaOrderDialog, setShowEuthanasiaOrderDialog] = useState(false)

  // Tab state derived from URL
  const activeTab = parseTabFromUrl(tabParam, undefined)
  const setActiveTab = useCallback(
    (tab: TabType) => {
      setSearchParams((prev) => {
        const next = new URLSearchParams(prev)
        if (tab === 'timeline') next.delete('tab')
        else next.set('tab', tab)
        return next
      })
    },
    [setSearchParams],
  )

  // Queries
  const queries = useAnimalDetailQueries({
    animalId,
    activeTab,
    animalStatus: undefined,
  })

  // Re-derive activeTab with actual animal status
  const resolvedActiveTab = parseTabFromUrl(tabParam, queries.animal?.status)

  // Mutations
  const mutations = useAnimalDetailMutations(animalId)

  if (queries.animalLoading) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
      </div>
    )
  }

  if (!queries.animal) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px]">
        <AlertCircle className="h-12 w-12 text-slate-400 mb-4" />
        <p className="text-slate-500">
          {t('animalDetail.notFound', '\u627E\u4E0D\u5230\u6B64\u52D5\u7269')}
        </p>
        <Button variant="outline" className="mt-4" onClick={() => navigate('/animals')}>
          {t('animalDetail.backToList', '\u8FD4\u56DE\u5217\u8868')}
        </Button>
      </div>
    )
  }

  const { animal } = queries

  const handleSuddenDeathConfirm = async () => {
    const ok = await confirm({
      title: '\u767B\u8A18\u731D\u6B7B',
      description: `\u78BA\u5B9A\u8981\u5C07\u8033\u865F ${animal.ear_tag} \u767B\u8A18\u70BA\u731D\u6B7B\uFF1F\u6B64\u64CD\u4F5C\u4E0D\u53EF\u5FA9\u539F\u3002`,
      variant: 'destructive',
      confirmLabel: '\u78BA\u8A8D\u767B\u8A18',
    })
    if (ok) {
      mutations.createSuddenDeathMutation.mutate(mutations.suddenDeathForm)
    }
  }

  return (
    <div className="space-y-6">
      {/* Back Button & Export */}
      <div className="flex items-center justify-between">
        <Link
          to="/animals"
          className="inline-flex items-center text-slate-600 hover:text-slate-900"
        >
          <ArrowLeft className="h-4 w-4 mr-2" />
          {t('animalDetail.backToAnimalList', '\u56DE\u5230\u52D5\u7269\u5217\u8868')}
        </Link>
        <Button variant="outline" onClick={() => setShowExportDialog(true)}>
          <Download className="h-4 w-4 mr-2" />
          {t('animalDetail.exportRecord', '\u532F\u51FA\u75C5\u6B77')}
        </Button>
      </div>

      {/* Emergency Actions */}
      <AnimalDetailActions
        status={animal.status}
        onEmergencyMedication={() => setShowEmergencyMedicationDialog(true)}
        onEuthanasiaOrder={() => setShowEuthanasiaOrderDialog(true)}
        onSuddenDeath={() => mutations.setShowSuddenDeathDialog(true)}
      />

      {/* Animal Header Card */}
      <AnimalHeaderCard
        animal={animal}
        weights={queries.weights}
        showTrialSelect={mutations.showTrialSelect}
        setShowTrialSelect={mutations.setShowTrialSelect}
        approvedProtocols={queries.approvedProtocols}
        assignTrialMutation={mutations.assignTrialMutation}
      />

      {/* Tab Bar */}
      <TabBar
        activeTab={resolvedActiveTab}
        setActiveTab={setActiveTab}
        animalStatus={animal.status}
      />

      {/* Tab Content */}
      <TabContent
        activeTab={resolvedActiveTab}
        animal={animal}
        animalId={animalId}
        afterParam={queries.afterParam}
        observations={queries.observations}
        surgeries={queries.surgeries}
        weights={queries.weights}
        vaccinations={queries.vaccinations}
        sacrifice={queries.sacrifice ?? undefined}
        suddenDeath={queries.suddenDeath ?? undefined}
        transfers={queries.transfers}
        iacucEvents={queries.iacucEvents}
        hasAdminRole={hasRole('admin')}
        developerMode={developerMode}
        toggleDeveloperMode={toggleDeveloperMode}
        setActiveTab={setActiveTab}
      />

      {/* Dialogs */}
      <ExportDialog
        open={showExportDialog}
        onOpenChange={setShowExportDialog}
        type="single_animal"
        animalId={animalId}
        earTag={animal.ear_tag}
      />

      <EmergencyMedicationDialog
        open={showEmergencyMedicationDialog}
        onOpenChange={setShowEmergencyMedicationDialog}
        animalId={animalId}
        earTag={animal.ear_tag}
      />

      <EuthanasiaOrderDialog
        open={showEuthanasiaOrderDialog}
        onOpenChange={setShowEuthanasiaOrderDialog}
        animalId={animalId}
        earTag={animal.ear_tag}
        iacucNo={animal.iacuc_no}
      />

      <SuddenDeathDialog
        open={mutations.showSuddenDeathDialog}
        onOpenChange={mutations.setShowSuddenDeathDialog}
        earTag={animal.ear_tag}
        form={mutations.suddenDeathForm}
        onFormChange={mutations.setSuddenDeathForm}
        isPending={mutations.createSuddenDeathMutation.isPending}
        onConfirm={handleSuddenDeathConfirm}
      />

      <ConfirmDialog state={dialogState} />
    </div>
  )
}
