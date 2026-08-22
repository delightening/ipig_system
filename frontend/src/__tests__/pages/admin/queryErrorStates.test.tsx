/**
 * R99-1：`pages/admin/components/` 的查詢失敗必須跟「真的沒有資料」分開呈現。
 *
 * 這些元件過去一律寫成 `const { data = [], isLoading } = useQuery(...)`——查詢失敗時
 * `data` 是 `undefined`，經預設值變成空陣列，於是 403／斷線／500 全部渲染成空狀態，
 * 而空狀態的文案往往還會叫使用者去做一件其實早就做過的事（「指派角色後人員會自動
 * 出現」）。畫面上不會有任何錯誤訊息，真正的失敗原因被完全蓋掉。
 *
 * 每支元件一條 `mockRejectedValue` 案例，**同時斷言錯誤訊息出現、空狀態文案不出現**
 * ——只斷言前者的話，兩個分支都渲染時測試仍會綠。
 *
 * 用單一檔案而非 13 個檔案：13 條案例的斷言形狀完全相同（錯誤在、空狀態不在），
 * 差別只在元件的 props 與該 mock 哪支 API。集中放一起，日後新增元件時照抄一列即可，
 * 也讓「這條契約適用於整個目錄」這件事在檔案結構上看得出來。
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
// 用 fireEvent 而非 @testing-library/user-event：後者不在專案依賴內
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { MemoryRouter } from 'react-router-dom'

// t() 回傳 key 本身，斷言就直接比對 key，不受文案改寫影響
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { defaultValue?: string }) => opts?.defaultValue ?? key,
  }),
  // `lib/utils` 會拉進 `lib/i18n`，後者 `.use(initReactI18next)`——mock 掉整個模組
  // 就得補上這個 plugin 形狀，否則 i18next 在 import 階段就炸
  initReactI18next: { type: '3rdParty', init: () => {} },
}))

// vi.mock 的 factory 會被提升到檔案最上方，直接引用一般 const 會踩 TDZ；
// 用 vi.hoisted 讓這些 mock 物件跟著一起提升。
const { facilityApi, alertLockApi, ipBlocklistApi, invitationApi, apiGet, useInternalUsersBrief } =
  vi.hoisted(() => ({
    facilityApi: {
      listFacilities: vi.fn(),
      listBuildings: vi.fn(),
      listZones: vi.fn(),
      listPens: vi.fn(),
      listSpecies: vi.fn(),
      listDepartments: vi.fn(),
      listDepartmentMembers: vi.fn(),
      listAllDepartmentMembers: vi.fn(),
    },
    alertLockApi: { status: vi.fn(), clearAccountLockout: vi.fn() },
    ipBlocklistApi: { list: vi.fn(), add: vi.fn(), unblock: vi.fn() },
    invitationApi: { availableRoles: vi.fn(), create: vi.fn() },
    apiGet: vi.fn(),
    useInternalUsersBrief: vi.fn((): { data: unknown[] } => ({ data: [] })),
  }))

vi.mock('@/lib/api/facility', () => ({ facilityApi }))
vi.mock('@/lib/api/alertLock', () => ({ alertLockApi }))
vi.mock('@/lib/api/ipBlocklist', () => ({ ipBlocklistApi }))
vi.mock('@/lib/api/invitation', () => ({ invitationApi }))
vi.mock('@/lib/api', () => ({ default: { get: apiGet } }))
vi.mock('@/hooks/useInternalUsersBrief', () => ({ useInternalUsersBrief }))

import { AlertLockPanel } from '@/pages/admin/components/AlertLockPanel'
import { BuildingTab } from '@/pages/admin/components/BuildingTab'
import { DepartmentMembersDialog } from '@/pages/admin/components/DepartmentMembersDialog'
import { DepartmentOrgChartTab } from '@/pages/admin/components/DepartmentOrgChartTab'
import { DepartmentTab } from '@/pages/admin/components/DepartmentTab'
import { EquipmentTabContent } from '@/pages/admin/components/EquipmentTabContent'
import { FacilityTab } from '@/pages/admin/components/FacilityTab'
import { InvitationCreateDialog } from '@/pages/admin/components/InvitationCreateDialog'
import { IpBlocklistTab } from '@/pages/admin/components/IpBlocklistTab'
import { MaintenanceHistoryDialog } from '@/pages/admin/components/MaintenanceHistoryDialog'
import { PenTab } from '@/pages/admin/components/PenTab'
import { SpeciesTab } from '@/pages/admin/components/SpeciesTab'
import { ZoneTab } from '@/pages/admin/components/ZoneTab'
import type { DepartmentWithManager } from '@/types/facility'
import type { Equipment } from '@/pages/admin/types'

const LOAD_FAILED = 'common.loadFailed'

function renderWithClient(ui: React.ReactElement, seed?: (qc: QueryClient) => void) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  seed?.(queryClient)
  return {
    queryClient,
    ...render(
      <QueryClientProvider client={queryClient}>
        <MemoryRouter>{ui}</MemoryRouter>
      </QueryClientProvider>,
    ),
  }
}

/**
 * 等到那支查詢真的進入 error 狀態才斷言。
 *
 * ⚠️ 少了這一步，「快取仍在」那組測試會是**假綠**：`findByText` 在首次 render
 * 就找到快取資料並立刻回來，此時 refetch 根本還沒 reject，於是拿掉修法也照樣過
 * （本輪實測：還原修法後那兩條仍然綠，據此改成現在的寫法）。
 */
async function waitForQueryError(queryClient: QueryClient, queryKey: unknown[]) {
  await waitFor(() =>
    expect(queryClient.getQueryState(queryKey)?.status).toBe('error'),
  )
}

/** 讓所有次要查詢有東西可回，測試才只在「主查詢失敗」這一個變因上 */
function resolveAll() {
  for (const fn of Object.values(facilityApi)) fn.mockResolvedValue({ data: [] })
  alertLockApi.status.mockResolvedValue({ data: { account: null, ip: null } })
  ipBlocklistApi.list.mockResolvedValue({ data: { items: [] } })
  invitationApi.availableRoles.mockResolvedValue({ data: [] })
  apiGet.mockResolvedValue({ data: { data: [] } })
}

beforeEach(() => {
  vi.clearAllMocks()
  resolveAll()
})

/** 斷言錯誤分支取代了空狀態，而非兩個都畫 */
async function expectErrorNotEmpty(errorText: string, emptyText: string) {
  expect(await screen.findByText(errorText)).toBeInTheDocument()
  expect(screen.queryByText(emptyText)).not.toBeInTheDocument()
}

describe('admin 元件的查詢失敗不得渲染成空狀態', () => {
  it('AlertLockPanel：狀態查不到時明說失敗，而非靜默隱藏整個面板', async () => {
    alertLockApi.status.mockRejectedValue(new Error('503'))

    renderWithClient(<AlertLockPanel alertId="alert-1" />)

    // 面板整個消失時使用者會以為「沒鎖」——實際是不知道有沒有鎖
    expect(await screen.findByText(LOAD_FAILED)).toBeInTheDocument()
  })

  it('BuildingTab', async () => {
    facilityApi.listBuildings.mockRejectedValue(new Error('403'))

    renderWithClient(<BuildingTab canManage />)

    await expectErrorNotEmpty(LOAD_FAILED, 'common.noData')
  })

  it('DepartmentTab', async () => {
    facilityApi.listDepartments.mockRejectedValue(new Error('403'))

    renderWithClient(<DepartmentTab canManage canAssignMembers />)

    await expectErrorNotEmpty(LOAD_FAILED, 'admin.departmentTab.emptyTitle')
  })

  it('FacilityTab', async () => {
    facilityApi.listFacilities.mockRejectedValue(new Error('403'))

    renderWithClient(<FacilityTab canManage />)

    await expectErrorNotEmpty(LOAD_FAILED, 'common.noData')
  })

  it('PenTab', async () => {
    facilityApi.listPens.mockRejectedValue(new Error('403'))

    renderWithClient(<PenTab canManage />)

    await expectErrorNotEmpty(LOAD_FAILED, 'common.noData')
  })

  it('SpeciesTab', async () => {
    facilityApi.listSpecies.mockRejectedValue(new Error('403'))

    renderWithClient(<SpeciesTab canManage />)

    await expectErrorNotEmpty(LOAD_FAILED, 'common.noData')
  })

  it('ZoneTab', async () => {
    facilityApi.listZones.mockRejectedValue(new Error('403'))

    renderWithClient(<ZoneTab canManage />)

    await expectErrorNotEmpty(LOAD_FAILED, 'common.noData')
  })

  it('IpBlocklistTab', async () => {
    ipBlocklistApi.list.mockRejectedValue(new Error('500'))

    renderWithClient(<IpBlocklistTab />)

    await expectErrorNotEmpty(LOAD_FAILED, 'admin.ipBlocklistTab.emptyTitle')
  })

  it('DepartmentMembersDialog', async () => {
    facilityApi.listDepartmentMembers.mockRejectedValue(new Error('403'))
    const department = { id: 'dep-1', name: '獸醫部' } as DepartmentWithManager

    renderWithClient(
      <DepartmentMembersDialog open onOpenChange={() => {}} department={department} canManage />,
    )

    await expectErrorNotEmpty(LOAD_FAILED, 'admin.departmentTab.members.empty')
  })

  it('DepartmentOrgChartTab：部門查詢失敗', async () => {
    facilityApi.listDepartments.mockRejectedValue(new Error('403'))

    renderWithClient(<DepartmentOrgChartTab />)

    await expectErrorNotEmpty(LOAD_FAILED, 'admin.departmentTab.orgChart.empty')
  })

  // 成員查詢單獨失敗時整棵樹還在，但每個部門都顯示 0 人——與「部門真的沒人」同形，
  // 所以不能只擋部門那支查詢
  it('DepartmentOrgChartTab：成員查詢失敗', async () => {
    facilityApi.listDepartments.mockResolvedValue({
      data: [{ id: 'dep-1', name: '獸醫部', parent_id: null }],
    })
    facilityApi.listAllDepartmentMembers.mockRejectedValue(new Error('403'))

    renderWithClient(<DepartmentOrgChartTab />)

    expect(await screen.findByText(LOAD_FAILED)).toBeInTheDocument()
  })

  it('MaintenanceHistoryDialog', async () => {
    apiGet.mockRejectedValue(new Error('500'))

    renderWithClient(
      <MaintenanceHistoryDialog open onOpenChange={() => {}} recordId="rec-1" />,
    )

    await expectErrorNotEmpty(LOAD_FAILED, 'admin.maintenanceHistoryDialog.empty')
  })

  it('InvitationCreateDialog：角色清單載不出來時說明原因（角色為必填，否則卡住無解）', async () => {
    invitationApi.availableRoles.mockRejectedValue(new Error('500'))

    renderWithClient(
      <InvitationCreateDialog open onOpenChange={() => {}} onSuccess={() => {}} />,
    )

    expect(await screen.findByText(LOAD_FAILED)).toBeInTheDocument()
    expect(screen.queryByText('admin.invitationCreateDialog.rolesLoading')).not.toBeInTheDocument()
  })

  // ── 反向的另一半契約 ────────────────────────────────────────────────────
  // TanStack Query v5 在「已成功取過、之後 refetch 失敗」時，`isError` 會是 true
  // 而 `data` 仍保留上一次的結果（本 repo 以 renderHook 實測確認）。所以錯誤分支
  // 的條件必須是「`isError` 且手上沒有資料可顯示」——只看 `isError` 的話，使用者
  // 新增一筆資料後 `invalidateQueries` 的重取一旦失敗，畫面上原本好好的資料就會
  // 整片被換成一行錯誤字。這比原本的 bug 更糟：那至少還看得到真的資料。
  describe('已有快取資料時，refetch 失敗不得把資料換成錯誤訊息', () => {
    it('BuildingTab：仍顯示既有列，不顯示錯誤列', async () => {
      facilityApi.listBuildings.mockRejectedValue(new Error('500'))

      const { queryClient } = renderWithClient(<BuildingTab canManage />, (qc) =>
        qc.setQueryData(['buildings'], [
          {
            id: 'b-1',
            code: 'B01',
            name: '一號舍',
            facility_name: '本場',
            facility_code: 'F01',
            sort_order: 1,
            is_active: true,
          },
        ]),
      )

      await waitForQueryError(queryClient, ['buildings'])

      expect(screen.getByText('一號舍')).toBeInTheDocument()
      expect(screen.queryByText(LOAD_FAILED)).not.toBeInTheDocument()
    })

    it('AlertLockPanel：仍顯示既有鎖定狀態，不整片換成錯誤字', async () => {
      alertLockApi.status.mockRejectedValue(new Error('503'))

      const { queryClient } = renderWithClient(<AlertLockPanel alertId="alert-1" />, (qc) =>
        qc.setQueryData(['alert-lock-status', 'alert-1'], {
          account: {
            email: 'locked@example.test',
            locked: true,
            user_exists: true,
            fail_count: 5,
            max_attempts: 5,
            unlock_at: null,
          },
          ip: null,
        }),
      )

      await waitForQueryError(queryClient, ['alert-lock-status', 'alert-1'])

      expect(screen.getByText('locked@example.test')).toBeInTheDocument()
      expect(screen.queryByText(LOAD_FAILED)).not.toBeInTheDocument()
    })
  })

  // CodeRabbit 於 PR #26 指出（Major）：這兩支查詢是「指派成員」的兩道把關——
  // `members` 用來去重、`allMembers` 用來判斷要不要跳轉調確認。任一支失敗都會退成
  // 空陣列，於是 `current` 查不到、`handleAdd` 的確認整段被跳過，按下去就把人從
  // 原部門靜默移走。屬既有行為（非本 PR 引入），但踩到「防資料遺失」底線，一併修。
  describe('DepartmentMembersDialog：把關查詢失敗時不得放行指派', () => {
    const department = { id: 'dep-1', name: '獸醫部' } as DepartmentWithManager

    function renderDialog() {
      return renderWithClient(
        <DepartmentMembersDialog open onOpenChange={() => {}} department={department} canManage />,
      )
    }

    it('轉調判斷用的 allMembers 查詢失敗時，指派按鈕停用並說明原因', async () => {
      useInternalUsersBrief.mockReturnValue({
        data: [{ id: 'u-1', display_name: '甲君', is_internal: true }],
      })
      facilityApi.listAllDepartmentMembers.mockRejectedValue(new Error('500'))

      const { queryClient } = renderDialog()
      await waitForQueryError(queryClient, ['department-members-all'])

      expect(
        screen.getByText('admin.departmentTab.members.assignBlockedByLoadFailure'),
      ).toBeInTheDocument()
      // 斷言人員下拉本身被停用，不斷言「加入」按鈕——後者本來就會因為
      // `!selectedUser` 而 disabled，拿它當證據的話，拿掉修法測試照樣綠
      expect(screen.getByRole('combobox')).toBeDisabled()
    })
  })

  describe('EquipmentTabContent', () => {
    const equipment = {
      id: 'eq-1',
      name: '離心機',
      model: null,
      serial_number: null,
      location: null,
      status: 'active',
      calibration_type: null,
    } as unknown as Equipment

    function renderEquipmentTab() {
      return renderWithClient(
        <EquipmentTabContent
          canManage
          keyword=""
          onKeywordChange={() => {}}
          statusFilter=""
          onStatusFilterChange={() => {}}
          allCalibrations={[]}
          tableProps={{
            records: [equipment],
            isLoading: false,
            page: 1,
            totalPages: 1,
            onPageChange: () => {},
          }}
          actions={{ onEdit: () => {}, onDelete: () => {} }}
        />,
      )
    }

    // 廠商 summary 掛掉時整欄退成「—」，跟「這台設備沒登錄廠商」完全同形，
    // 而「沒登錄廠商」在稽核上是要去補登的——所以必須講出來
    it('廠商 summary 失敗時在表格上方說明，破折號不代表未登錄', async () => {
      apiGet.mockRejectedValue(new Error('500'))

      renderEquipmentTab()

      expect(
        await screen.findByText('admin.equipmentTabContent.supplierSummaryLoadFailed'),
      ).toBeInTheDocument()
    })

    it('廠商明細 Dialog 載入失敗時不顯示「尚未關聯廠商」', async () => {
      apiGet.mockImplementation((url: string) =>
        url === '/equipment-suppliers/summary'
          ? Promise.resolve({ data: [{ equipment_id: 'eq-1', partner_name: '甲廠商' }] })
          : Promise.reject(new Error('500')),
      )

      renderEquipmentTab()

      fireEvent.click(await screen.findByText('甲廠商'))

      await expectErrorNotEmpty(LOAD_FAILED, 'admin.equipmentTabContent.noSuppliers')
    })
  })
})
