import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'

import api, { Animal, AnimalSource, ProtocolListItem, facilityApi } from '@/lib/api'
import { getApiErrorMessage, animalEditSchema, type AnimalEditFormData } from '@/lib/validation'
import { toast } from '@/components/ui/use-toast'

export function useAnimalEdit(animalId: string) {
    const navigate = useNavigate()
    const queryClient = useQueryClient()

    const form = useForm<AnimalEditFormData>({
        resolver: zodResolver(animalEditSchema),
        defaultValues: { status: '', pen_location: '', iacuc_no: '', experiment_date: '', remark: '' },
    })

    const { data: animal, isLoading: animalLoading } = useQuery({
        queryKey: ['animal', animalId],
        queryFn: async () => (await api.get<Animal>(`/animals/${animalId}`)).data,
        staleTime: 5 * 60_000,
    })

    const { data: sources } = useQuery({
        queryKey: ['animal-sources'],
        queryFn: async () => (await api.get<AnimalSource[]>('/animal-sources')).data,
        staleTime: 600_000,
    })

    const { data: pens } = useQuery({
        queryKey: ['pens'],
        queryFn: async () => (await facilityApi.listPens()).data,
        staleTime: 600_000,
    })

    const { data: approvedProtocols } = useQuery({
        queryKey: ['approved-protocols'],
        queryFn: async () => {
            const res = await api.get<ProtocolListItem[]>('/protocols')
            return res.data.filter(p => {
                if (p.status === 'CLOSED') return false
                return (p.status === 'APPROVED' || p.status === 'APPROVED_WITH_CONDITIONS') && p.iacuc_no
            })
        },
    })

    useEffect(() => {
        if (animal) {
            form.reset({
                status: animal.status,
                pen_location: animal.pen_location || '',
                iacuc_no: animal.iacuc_no || '',
                experiment_date: animal.experiment_date || '',
                remark: animal.remark || '',
            })
        }
    }, [animal, form])

    const updateMutation = useMutation({
        mutationFn: async (data: AnimalEditFormData) => {
            return api.put(`/animals/${animalId}`, {
                status: data.status || undefined,
                pen_location: data.pen_location || undefined,
                iacuc_no: data.iacuc_no || undefined,
                experiment_date: data.experiment_date || undefined,
                remark: data.remark || undefined,
                // R30-B: optimistic lock — 從 query 結果取當前 version 防 lost update
                version: animal?.version,
            })
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['animal', animalId] })
            queryClient.invalidateQueries({ queryKey: ['animals'] })
            toast({ title: '成功', description: '動物資料已更新' })
            navigate(`/animals/${animalId}`)
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '更新失敗'), variant: 'destructive' })
        },
    })

    return {
        form,
        animal,
        animalLoading,
        sources,
        pens,
        approvedProtocols,
        updateMutation,
        navigate,
        queryClient,
    }
}
