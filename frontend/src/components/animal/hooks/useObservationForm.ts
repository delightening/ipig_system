import { useState, useEffect, useRef, useCallback } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'

import api, { AnimalObservation, RecordType } from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import type { FileInfo } from '@/components/ui/file-upload'

export interface TreatmentItem {
    drug: string
    dosage: string
    end_date?: string
    drug_option_id?: string
    dosage_unit?: string
}

export interface ObservationFormData {
    event_date: string
    record_type: RecordType
    equipment_used: string[]
    anesthesia_start: string
    anesthesia_end: string
    content: string
    no_medication_needed: boolean
    treatments: TreatmentItem[]
    remark: string
    photos: FileInfo[]
    attachments: FileInfo[]
}

const defaultFormData: ObservationFormData = {
    event_date: new Date().toISOString().split('T')[0],
    record_type: 'observation',
    equipment_used: [],
    anesthesia_start: '',
    anesthesia_end: '',
    content: '',
    no_medication_needed: false,
    treatments: [],
    remark: '',
    photos: [],
    attachments: [],
}

function isoToDateTimeLocal(isoString: string | undefined): string {
    if (!isoString) return ''
    try {
        const date = new Date(isoString)
        if (isNaN(date.getTime())) return ''
        const year = date.getFullYear()
        const month = String(date.getMonth() + 1).padStart(2, '0')
        const day = String(date.getDate()).padStart(2, '0')
        const hours = String(date.getHours()).padStart(2, '0')
        const minutes = String(date.getMinutes()).padStart(2, '0')
        return `${year}-${month}-${day}T${hours}:${minutes}`
    } catch {
        return ''
    }
}

function dateTimeLocalToISO(datetimeLocal: string | null | undefined): string | null {
    if (!datetimeLocal || datetimeLocal.trim() === '') return null
    try {
        const date = new Date(datetimeLocal)
        if (isNaN(date.getTime())) return null
        return date.toISOString()
    } catch {
        return null
    }
}

interface UseObservationFormOptions {
    open: boolean
    animalId: string
    observation?: AnimalObservation
    onOpenChange: (open: boolean) => void
}

export function useObservationForm({ open, animalId, observation, onOpenChange }: UseObservationFormOptions) {
    const queryClient = useQueryClient()
    const isEdit = !!observation
    const [formData, setFormData] = useState<ObservationFormData>(defaultFormData)
    const pendingFilesRef = useRef<Map<string, File>>(new Map())

    useEffect(() => {
        if (observation) {
            setFormData({
                event_date: observation.event_date.split('T')[0],
                record_type: observation.record_type,
                equipment_used: observation.equipment_used || [],
                anesthesia_start: isoToDateTimeLocal(observation.anesthesia_start),
                anesthesia_end: isoToDateTimeLocal(observation.anesthesia_end),
                content: observation.content,
                no_medication_needed: observation.no_medication_needed,
                treatments: observation.treatments || [],
                remark: observation.remark || '',
                photos: [],
                attachments: [],
            })
        } else {
            setFormData(defaultFormData)
        }
    // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [observation, open])

    const handleEquipmentChange = (value: string, checked: boolean) => {
        setFormData((prev) => ({
            ...prev,
            equipment_used: checked
                ? [...prev.equipment_used, value]
                : prev.equipment_used.filter((e) => e !== value),
        }))
    }

    const uploadFilesToObservation = async (observationId: string) => {
        const files = Array.from(pendingFilesRef.current.values())
        if (files.length === 0) return
        const fd = new FormData()
        for (const file of files) fd.append('file', file)
        await api.post(`/observations/${observationId}/attachments`, fd, {
            headers: { 'Content-Type': 'multipart/form-data' },
        })
    }

    const handlePhotoUpload = async (file: File): Promise<FileInfo> => {
        if (!isEdit) {
            const id = `local-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
            pendingFilesRef.current.set(id, file)
            return {
                id,
                file_name: file.name,
                file_path: '',
                file_size: file.size,
                file_type: file.type,
                preview_url: file.type.startsWith('image/') ? URL.createObjectURL(file) : undefined,
            }
        }
        const fd = new FormData()
        fd.append('file', file)
        const res = await api.post(`/observations/${observation!.id}/attachments`, fd, {
            headers: { 'Content-Type': 'multipart/form-data' },
        })
        const uploaded = res.data[0]
        return { id: uploaded.id, file_name: uploaded.file_name, file_path: uploaded.file_path, file_size: uploaded.file_size }
    }

    const mutation = useMutation({
        mutationFn: async (data: ObservationFormData) => {
            const payload = {
                event_date: data.event_date,
                record_type: data.record_type,
                equipment_used: data.equipment_used.length > 0 ? data.equipment_used : null,
                anesthesia_start: dateTimeLocalToISO(data.anesthesia_start),
                anesthesia_end: dateTimeLocalToISO(data.anesthesia_end),
                content: data.content,
                no_medication_needed: data.no_medication_needed,
                treatments: data.treatments.length > 0 ? data.treatments : null,
                remark: data.remark || null,
            }
            if (isEdit) return api.put(`/observations/${observation.id}`, payload)
            return api.post(`/animals/${animalId}/observations`, payload)
        },
        onSuccess: async (response) => {
            try {
                if (!isEdit && pendingFilesRef.current.size > 0) {
                    const newId = response.data?.id
                    if (newId) await uploadFilesToObservation(newId)
                }
            } catch {
                toast({ title: '警告', description: '紀錄已儲存，但部分檔案上傳失敗', variant: 'destructive' })
            }
            pendingFilesRef.current.clear()
            queryClient.invalidateQueries({ queryKey: ['animal-observations', animalId] })
            toast({ title: '成功', description: isEdit ? '觀察紀錄已更新' : '觀察紀錄已新增' })
            onOpenChange(false)
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '儲存失敗'), variant: 'destructive' })
        },
    })

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (!formData.content.trim()) {
            toast({ title: '錯誤', description: '請填寫內容', variant: 'destructive' })
            return
        }
        mutation.mutate(formData)
    }

    const jumpToNextEmptyField = useCallback(() => {
        const fields = [
            { id: 'event_date', value: formData.event_date },
            { id: 'anesthesia_start', value: formData.anesthesia_start },
            { id: 'anesthesia_end', value: formData.anesthesia_end },
            { id: 'content', value: formData.content },
            { id: 'remark', value: formData.remark },
        ]
        const nextEmpty = fields.find(f => !f.value || f.value.trim() === '')
        if (nextEmpty) {
            const element = document.getElementById(nextEmpty.id)
            if (element) {
                element.focus()
                toast({ title: '已跳轉', description: '跳轉至下一個空白欄位', duration: 2000 })
                return
            }
        }
        toast({ title: '完成', description: '所有主要欄位皆已填寫', duration: 2000 })
    }, [formData])

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.altKey && e.key === 'n') {
                e.preventDefault()
                jumpToNextEmptyField()
            }
        }
        window.addEventListener('keydown', handleKeyDown)
        return () => window.removeEventListener('keydown', handleKeyDown)
    }, [jumpToNextEmptyField])

    return {
        formData, setFormData, isEdit, mutation,
        handleEquipmentChange, handlePhotoUpload, handleSubmit, jumpToNextEmptyField,
    }
}
