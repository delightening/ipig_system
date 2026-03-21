import React, { lazy, Suspense, useMemo, useCallback } from 'react'
import { useTranslation } from 'react-i18next'

import {
  Animal,
  AnimalObservation,
  AnimalSurgery,
  AnimalWeight,
  AnimalVaccination,
  AnimalSacrifice,
  AnimalSuddenDeath,
  AnimalEvent,
  AnimalTransfer,
} from '@/lib/api'
import {
  Scale,
  Syringe,
  FileText,
  Scissors,
  ClipboardList,
  Heart,
  Stethoscope,
  Droplets,
  ArrowRightLeft,
  History,
} from 'lucide-react'
import { Skeleton } from '@/components/ui/skeleton'

import type { TabType } from '../constants'

// Lazy-loaded tab components
const AnimalTimelineView = lazy(() => import('@/components/animal/AnimalTimelineView').then(m => ({ default: m.AnimalTimelineView })))
const ObservationsTab = lazy(() => import('@/components/animal/ObservationsTab').then(m => ({ default: m.ObservationsTab })))
const SurgeriesTab = lazy(() => import('@/components/animal/SurgeriesTab').then(m => ({ default: m.SurgeriesTab })))
const WeightsTab = lazy(() => import('@/components/animal/WeightsTab').then(m => ({ default: m.WeightsTab })))
const VaccinationsTab = lazy(() => import('@/components/animal/VaccinationsTab').then(m => ({ default: m.VaccinationsTab })))
const SacrificeTab = lazy(() => import('@/components/animal/SacrificeTab').then(m => ({ default: m.SacrificeTab })))
const AnimalInfoTab = lazy(() => import('@/components/animal/AnimalInfoTab').then(m => ({ default: m.AnimalInfoTab })))
const PathologyTab = lazy(() => import('@/components/animal/PathologyTab').then(m => ({ default: m.PathologyTab })))
const BloodTestTab = lazy(() => import('@/components/animal/BloodTestTab').then(m => ({ default: m.BloodTestTab })))
const TransferTab = lazy(() => import('@/components/animal/TransferTab').then(m => ({ default: m.TransferTab })))
const PainAssessmentTab = lazy(() => import('@/components/animal/PainAssessmentTab').then(m => ({ default: m.PainAssessmentTab })))

const TabFallback = () => <Skeleton variant="form" fields={4} />

interface TabBarProps {
  activeTab: TabType
  setActiveTab: (tab: TabType) => void
  animalStatus: string
}

export function TabBar({ activeTab, setActiveTab, animalStatus }: TabBarProps) {
  const { t } = useTranslation()

  const tabs = useMemo(
    () => [
      { id: 'timeline' as const, label: t('animalDetail.tabs.timeline', '\u7D00\u9304\u6642\u9593\u8EF8'), icon: History },
      { id: 'observations' as const, label: t('animalDetail.tabs.observations', '\u89C0\u5BDF\u8A66\u9A57\u7D00\u9304'), icon: ClipboardList },
      { id: 'surgeries' as const, label: t('animalDetail.tabs.surgeries', '\u624B\u8853\u7D00\u9304'), icon: Scissors },
      { id: 'weights' as const, label: t('animalDetail.tabs.weights', '\u9AD4\u91CD\u7D00\u9304'), icon: Scale },
      { id: 'vaccinations' as const, label: t('animalDetail.tabs.vaccinations', '\u758B\u82D7/\u9A45\u87F2\u7D00\u9304'), icon: Syringe },
      { id: 'sacrifice' as const, label: t('animalDetail.tabs.sacrifice', '\u72A7\u7272/\u63A1\u6A23\u7D00\u9304'), icon: Heart },
      { id: 'blood_tests' as const, label: t('animalDetail.tabs.bloodTests', '\u8840\u6DB2\u6AA2\u67E5'), icon: Droplets },
      { id: 'pain_assessment' as const, label: t('animalDetail.tabs.painAssessment', '\u75BC\u75DB\u8A55\u4F30'), icon: Stethoscope },
      { id: 'info' as const, label: t('animalDetail.tabs.info', '\u52D5\u7269\u8CC7\u6599'), icon: FileText },
      { id: 'pathology' as const, label: t('animalDetail.tabs.pathology', '\u75C5\u7406\u7D44\u7E54\u5831\u544A'), icon: FileText },
      ...((animalStatus === 'completed' || animalStatus === 'transferred')
        ? [{ id: 'transfer' as const, label: t('animalDetail.tabs.transfer', '\u8F49\u8B93\u7BA1\u7406'), icon: ArrowRightLeft }]
        : []),
    ],
    [t, animalStatus],
  )

  return (
    <div className="border-b border-slate-200">
      <div className="flex flex-wrap gap-x-6 gap-y-1">
        {tabs.map((tab) => {
          const Icon = tab.icon
          const isActive = activeTab === tab.id
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-2 py-3 text-sm font-medium transition-colors border-b-2 -mb-px ${
                isActive
                  ? 'border-orange-500 text-orange-600'
                  : 'border-transparent text-slate-500 hover:text-slate-700'
              }`}
            >
              <Icon className="h-4 w-4 shrink-0" />
              {tab.label}
            </button>
          )
        })}
      </div>
    </div>
  )
}

interface TabContentProps {
  activeTab: TabType
  animal: Animal
  animalId: string
  afterParam: string
  observations: AnimalObservation[] | undefined
  surgeries: AnimalSurgery[] | undefined
  weights: AnimalWeight[] | undefined
  vaccinations: AnimalVaccination[] | undefined
  sacrifice: AnimalSacrifice | undefined
  suddenDeath: AnimalSuddenDeath | undefined
  transfers: AnimalTransfer[] | undefined
  iacucEvents: AnimalEvent[] | undefined
  hasAdminRole: boolean
  developerMode: boolean
  toggleDeveloperMode: () => void
  setActiveTab: (tab: TabType) => void
}

export function TabContent({
  activeTab,
  animal,
  animalId,
  afterParam,
  observations,
  surgeries,
  weights,
  vaccinations,
  sacrifice,
  suddenDeath,
  transfers,
  iacucEvents,
  hasAdminRole,
  developerMode,
  toggleDeveloperMode,
  setActiveTab,
}: TabContentProps) {
  const handleTimelineAction = useCallback(
    (type: string) => {
      setActiveTab(type === 'observation' ? 'observations' : 'surgeries')
    },
    [setActiveTab],
  )

  return (
    <div className="mt-6">
      <Suspense fallback={<TabFallback />}>
        {activeTab === 'timeline' && (
          <AnimalTimelineView
            observations={observations || []}
            surgeries={surgeries || []}
            animalWeights={weights || []}
            sacrifice={sacrifice || undefined}
            suddenDeath={suddenDeath || undefined}
            transfers={transfers || []}
            iacucEvents={iacucEvents || []}
            animal={animal}
            onView={handleTimelineAction}
            onEdit={handleTimelineAction}
            onCopy={handleTimelineAction}
            onHistory={handleTimelineAction}
            onVet={handleTimelineAction}
            onDelete={handleTimelineAction}
          />
        )}

        {activeTab === 'observations' && (
          <ObservationsTab
            animalId={animalId}
            earTag={animal.ear_tag}
            afterParam={afterParam}
            observations={observations}
          />
        )}

        {activeTab === 'surgeries' && (
          <SurgeriesTab
            animalId={animalId}
            earTag={animal.ear_tag}
            afterParam={afterParam}
            surgeries={surgeries}
          />
        )}

        {activeTab === 'weights' && (
          <WeightsTab
            animalId={animalId}
            earTag={animal.ear_tag}
            afterParam={afterParam}
            weights={weights}
            hasAdminRole={hasAdminRole}
            developerMode={developerMode}
            toggleDeveloperMode={toggleDeveloperMode}
          />
        )}

        {activeTab === 'vaccinations' && (
          <VaccinationsTab
            animalId={animalId}
            earTag={animal.ear_tag}
            afterParam={afterParam}
            vaccinations={vaccinations}
          />
        )}

        {activeTab === 'sacrifice' && (
          <SacrificeTab
            animalId={animalId}
            earTag={animal.ear_tag}
            sacrifice={sacrifice}
          />
        )}

        {activeTab === 'info' && <AnimalInfoTab animal={animal} />}

        {activeTab === 'blood_tests' && (
          <BloodTestTab animalId={animalId} afterParam={afterParam} />
        )}

        {activeTab === 'pain_assessment' && (
          <PainAssessmentTab
            animalId={animalId}
            observations={(observations || []).map((o) => ({
              id: o.id,
              observation_date: o.event_date,
            }))}
            surgeries={(surgeries || []).map((s) => ({
              id: s.id,
              surgery_date: s.surgery_date,
            }))}
          />
        )}

        {activeTab === 'transfer' && (
          <TransferTab
            animalId={animalId}
            animalStatus={animal.status}
            earTag={animal.ear_tag}
          />
        )}

        {activeTab === 'pathology' && (
          <PathologyTab animalId={animalId} earTag={animal.ear_tag} />
        )}
      </Suspense>
    </div>
  )
}
