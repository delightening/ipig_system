// Report-level 整體環境照片邏輯（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { compressImage } from '@/lib/imageCompress'
import type { PatrolPhoto } from './types'

export function usePatrolReportPhotos({
    savedReportId,
    open,
    onCommitted,
}: {
    savedReportId: string | null
    open: boolean
    onCommitted: () => void
}) {
    const queryClient = useQueryClient()
    const photosQueryKey = ['vet-patrol-photos', savedReportId]

    const { data: photos = [] } = useQuery({
        queryKey: photosQueryKey,
        queryFn: async () => {
            const res = await api.get<PatrolPhoto[]>(`/vet-patrol-reports/${savedReportId}/photos`)
            return res.data
        },
        enabled: !!savedReportId && open,
    })

    const uploadReportPhotoMutation = useMutation({
        mutationFn: async ({ file, caption }: { file: File; caption: string }) => {
            const compressed = await compressImage(file)
            const fd = new FormData()
            fd.append('file', compressed)
            fd.append('caption', caption)
            const res = await api.post<PatrolPhoto>(
                `/vet-patrol-reports/${savedReportId}/photos`,
                fd,
                { headers: { 'Content-Type': 'multipart/form-data' } },
            )
            return res.data
        },
        onSuccess: () => {
            onCommitted()
            queryClient.invalidateQueries({ queryKey: photosQueryKey })
            toast({ title: '成功', description: '照片已上傳' })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '照片上傳失敗'), variant: 'destructive' })
        },
    })

    const updateReportCaptionMutation = useMutation({
        mutationFn: async ({ photoId, caption }: { photoId: string; caption: string }) => {
            await api.put(`/vet-patrol-photos/${photoId}`, { caption })
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: photosQueryKey }),
    })

    const deleteReportPhotoMutation = useMutation({
        mutationFn: async (photoId: string) => {
            await api.delete(`/vet-patrol-photos/${photoId}`)
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: photosQueryKey }),
    })

    const handleSelectReportPhoto = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const fileList = e.target.files
        if (!fileList) return
        for (const file of Array.from(fileList)) {
            uploadReportPhotoMutation.mutate({ file, caption: '' })
        }
        e.target.value = ''
    }

    return {
        photos,
        uploadReportPhotoMutation,
        updateReportCaptionMutation,
        deleteReportPhotoMutation,
        handleSelectReportPhoto,
    }
}
