// Entry-level 照片邏輯（R82-7 由 VetPatrolReportDialog.tsx 抽出）

import { useMemo } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api from '@/lib/api'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/apiError'
import { compressImage } from '@/lib/imageCompress'
import type { EntryPhoto, PatrolReport } from './types'

export function usePatrolEntryPhotos({
    savedReportId,
    open,
    onCommitted,
}: {
    savedReportId: string | null
    open: boolean
    onCommitted: () => void
}) {
    const queryClient = useQueryClient()
    // ── Entry-level 照片：以 Map<entry_id, EntryPhoto[]> 維護快取 ──
    const entryPhotosQueryKey = ['vet-patrol-entry-photos', savedReportId]

    const { data: allEntryPhotos = [] } = useQuery({
        queryKey: entryPhotosQueryKey,
        queryFn: async () => {
            const res = await api.get<PatrolReport>(`/vet-patrol-reports/${savedReportId}`)
            return res.data.entry_photos
        },
        enabled: !!savedReportId && open,
    })

    const entryPhotosByEntry = useMemo(() => {
        const map = new Map<string, EntryPhoto[]>()
        for (const p of allEntryPhotos) {
            const arr = map.get(p.entry_id) ?? []
            arr.push(p)
            map.set(p.entry_id, arr)
        }
        return map
    }, [allEntryPhotos])

    const uploadEntryPhotoMutation = useMutation({
        mutationFn: async ({ entryId, file }: { entryId: string; file: File }) => {
            const compressed = await compressImage(file)
            const fd = new FormData()
            fd.append('file', compressed)
            fd.append('caption', '')
            await api.post(
                `/vet-patrol-entries/${entryId}/photos`,
                fd,
                { headers: { 'Content-Type': 'multipart/form-data' } },
            )
        },
        onSuccess: () => {
            onCommitted()
            queryClient.invalidateQueries({ queryKey: entryPhotosQueryKey })
        },
        onError: (error: unknown) => {
            toast({ title: '錯誤', description: getApiErrorMessage(error, '照片上傳失敗'), variant: 'destructive' })
        },
    })

    const deleteEntryPhotoMutation = useMutation({
        mutationFn: async (photoId: string) => {
            await api.delete(`/vet-patrol-entry-photos/${photoId}`)
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: entryPhotosQueryKey }),
    })

    const updateEntryCaptionMutation = useMutation({
        mutationFn: async ({ photoId, caption }: { photoId: string; caption: string }) => {
            await api.put(`/vet-patrol-entry-photos/${photoId}`, { caption })
        },
        onSuccess: () => queryClient.invalidateQueries({ queryKey: entryPhotosQueryKey }),
    })

    const handleSelectEntryPhoto = async (
        entryId: string,
        e: React.ChangeEvent<HTMLInputElement>,
    ) => {
        const fileList = e.target.files
        if (!fileList) return
        for (const file of Array.from(fileList)) {
            uploadEntryPhotoMutation.mutate({ entryId, file })
        }
        e.target.value = ''
    }

    return {
        entryPhotosByEntry,
        uploadEntryPhotoMutation,
        deleteEntryPhotoMutation,
        updateEntryCaptionMutation,
        handleSelectEntryPhoto,
    }
}
