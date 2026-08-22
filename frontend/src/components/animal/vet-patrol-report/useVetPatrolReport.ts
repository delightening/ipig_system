// 獸醫巡場報告 Dialog 核心狀態機（R82-7 由 VetPatrolReportDialog.tsx 抽出）
//
// 保留原元件內宣告順序：初始草稿 effect 對 upsertMutation 的前向引用依賴
// closure binding（呼叫時才解析），不可調動宣告先後。

import { useState, useEffect, useMemo, useRef, useCallback } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { format } from 'date-fns'
import api from '@/lib/api'
import { useAuthUser } from '@/stores/auth'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { CATEGORIES, emptyEntry } from './constants'
import type { CategoryKey } from './constants'
import type { AnimalOption, EntryRow, PatrolReport } from './types'
import { usePatrolReportPhotos } from './usePatrolReportPhotos'
import { usePatrolEntryPhotos } from './usePatrolEntryPhotos'
import { usePatrolPdfExport } from './usePatrolPdfExport'

interface UseVetPatrolReportArgs {
    open: boolean
    onOpenChange: (open: boolean) => void
    editReportId?: string | null
}

export function useVetPatrolReport({ open, onOpenChange, editReportId }: UseVetPatrolReportArgs) {
    const queryClient = useQueryClient()
    const currentUser = useAuthUser()
    const today = format(new Date(), 'yyyy-MM-dd')

    const [patrolDate, setPatrolDate] = useState(today)
    const [accompanyingPersonnel, setAccompanyingPersonnel] = useState('')
    /** R39+ 兩階段流程：指派的追蹤者（陪同人員的 user.id），送出時當 follow_up_user_id 用 */
    const [followUpUserId, setFollowUpUserId] = useState<string>('')
    /** 報告當前 status；空白＝新建（尚未送 server）；其他依 server 回 */
    const [reportStatus, setReportStatus] = useState<'draft' | 'awaiting_acknowledgement' | 'awaiting_follow_up' | 'completed'>('draft')
    const [reportCreatedBy, setReportCreatedBy] = useState<string | null>(null)
    const [reportFollowUpUserId, setReportFollowUpUserId] = useState<string | null>(null)
    const [entries, setEntries] = useState<Record<CategoryKey, EntryRow[]>>({
        pig_condition: [emptyEntry('pig_condition')],
        epidemic_prevention: [emptyEntry('epidemic_prevention')],
        case_record: [emptyEntry('case_record')],
        other: [emptyEntry('other')],
    })
    const [collapsedCategories, setCollapsedCategories] = useState<Set<string>>(new Set())
    const [savedReportId, setSavedReportId] = useState<string | null>(editReportId ?? null)
    const [hasInteracted, setHasInteracted] = useState(false)
    /**
     * R39+++ 嚴格存檔模式：
     * - committedAnyChange：本次 dialog 期間是否已執行過任何 server-side commit
     *   （存草稿 / 上傳照片 / 送出 / 確認 / 完成）。關閉時若 false → 預建的草稿會被棄置。
     * - hasUnsavedTextChanges：client side 文字 / 動物選擇變更但還沒按存草稿。關閉時 true → 跳 confirm。
     * - wasPreCreated：本次 dialog 是「新建」流程預先建立草稿的（vs editReportId 帶進來的既有報告）。
     *   只有 wasPreCreated 的 draft 在 !committedAnyChange 時會被 hard delete。
     */
    const [committedAnyChange, setCommittedAnyChange] = useState(false)
    const [hasUnsavedTextChanges, setHasUnsavedTextChanges] = useState(false)
    const wasPreCreatedRef = useRef(false)
    const markCommitted = useCallback(() => setCommittedAnyChange(true), [])

    // ── R39++ 三階段角色判定 ──
    // 新建報告（沒 server id）：當前使用者就是獸醫
    // 既有報告：created_by / follow_up_user_id 對照 currentUser.id
    const isNewReport = !savedReportId
    const isVet = isNewReport || (currentUser?.id != null && currentUser.id === reportCreatedBy)
    const isFollowUpTracker =
        currentUser?.id != null && currentUser.id === reportFollowUpUserId
    const isCompleted = reportStatus === 'completed'
    const isAwaitingAcknowledgement = reportStatus === 'awaiting_acknowledgement'
    const isAwaitingFollowUp = reportStatus === 'awaiting_follow_up'
    const isDraft = reportStatus === 'draft'
    // 唯讀條件：完成鎖定 / acknowledgement 階段 / 非該階段負責人
    // awaiting_acknowledgement → 必須先按「確認收到」才能進入填寫階段，全程鎖內容
    const isReadOnly =
        isCompleted ||
        isAwaitingAcknowledgement ||
        (isDraft && !isVet) ||
        (isAwaitingFollowUp && !isFollowUpTracker)
    // 追蹤者階段：只能填 follow_up 欄位
    const canEditFollowUpOnly = isAwaitingFollowUp && isFollowUpTracker

    // 取得動物列表（耳號選擇用）
    const { data: animals } = useQuery({
        queryKey: ['animals-for-patrol'],
        queryFn: async () => {
            // backend AnimalQuery.status 是單一 enum 不接受 comma list；改不帶 status filter
            // 拿全部，前端 client-side 過濾「在欄活體」(unassigned + in_experiment)
            const res = await api.get<{ data: (AnimalOption & { status?: string })[] }>('/animals?per_page=9999')
            const aliveStatuses = new Set(['unassigned', 'in_experiment'])
            return res.data.data.filter(a => !a.status || aliveStatuses.has(a.status))
        },
        staleTime: 60_000,
        enabled: open,
    })

    const animalOptions = (animals ?? []).map(a => ({
        value: a.id,
        label: a.ear_tag,
        description: a.pen_location ?? '',
    }))

    // 取得 staff 列表（陪同人員下拉用）
    // 獸醫權限路徑後端回傳最小 DTO（僅 id + display_name，不含 email 等 PII）
    const { data: staffList } = useQuery({
        queryKey: ['hr-staff-for-patrol'],
        queryFn: async () => {
            // include_external=true：陪同人員不是人事作業，外聘獸醫本來就會陪同巡場。
            // 端點預設只回本場受僱人員（供請假代理人等人事用途），這裡要明確放寬。
            const res = await api.get<Array<{ id: string; display_name: string }>>(
                '/hr/staff?include_external=true',
            )
            return res.data
        },
        staleTime: 5 * 60_000,
        enabled: open,
    })

    const staffOptions = useMemo(() => {
        const opts = (staffList ?? []).map(s => ({
            value: s.display_name,
            label: s.display_name,
        }))
        if (
            accompanyingPersonnel &&
            !opts.some(o => o.value === accompanyingPersonnel)
        ) {
            opts.unshift({
                value: accompanyingPersonnel,
                label: accompanyingPersonnel,
            })
        }
        return opts
    }, [staffList, accompanyingPersonnel])

    // 載入既有報告
    const { data: existingReport } = useQuery({
        queryKey: ['vet-patrol-report', editReportId],
        queryFn: async () => {
            const res = await api.get<PatrolReport>(`/vet-patrol-reports/${editReportId}`)
            return res.data
        },
        enabled: !!editReportId && open,
    })

    useEffect(() => {
        if (existingReport) {
            setPatrolDate(existingReport.patrol_date)
            setAccompanyingPersonnel(existingReport.accompanying_personnel ?? '')
            setReportStatus(existingReport.status)
            setReportCreatedBy(existingReport.created_by)
            setReportFollowUpUserId(existingReport.follow_up_user_id)
            setFollowUpUserId(existingReport.follow_up_user_id ?? '')

            const grouped: Record<CategoryKey, EntryRow[]> = {
                pig_condition: [],
                epidemic_prevention: [],
                case_record: [],
                other: [],
            }
            for (const e of existingReport.entries) {
                const cat = e.category as CategoryKey
                if (grouped[cat]) {
                    // R39+++ 多動物：優先用 animal_ids，舊資料 fallback 到單一 animal_id
                    const ids = e.animal_ids && e.animal_ids.length > 0
                        ? e.animal_ids
                        : (e.animal_id ? [e.animal_id] : [])
                    grouped[cat].push({
                        id: e.id,
                        tempKey: e.id, // 用 server id 當 key 確保穩定
                        category: cat,
                        animal_ids: ids,
                        observation: e.observation,
                        suggestion: e.suggestion,
                        follow_up: e.follow_up,
                    })
                }
            }
            for (const cat of CATEGORIES) {
                if (grouped[cat.key].length === 0) {
                    grouped[cat.key] = [emptyEntry(cat.key)]
                }
            }
            setEntries(grouped)
        }
    }, [existingReport])

    // 載入草稿時 follow_up_user_id 可能是 null（auto-save 不存此欄），
    // 但陪同人員文字已存 → 等 staffList 到手後反查 UUID，讓「送出給追蹤者」按鈕可點
    useEffect(() => {
        if (followUpUserId || !accompanyingPersonnel || !staffList?.length) return
        const match = staffList.find(s => s.display_name === accompanyingPersonnel)
        if (match) setFollowUpUserId(match.id)
    }, [staffList, accompanyingPersonnel, followUpUserId])

    // Reset on open
    useEffect(() => {
        if (open && !editReportId) {
            setPatrolDate(today)
            setAccompanyingPersonnel('')
            setFollowUpUserId('')
            setReportStatus('draft')
            setReportCreatedBy(null)
            setReportFollowUpUserId(null)
            setEntries({
                pig_condition: [emptyEntry('pig_condition')],
                epidemic_prevention: [emptyEntry('epidemic_prevention')],
                case_record: [emptyEntry('case_record')],
                other: [emptyEntry('other')],
            })
            setCollapsedCategories(new Set())
            setSavedReportId(null)
            setHasInteracted(false)
            setCommittedAnyChange(false)
            setHasUnsavedTextChanges(false)
            wasPreCreatedRef.current = false
        } else if (open && editReportId) {
            setSavedReportId(editReportId)
            setHasInteracted(true)
            setCommittedAnyChange(false)
            setHasUnsavedTextChanges(false)
            wasPreCreatedRef.current = false
        }
    }, [open, editReportId, today])

    // R39+++ 預先建立空草稿：開新報告 dialog 時自動建一筆 draft 拿到 id，
    // 讓使用者一開始就能上傳照片（不用等先輸入內容觸發 auto-save）。
    // 沒填內容直接關 dialog 的草稿由 7 天 GC（services/animal/vet_patrol::cleanup_stale_drafts）自動清。
    const initialDraftCreatedRef = useRef(false)
    // R39++++ addRow spam-click catch-up flag — 詳見 addRow / upsertMutation.onSuccess
    const pendingAddRowRef = useRef(false)
    const scrollToNewRowRef = useRef<string | null>(null)
    useEffect(() => {
        if (!open) {
            initialDraftCreatedRef.current = false
            pendingAddRowRef.current = false
            return
        }
        if (editReportId || savedReportId || initialDraftCreatedRef.current) return
        initialDraftCreatedRef.current = true
        wasPreCreatedRef.current = true
        upsertMutation.mutate()
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [open, editReportId, savedReportId])

    // ── auto-save mutation：建 draft + 後續更新 ──
    const upsertMutation = useMutation({
        mutationFn: async (): Promise<PatrolReport> => {
            const allEntries = Object.values(entries).flat()
                .map((e, i) => ({
                    id: e.id,
                    category: e.category,
                    animal_ids: e.animal_ids,
                    observation: e.observation,
                    suggestion: e.suggestion,
                    follow_up: e.follow_up,
                    sort_order: i,
                }))

            const payload = {
                patrol_date: patrolDate,
                accompanying_personnel: accompanyingPersonnel.trim(),
                entries: allEntries,
            }

            // R39 Phase 5：backend POST/PUT 直接回 VetPatrolReportWithEntries（含
            // 含 server-assigned entry id），auto-save 從 3 round trips 簡化為 1。
            if (savedReportId) {
                const res = await api.put<PatrolReport>(`/vet-patrol-reports/${savedReportId}`, payload)
                return res.data
            }
            const res = await api.post<PatrolReport>('/vet-patrol-reports', payload)
            return res.data
        },
        onSuccess: (report) => {
            const { id, entries: serverEntries } = report
            setSavedReportId(id)
            // 同步 server-side 權限相關狀態，否則 isVet 計算會把剛建好 draft 的獸醫
            // 誤判為非獸醫 → isReadOnly=true → dialog 全鎖死，使用者誤認為被自動送出。
            setReportStatus(report.status)
            setReportCreatedBy(report.created_by)
            setReportFollowUpUserId(report.follow_up_user_id)
            // 用 server 回傳的 entry id 回填本地 state，讓新列拿到 UUID 以利後續 entry photo 上傳。
            //
            // **配對策略**：在同一 category 內，第 N 個尚無 id 的本地列，配給第 N 個尚未被
            //   配對的 server 列。比依「內容相等」find() 更穩定（兩個空白列不會誤配同一筆 server entry）。
            // **無變動短路**：若沒有任何 id 變更則 return prev，避免 setEntries 觸發重新 render →
            //   debounce effect 重觸發 → 無限 PUT 迴圈。
            setEntries(prev => {
                let changed = false
                const next: Record<CategoryKey, EntryRow[]> = {
                    pig_condition: [], epidemic_prevention: [], case_record: [], other: [],
                }
                for (const cat of CATEGORIES) {
                    const localRows = prev[cat.key]
                    // 同 category 內未被 local id 配對的 server 列（按 sort_order 排序，定義穩定順序）
                    const usedServerIds = new Set(localRows.filter(r => r.id).map(r => r.id))
                    const unmatchedServer = serverEntries
                        .filter(s => s.category === cat.key && !usedServerIds.has(s.id))
                        .sort((a, b) => a.sort_order - b.sort_order)
                    let unmatchedIdx = 0
                    next[cat.key] = localRows.map(local => {
                        if (local.id) return local
                        const match = unmatchedServer[unmatchedIdx]
                        unmatchedIdx += 1
                        if (match) {
                            changed = true
                            return { ...local, id: match.id, tempKey: match.id }
                        }
                        return local
                    })
                }
                return changed ? next : prev
            })
            // R39++++ catch-up：若 mutation 進行中又被 addRow 觸發過，本次完成後
            // 補打一次 PUT，把那些列同步到 server（拿到 entry id）。
            if (pendingAddRowRef.current) {
                pendingAddRowRef.current = false
                setTimeout(() => upsertMutation.mutate(), 0)
            }
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '儲存失敗'), variant: 'destructive' })
        },
    })

    // ── auto-save 已移除（R39+++）：改為明確「存草稿」按鈕 ──
    useEffect(() => {
        if (!scrollToNewRowRef.current) return
        const key = scrollToNewRowRef.current
        scrollToNewRowRef.current = null
        requestAnimationFrame(() => {
            const el = document.querySelector(`[data-temp-key="${key}"]`)
            el?.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
        })
    })

    // 保留 debounceRef 變數讓既有的 submitForFollowupMutation 取消邏輯不會壞
    const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)
    // 維持 useEffect signature 但不做事，以利後續若想加 dirty marker 等
    useEffect(() => {
        if (!open) return
        return () => {
            if (debounceRef.current) clearTimeout(debounceRef.current)
        }
        // 故意不把 mutation 放 deps（reference 不穩）
    }, [patrolDate, accompanyingPersonnel, entries, open, hasInteracted])

    // R39+++ 存草稿：明確按鈕觸發 PUT，無 auto-save
    const saveDraftMutation = useMutation({
        mutationFn: async () => {
            await upsertMutation.mutateAsync()
        },
        onSuccess: () => {
            setCommittedAnyChange(true)
            setHasUnsavedTextChanges(false)
            queryClient.invalidateQueries({ queryKey: ['vet-patrol-reports'] })
            toast({ title: '已存草稿', description: '可繼續編輯或之後再送出' })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '存草稿失敗'), variant: 'destructive' })
        },
    })

    // R39+++ 棄置草稿：關閉 dialog 且未 commit 過任何動作時呼叫
    const discardDraftMutation = useMutation({
        mutationFn: async (id: string) => {
            await api.post(`/vet-patrol-reports/${id}/discard`)
        },
    })

    // ── R39+ 兩階段送出 ──
    // 階段 1：獸醫 draft → awaiting_follow_up，指派追蹤者
    const submitForFollowupMutation = useMutation({
        mutationFn: async () => {
            // 取消待執行的 auto-save debounce，避免送出後 800ms 又補一個 upsert（重複請求）
            if (debounceRef.current) {
                clearTimeout(debounceRef.current)
                debounceRef.current = null
            }
            const report = await upsertMutation.mutateAsync()
            await api.post(`/vet-patrol-reports/${report.id}/submit-for-followup`, {
                follow_up_user_id: followUpUserId,
            })
        },
        onSuccess: () => {
            setCommittedAnyChange(true)
            setHasUnsavedTextChanges(false)
            queryClient.invalidateQueries({ queryKey: ['vet-patrol-reports'] })
            queryClient.invalidateQueries({ queryKey: ['animal-vet-advice-records'] })
            toast({ title: '成功', description: '已送出給追蹤者，已通知對方' })
            onOpenChange(false)
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '送出失敗'), variant: 'destructive' })
        },
    })

    // 階段 2：追蹤者 awaiting_acknowledgement → awaiting_follow_up（按「確認收到」）
    // per-report-id guard：每份報告只自動觸發一次 auto-ack（避免 StrictMode / 重渲染重入）。
    // 宣告於 mutation 之前，讓 onError 能在失敗時重置（暫態錯誤可於重開 dialog 時重試）。
    const autoAckedIdRef = useRef<string | null>(null)
    const acknowledgeMutation = useMutation({
        mutationFn: async () => {
            if (!savedReportId) return
            await api.post(`/vet-patrol-reports/${savedReportId}/acknowledge`)
        },
        onSuccess: () => {
            setCommittedAnyChange(true)
            queryClient.invalidateQueries({ queryKey: ['vet-patrol-reports'] })
            queryClient.invalidateQueries({ queryKey: ['vet-patrol-report', savedReportId] })
            // 留在 dialog 內，狀態會切到 awaiting_follow_up，追蹤者可繼續填回覆內容
            setReportStatus('awaiting_follow_up')
            // 自動與手動觸發共用此 mutation，文案保持中性
            toast({ title: '已確認收到', description: '可直接填寫追蹤改善欄位，完成後按「確認完成」' })
        },
        onError: (error: unknown) => {
            // 自動確認失敗 → 重置 guard，讓使用者重開 dialog 時可重試（或改用 fallback 按鈕）
            autoAckedIdRef.current = null
            toast({ title: '錯誤', description: getApiErrorMessage(error, '確認失敗'), variant: 'destructive' })
        },
    })
    // mutate 由 React Query 保證 referentially stable，供下方 auto-ack effect 安全放入 deps
    const acknowledge = acknowledgeMutation.mutate

    // Option A：追蹤者打開 awaiting_acknowledgement 報告時「自動確認收到」，
    // 保留 acknowledged_at / audit，省去手動按一次（手動按鈕保留為 fallback）。
    useEffect(() => {
        if (
            isAwaitingAcknowledgement &&
            isFollowUpTracker &&
            savedReportId &&
            autoAckedIdRef.current !== savedReportId
        ) {
            autoAckedIdRef.current = savedReportId
            acknowledge()
        }
    }, [isAwaitingAcknowledgement, isFollowUpTracker, savedReportId, acknowledge])

    // 階段 3：追蹤者 awaiting_follow_up → completed（鎖定）
    const completeFollowupMutation = useMutation({
        mutationFn: async () => {
            // 同 submitForFollowup：先取消 debounce
            if (debounceRef.current) {
                clearTimeout(debounceRef.current)
                debounceRef.current = null
            }
            const report = await upsertMutation.mutateAsync()
            await api.post(`/vet-patrol-reports/${report.id}/complete-followup`)
        },
        onSuccess: () => {
            setCommittedAnyChange(true)
            setHasUnsavedTextChanges(false)
            queryClient.invalidateQueries({ queryKey: ['vet-patrol-reports'] })
            queryClient.invalidateQueries({ queryKey: ['animal-vet-advice-records'] })
            toast({ title: '成功', description: '追蹤已完成，報告已鎖定' })
            onOpenChange(false)
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '完成追蹤失敗'), variant: 'destructive' })
        },
    })

    // ── 照片 / PDF：抽為獨立 sub-hooks（R82-7）──
    const reportPhotos = usePatrolReportPhotos({ savedReportId, open, onCommitted: markCommitted })
    const entryPhotos = usePatrolEntryPhotos({ savedReportId, open, onCommitted: markCommitted })
    const pdf = usePatrolPdfExport({ savedReportId, patrolDate })

    /**
     * R39+++ 關閉 dialog 處理器：
     *  - 預建草稿 + 完全沒 commit → 直接 hard delete（連同上傳的照片）
     *  - 有未存文字變更 → confirm 確認是否丟棄
     *  - 其他狀況直接關
     */
    const handleClose = useCallback(() => {
        const isPreCreatedUncommitted =
            wasPreCreatedRef.current && !committedAnyChange && !!savedReportId
        if (isPreCreatedUncommitted) {
            // 預建但沒按存草稿就關 → discard（含照片）
            const idToDiscard = savedReportId
            discardDraftMutation.mutate(idToDiscard, {
                onSettled: () => {
                    queryClient.invalidateQueries({ queryKey: ['vet-patrol-reports'] })
                    onOpenChange(false)
                },
            })
            return
        }
        if (hasUnsavedTextChanges) {
            if (!window.confirm('有未儲存的變更，關閉後會遺失。確定關閉？')) return
        }
        onOpenChange(false)
    }, [committedAnyChange, hasUnsavedTextChanges, savedReportId, discardDraftMutation, onOpenChange, queryClient])

    const markInteracted = () => {
        if (!hasInteracted) setHasInteracted(true)
    }

    const addRow = (cat: CategoryKey) => {
        markInteracted()
        const newEntry = emptyEntry(cat)
        scrollToNewRowRef.current = newEntry.tempKey
        setEntries(prev => ({ ...prev, [cat]: [...prev[cat], newEntry] }))
        // R39++++ 新增列即觸發 PUT：使用者一新增空白列就拿到 server-assigned
        // entry id，可立即上傳該列照片，不必先輸入文字再按「存草稿」。
        // setTimeout(0) 確保 setEntries 已 commit、mutationFn 讀到的 entries
        // 是含新列的最新版本。pending 中改設 ref，由 onSuccess 觸發 catch-up
        // PUT，避免 spam-click 時後續列無 id。
        if (upsertMutation.isPending) {
            pendingAddRowRef.current = true
        } else {
            setTimeout(() => upsertMutation.mutate(), 0)
        }
    }

    const removeRow = (cat: CategoryKey, idx: number) => {
        markInteracted()
        setEntries(prev => ({
            ...prev,
            [cat]: prev[cat].length > 1
                ? prev[cat].filter((_, i) => i !== idx)
                : [emptyEntry(cat)],
        }))
    }

    const updateRow = (cat: CategoryKey, idx: number, field: keyof EntryRow, value: string) => {
        markInteracted()
        setHasUnsavedTextChanges(true)
        setEntries(prev => ({
            ...prev,
            [cat]: prev[cat].map((row, i) => i === idx ? { ...row, [field]: value } : row),
        }))
    }

    /** R39+++ 多動物：替換 animal_ids 整個陣列 */
    const setAnimalIds = (cat: CategoryKey, idx: number, ids: string[]) => {
        markInteracted()
        setHasUnsavedTextChanges(true)
        setEntries(prev => ({
            ...prev,
            [cat]: prev[cat].map((row, i) => i === idx ? { ...row, animal_ids: ids } : row),
        }))
    }

    const toggleCategory = (cat: string) => {
        setCollapsedCategories(prev => {
            const next = new Set(prev)
            if (next.has(cat)) next.delete(cat)
            else next.add(cat)
            return next
        })
    }

    // 計算「至少有 1 筆有內容的 entry」決定是否能送出
    const hasContent = useMemo(() => {
        return Object.values(entries).flat().some(e =>
            e.observation.trim() || e.suggestion.trim() || e.follow_up.trim() || e.animal_ids.length > 0
        )
    }, [entries])

    const isSaving = upsertMutation.isPending
    const isSubmitting =
        submitForFollowupMutation.isPending || acknowledgeMutation.isPending || completeFollowupMutation.isPending

    return {
        // props passthrough
        open,
        onOpenChange,
        // 基本資訊
        patrolDate,
        setPatrolDate,
        accompanyingPersonnel,
        setAccompanyingPersonnel,
        followUpUserId,
        setFollowUpUserId,
        staffList,
        staffOptions,
        animalOptions,
        // entries
        entries,
        collapsedCategories,
        // 角色 / 狀態
        isReadOnly,
        canEditFollowUpOnly,
        isCompleted,
        isAwaitingAcknowledgement,
        isAwaitingFollowUp,
        isDraft,
        isVet,
        isFollowUpTracker,
        savedReportId,
        hasContent,
        isSaving,
        isSubmitting,
        // handlers
        handleClose,
        markInteracted,
        setHasUnsavedTextChanges,
        addRow,
        removeRow,
        updateRow,
        setAnimalIds,
        toggleCategory,
        // mutations
        saveDraftMutation,
        submitForFollowupMutation,
        acknowledgeMutation,
        completeFollowupMutation,
        // sub-hooks
        ...reportPhotos,
        ...entryPhotos,
        ...pdf,
    }
}

export type VetPatrolReportVM = ReturnType<typeof useVetPatrolReport>
