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

import { useAuthHasPermission } from '@/stores/auth'
import { PERMISSIONS } from '@/lib/permissions.generated'
import { PageHeader } from '@/components/ui/page-header'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { PageTabs, PageTabContent } from '@/components/ui/page-tabs'
import { Building2, Layers, Grid3X3, TreeDeciduous, Users, MapPin, Network } from 'lucide-react'

import { SpeciesTab } from './components/SpeciesTab'
import { FacilityTab } from './components/FacilityTab'
import { BuildingTab } from './components/BuildingTab'
import { ZoneTab } from './components/ZoneTab'
import { PenTab } from './components/PenTab'
import { DepartmentTab } from './components/DepartmentTab'
import { DepartmentOrgChartTab } from './components/DepartmentOrgChartTab'
import { CommitteeRoster } from './components/CommitteeRoster'

export function FacilitiesPage() {
  const hasPermission = useAuthHasPermission()
  // ⚠️ 原本寫 hasPermission('facilities.manage') —— **後端的碼是 facility.manage（單數）**，
  // 這個字串在 permissions 表裡不存在，永遠回 false。實際能看到管理按鈕的只有 admin
  // （靠 hasPermission 對 admin 的短路），與後端 facility.manage 目前未授予任何角色
  // 的結果剛好一致 —— 又是一次「對得上但是靠巧合」。
  // 改用產生式常數後，打錯字會在 tsc 階段就被擋下（PERMISSIONS 是後端 permissions 表產生的）。
  //
  // hasPermission('admin') 那一段也一併移除：'admin' 不是權限碼，它之所以有效是因為
  // hasPermission 內部對 admin 短路，而那個短路對任何權限碼都成立，寫了是多餘的。
  const canManage = hasPermission(PERMISSIONS.FACILITY_MANAGE)
  // 成員指派走 users.department_id 變更，對齊後端 admin.user.edit gate
  const canAssignMembers = hasPermission(PERMISSIONS.ADMIN_USER_EDIT)

  return (
    <div className="p-6 space-y-6">
      <PageHeader
        title="設施管理"
        description="管理物種分類、設施、棟舍、區域、欄位與部門架構"
      />

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base">基礎資料維護</CardTitle>
        </CardHeader>
        <CardContent>
          <PageTabs
            tabs={[
              { value: 'species', label: '物種', icon: TreeDeciduous },
              { value: 'facilities', label: '設施', icon: MapPin },
              { value: 'buildings', label: '棟舍', icon: Building2 },
              { value: 'zones', label: '區域', icon: Layers },
              { value: 'pens', label: '欄位', icon: Grid3X3 },
              { value: 'departments', label: '部門', icon: Users },
              { value: 'org-chart', label: '組織圖', icon: Network },
            ]}
            defaultTab="species"
          >
            <PageTabContent value="species">
              <SpeciesTab canManage={canManage} />
            </PageTabContent>
            <PageTabContent value="facilities">
              <FacilityTab canManage={canManage} />
            </PageTabContent>
            <PageTabContent value="buildings">
              <BuildingTab canManage={canManage} />
            </PageTabContent>
            <PageTabContent value="zones">
              <ZoneTab canManage={canManage} />
            </PageTabContent>
            <PageTabContent value="pens">
              <PenTab canManage={canManage} />
            </PageTabContent>
            <PageTabContent value="departments">
              <DepartmentTab canManage={canManage} canAssignMembers={canAssignMembers} />
            </PageTabContent>
            <PageTabContent value="org-chart">
              <div className="space-y-4">
                <DepartmentOrgChartTab />
                {/* 委員名冊獨立於部門樹：委員會是跨部門的橫切面，
                    同一個人可以同時出現在兩處 */}
                <CommitteeRoster />
              </div>
            </PageTabContent>
          </PageTabs>
        </CardContent>
      </Card>
    </div>
  )
}
