import { useState, useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'

import api, { AnimalSacrifice, signatureApi } from '@/lib/api'
import type { SignatureData } from '@/components/ui/handwritten-signature-pad'
import type { FileInfo } from '@/components/ui/file-upload'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'

export interface SacrificeFormData {
    sacrifice_date: string
    zoletil_dose: string
    method_electrocution: boolean
    method_bloodletting: boolean
    method_other: string
    method_other_enabled: boolean
    sampling: string[]
    sampling_other: string
    blood_volume_ml: string
    confirmed_sacrifice: boolean
    photos: FileInfo[]
}

const defaultFormData: SacrificeFormData = {
    sacrifice_date: new Date().toISOString().split('T')[0],
    zoletil_dose: '',
    method_electrocution: false,
    method_bloodletting: false,
    method_other: '',
    method_other_enabled: false,
    sampling: [],
    sampling_other: '',
    blood_volume_ml: '',
    confirmed_sacrifice: false,
    photos: [],
}

interface UseSacrificeFormOptions {
    open: boolean
    animalId: string
    sacrifice?: AnimalSacrifice
    onOpenChange: (open: boolean) => void
}

export function useSacrificeForm({ open, animalId, sacrifice, onOpenChange }: UseSacrificeFormOptions) {
    const { t } = useTranslation()
    const queryClient = useQueryClient()
    const isEdit = !!sacrifice
    const [formData, setFormData] = useState<SacrificeFormData>(defaultFormData)
    const [signatureData, setSignatureData] = useState<SignatureData | null>(null)

    useEffect(() => {
        if (sacrifice) {
            const samplingArray = sacrifice.sampling
                ? sacrifice.sampling.split(',').map((s) => s.trim()).filter(Boolean)
                : []
            setFormData({
                sacrifice_date: sacrifice.sacrifice_date ? sacrifice.sacrifice_date.split('T')[0] : new Date().toISOString().split('T')[0],
                zoletil_dose: sacrifice.zoletil_dose || '',
                method_electrocution: sacrifice.method_electrocution,
                method_bloodletting: sacrifice.method_bloodletting,
                method_other: sacrifice.method_other || '',
                method_other_enabled: !!sacrifice.method_other,
                sampling: samplingArray,
                sampling_other: sacrifice.sampling_other || '',
                blood_volume_ml: sacrifice.blood_volume_ml?.toString() || '',
                confirmed_sacrifice: sacrifice.confirmed_sacrifice,
                photos: [],
            })
        } else {
            setFormData(defaultFormData)
        }
    }, [sacrifice, open])

    const handleSamplingChange = (value: string, checked: boolean) => {
        setFormData((prev) => ({
            ...prev,
            sampling: checked ? [...prev.sampling, value] : prev.sampling.filter((s) => s !== value),
        }))
    }

    const handlePhotoUpload = async (file: File): Promise<FileInfo> => {
        const fd = new FormData()
        fd.append('file', file)
        const response = await api.post<{
            id: string; file_name: string; file_path: string; file_size: number; mime_type: string
        }>(`/animals/${animalId}/sacrifice/photos`, fd, {
            headers: { 'Content-Type': 'multipart/form-data' },
        })
        return {
            id: response.data.id,
            file_name: response.data.file_name,
            file_path: response.data.file_path,
            file_size: response.data.file_size,
            file_type: response.data.mime_type,
            preview_url: response.data.file_path.startsWith('http')
                ? response.data.file_path
                : `/api/files/${response.data.file_path}`,
        }
    }

    const mutation = useMutation({
        mutationFn: async (data: SacrificeFormData) => {
            const payload = {
                sacrifice_date: data.sacrifice_date || null,
                zoletil_dose: data.zoletil_dose || null,
                method_electrocution: data.method_electrocution,
                method_bloodletting: data.method_bloodletting,
                method_other: data.method_other || null,
                sampling: data.sampling.length > 0 ? data.sampling.join(',') : null,
                sampling_other: data.sampling_other || null,
                blood_volume_ml: data.blood_volume_ml ? parseFloat(data.blood_volume_ml) : null,
                confirmed_sacrifice: data.confirmed_sacrifice,
            }
            return api.post(`/animals/${animalId}/sacrifice`, payload)
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal-sacrifice', animalId] })
            queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
            toast({ title: '成功', description: isEdit ? '犧牲紀錄已更新' : '犧牲紀錄已建立' })
            onOpenChange(false)
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '儲存失敗'), variant: 'destructive' })
        },
    })

    const signMutation = useMutation({
        mutationFn: async (sigData: SignatureData) => {
            if (!sacrifice?.id) return
            return signatureApi.signSacrifice(sacrifice.id, {
                handwriting_svg: sigData.svg,
                stroke_data: sigData.strokeData,
            })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['sacrifice-signature', sacrifice?.id] })
            toast({ title: t('signature.signed', '已簽署'), description: t('signature.signSuccess', '簽章完成') })
        },
        onError: (error: unknown) => {
            toast({
                title: t('common.error', '錯誤'),
                description: getApiErrorMessage(error, t('signature.signFailed', '簽章失敗')),
                variant: 'destructive',
            })
        },
    })

    const { data: signatureStatus } = useQuery({
        queryKey: ['sacrifice-signature', sacrifice?.id],
        queryFn: () => signatureApi.getSacrificeStatus(sacrifice!.id),
        enabled: isEdit && !!sacrifice?.id,
        select: (res) => res.data,
        staleTime: 30_000,
        retry: false,
    })

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        mutation.mutate(formData)
    }

    return {
        t, formData, setFormData, isEdit, mutation, signMutation,
        signatureData, setSignatureData, signatureStatus,
        handleSamplingChange, handlePhotoUpload, handleSubmit,
        hasOtherSampling: formData.sampling.includes('其他'),
    }
}
