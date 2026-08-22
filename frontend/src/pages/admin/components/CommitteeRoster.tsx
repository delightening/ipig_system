/**
 * IACUC 委員名冊。
 *
 * 為什麼獨立於組織圖的部門樹：
 *
 * 委員身分來自**角色**（IACUC_CHAIR / REVIEWER / IACUC_STAFF），人事歸屬來自
 * **部門**（`users.department_id`）——兩者正交。委員會是跨部門的橫切面：獸醫
 * 可以兼任審查委員、執行秘書歸屬試驗部卻是委員會核心。而 `department_id` 是
 * 單值的，裝不下這種雙重身分。
 *
 * 硬把委員塞進 IACUC 部門，會逼出「一個人要同時屬於兩個部門」的假問題。
 * 正解是部門樹照常呈現人事歸屬，委員名冊另外依角色查出來——同一個人可以
 * 同時出現在兩處，各自表達不同的事實。
 *
 * 因此這份名冊也**不需要跟著部門編制維護**：指派了 REVIEWER 角色，人就自動
 * 出現在這裡；未來新增其他委員會或工作小組，加一份角色查詢即可，不必動
 * 資料結構。
 */
import { useQuery } from '@tanstack/react-query'
import { Users } from 'lucide-react'
import { facilityApi } from '@/lib/api/facility'
import { EmptyState } from '@/components/ui/empty-state'

export function CommitteeRoster() {
    const { data: members = [], isLoading, isError } = useQuery({
        queryKey: ['committee-members'],
        queryFn: async () => (await facilityApi.listCommitteeMembers()).data,
        staleTime: 5 * 60 * 1000,
    })

    return (
        <div className="rounded-lg border bg-card p-4">
            <div className="mb-1 flex items-center gap-2">
                <Users className="h-4 w-4 text-muted-foreground" />
                <h3 className="text-sm font-medium">IACUC 委員名冊</h3>
                {members.length > 0 && (
                    <span className="text-xs text-muted-foreground">（{members.length} 人）</span>
                )}
            </div>
            <p className="mb-3 text-xs text-muted-foreground">
                依角色列出，與部門編制無關——委員可以隸屬任何部門，也可以是外聘人員。
            </p>

            {isLoading ? (
                <div className="space-y-2">
                    {[0, 1, 2].map(i => (
                        <div key={i} className="h-8 animate-pulse rounded bg-muted" />
                    ))}
                </div>
            ) : isError ? (
                /* 錯誤與「真的沒有委員」必須分開講：兩者都讓 members 是空陣列，但
                   「尚無委員」會叫管理員去指派角色，而 403／斷線時角色其實早就指派好了，
                   照著做只會白忙一場。 */
                <p className="text-sm text-status-error-text">委員名冊載入失敗，請稍後再試。</p>
            ) : members.length === 0 ? (
                <EmptyState
                    icon={Users}
                    title="尚無委員"
                    description="指派 IACUC 主席、審查委員或執行秘書角色後，人員會自動出現在此。"
                />
            ) : (
                <ul className="divide-y">
                    {members.map(m => (
                        <li key={m.id} className="flex flex-wrap items-center gap-2 py-2 text-sm">
                            <span className="font-medium">{m.display_name}</span>
                            {/* 標出人事歸屬——名冊的價值就在於看得出「這位委員平時在哪個部門」，
                                例如兼任審查委員的獸醫會顯示「獸醫」而不是「IACUC」 */}
                            <span className="text-xs text-muted-foreground">
                                {m.department_name ?? '未編入部門'}
                            </span>
                            {!m.is_internal && (
                                <span className="text-xs text-muted-foreground">（外部人員）</span>
                            )}
                        </li>
                    ))}
                </ul>
            )}
        </div>
    )
}
