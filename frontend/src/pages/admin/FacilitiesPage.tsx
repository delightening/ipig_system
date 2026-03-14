/**
 * 設施管理頁 — 管理員後台
 *
 * 功能：
 * - 物種 (Species) CRUD
 * - 設施 (Facility) CRUD
 * - 棟舍 (Building) CRUD（依設施）
 * - 區域 (Zone) CRUD（依棟舍）
 * - 欄位 (Pen) CRUD（依區域）
 * - 部門 (Department) CRUD（樹狀結構）
 */

import { useTabState } from '@/hooks/useTabState'
import { useAuthStore } from '@/stores/auth'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Building2, Layers, Grid3X3, TreeDeciduous, Users, MapPin } from 'lucide-react'

import { SpeciesTab } from './components/SpeciesTab'
import { FacilityTab } from './components/FacilityTab'
import { BuildingTab } from './components/BuildingTab'
import { ZoneTab } from './components/ZoneTab'
import { PenTab } from './components/PenTab'
import { DepartmentTab } from './components/DepartmentTab'

type TabKey = 'species' | 'facilities' | 'buildings' | 'zones' | 'pens' | 'departments'

const TABS: { key: TabKey; label: string; icon: React.ReactNode }[] = [
  { key: 'species', label: '物種', icon: <TreeDeciduous className="h-4 w-4" /> },
  { key: 'facilities', label: '設施', icon: <MapPin className="h-4 w-4" /> },
  { key: 'buildings', label: '棟舍', icon: <Building2 className="h-4 w-4" /> },
  { key: 'zones', label: '區域', icon: <Layers className="h-4 w-4" /> },
  { key: 'pens', label: '欄位', icon: <Grid3X3 className="h-4 w-4" /> },
  { key: 'departments', label: '部門', icon: <Users className="h-4 w-4" /> },
]

export function FacilitiesPage() {
  const { hasPermission } = useAuthStore()
  const canManage = hasPermission('admin') || hasPermission('facilities.manage')
  const { activeTab, setActiveTab } = useTabState<TabKey>('species')

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold">設施管理</h1>
        <p className="text-muted-foreground">管理物種分類、設施、棟舍、區域、欄位與部門架構</p>
      </div>

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base">基礎資料維護</CardTitle>
        </CardHeader>
        <CardContent>
          <Tabs value={activeTab} onValueChange={v => setActiveTab(v as TabKey)}>
            <TabsList className="mb-4">
              {TABS.map(t => (
                <TabsTrigger key={t.key} value={t.key} className="gap-1.5">
                  {t.icon} {t.label}
                </TabsTrigger>
              ))}
            </TabsList>
            <TabsContent value="species">
              <SpeciesTab canManage={canManage} />
            </TabsContent>
            <TabsContent value="facilities">
              <FacilityTab canManage={canManage} />
            </TabsContent>
            <TabsContent value="buildings">
              <BuildingTab canManage={canManage} />
            </TabsContent>
            <TabsContent value="zones">
              <ZoneTab canManage={canManage} />
            </TabsContent>
            <TabsContent value="pens">
              <PenTab canManage={canManage} />
            </TabsContent>
            <TabsContent value="departments">
              <DepartmentTab canManage={canManage} />
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>
    </div>
  )
}
