import { useEffect, useState } from 'react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import {
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
} from '@/components/ui/dialog'
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
} from '@/components/ui/select'
import { formatDate } from '@/lib/utils'
import type { AttendanceWithUser, StaffInfo } from '@/types/hr'

import { useAttendanceCorrection } from '../hooks/useAttendanceCorrection'
import { MIN_CORRECTION_REASON_LENGTH, isoToTaipeiTimeInput, taipeiTimeInputToIso } from '../attendanceTime'

interface AttendanceCorrectionDialogProps {
    open: boolean
    onOpenChange: (open: boolean) => void
    /** 有值＝更正既有紀錄；null＝補登缺漏日 */
    record: AttendanceWithUser | null
    /** 補登模式的人員下拉選項；更正模式用不到 */
    staffList?: StaffInfo[]
}

/** 台灣時區的今天（yyyy-MM-dd），供日期上限使用 */
function taipeiToday(): string {
    return new Date().toLocaleDateString('sv-SE', { timeZone: 'Asia/Taipei' })
}

/**
 * 補卡對話框：補登缺漏日（`record === null`）與更正既有紀錄（`record !== null`）共用。
 *
 * 兩種模式差在「日期與人員能不能改」——更正時那兩者由既有紀錄決定，不可改；
 * 補登時要先選人與日期。時間一律以台灣時間輸入，送出前換算成 UTC ISO
 *（後端 `attendance_records` 存的是 UTC，直接送本機時間會差 8 小時）。
 *
 * 「不得補自己的卡」不在此判斷：後端 `reject_self_correction` 是唯一判準，
 * 這裡只呈現它回的 403 訊息。前端另判一次，兩邊遲早會分歧。
 */
export function AttendanceCorrectionDialog({
    open, onOpenChange, record, staffList,
}: AttendanceCorrectionDialogProps) {
    const isCorrection = record !== null
    const { backfillMutation, correctMutation } = useAttendanceCorrection()

    const [userId, setUserId] = useState('')
    const [workDate, setWorkDate] = useState('')
    const [clockIn, setClockIn] = useState('')
    const [clockOut, setClockOut] = useState('')
    const [reason, setReason] = useState('')

    // 每次開啟時依模式重設表單：更正帶入既有值，補登清空。
    // deps 只放穩定值（record 的欄位而非 record 物件本身），避免父層每次 render
    // 產生新 reference 而把使用者正在打的字洗掉。
    const recordId = record?.id ?? ''
    const recordClockIn = record?.clock_in_time ?? null
    const recordClockOut = record?.clock_out_time ?? null
    useEffect(() => {
        if (!open) return
        setReason('')
        if (recordId) {
            setClockIn(isoToTaipeiTimeInput(recordClockIn))
            setClockOut(isoToTaipeiTimeInput(recordClockOut))
            return
        }
        setUserId('')
        setWorkDate('')
        setClockIn('')
        setClockOut('')
    }, [open, recordId, recordClockIn, recordClockOut])

    const trimmedReason = reason.trim()
    const reasonTooShort = trimmedReason.length < MIN_CORRECTION_REASON_LENGTH
    const missingTarget = !isCorrection && (!userId || !workDate)
    // 兩種模式都要求至少一個時間。更正模式若不擋，使用者可以把兩欄清空後送出
    // `clock_in_time: null, clock_out_time: null`——後端 COALESCE 回原值，等於什麼都沒改，
    // 卻蓋上 is_corrected / corrected_by / correction_reason，污染稽核軌跡與月報的
    //「補登／更正天數」（CodeRabbit PR #35 第三輪指出）
    const noTimeGiven = !clockIn && !clockOut
    const pending = backfillMutation.isPending || correctMutation.isPending
    const canSubmit = !pending && !reasonTooShort && !missingTarget && !noTimeGiven

    const handleSubmit = () => {
        if (!canSubmit) return
        if (isCorrection && record) {
            correctMutation.mutate(
                {
                    id: record.id,
                    clock_in_time: taipeiTimeInputToIso(record.work_date, clockIn),
                    clock_out_time: taipeiTimeInputToIso(record.work_date, clockOut),
                    reason: trimmedReason,
                },
                { onSuccess: () => onOpenChange(false) },
            )
            return
        }
        backfillMutation.mutate(
            {
                user_id: userId,
                work_date: workDate,
                clock_in_time: taipeiTimeInputToIso(workDate, clockIn),
                clock_out_time: taipeiTimeInputToIso(workDate, clockOut),
                reason: trimmedReason,
            },
            { onSuccess: () => onOpenChange(false) },
        )
    }

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent size="md">
                <DialogHeader>
                    <DialogTitle>{isCorrection ? '更正出勤記錄' : '補登出勤記錄'}</DialogTitle>
                    <DialogDescription>
                        {isCorrection
                            ? '修改既有紀錄的上下班時間。原始時間會保留於稽核紀錄，工時自動重算。'
                            : '為完全沒有打卡紀錄的日子補登。已有紀錄的日子請改用該列的「更正」。'}
                        {' '}不得補登或更正自己的紀錄。
                    </DialogDescription>
                </DialogHeader>

                <div className="space-y-4">
                    {isCorrection && record ? (
                        <div className="rounded-md border bg-muted/40 px-3 py-2 text-sm">
                            <div className="font-medium">{record.user_name}</div>
                            <div className="text-muted-foreground">
                                {formatDate(record.work_date, { weekday: true })}
                            </div>
                        </div>
                    ) : (
                        <>
                            <div className="flex flex-col gap-2">
                                <Label htmlFor="backfill-user">人員</Label>
                                <Select value={userId} onValueChange={setUserId}>
                                    <SelectTrigger id="backfill-user">
                                        <SelectValue placeholder="選擇人員" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {staffList?.map((s) => (
                                            <SelectItem key={s.id} value={s.id}>{s.display_name}</SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                            </div>
                            <div className="flex flex-col gap-2">
                                <Label htmlFor="backfill-date">日期</Label>
                                <Input
                                    id="backfill-date"
                                    type="date"
                                    max={taipeiToday()}
                                    value={workDate}
                                    onChange={(e) => setWorkDate(e.target.value)}
                                />
                            </div>
                        </>
                    )}

                    <div className="grid grid-cols-2 gap-3">
                        <div className="flex flex-col gap-2">
                            <Label htmlFor="clock-in">上班時間</Label>
                            <Input
                                id="clock-in"
                                type="time"
                                step={60}
                                value={clockIn}
                                onChange={(e) => setClockIn(e.target.value)}
                            />
                        </div>
                        <div className="flex flex-col gap-2">
                            <Label htmlFor="clock-out">下班時間</Label>
                            <Input
                                id="clock-out"
                                type="time"
                                step={60}
                                value={clockOut}
                                onChange={(e) => setClockOut(e.target.value)}
                            />
                        </div>
                    </div>
                    {noTimeGiven && (
                        <p className="text-sm text-muted-foreground">
                            上班與下班至少要填一個。
                        </p>
                    )}

                    <div className="flex flex-col gap-2">
                        <Label htmlFor="correction-reason">
                            理由（至少 {MIN_CORRECTION_REASON_LENGTH} 個字）
                        </Label>
                        <Textarea
                            id="correction-reason"
                            value={reason}
                            error={reason.length > 0 && reasonTooShort}
                            placeholder="例：忘記打卡、出差外訪未帶手機"
                            onChange={(e) => setReason(e.target.value)}
                        />
                    </div>
                </div>

                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)} disabled={pending}>
                        取消
                    </Button>
                    <Button onClick={handleSubmit} disabled={!canSubmit}>
                        {pending ? '送出中...' : isCorrection ? '確認更正' : '確認補登'}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
}
