import { useRef } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Loader2, Paperclip, Upload } from 'lucide-react'

import api from '@/lib/api'
import { getApiErrorMessage } from '@/lib/apiError'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'

/** 補登：倫理委員會主席核准同意函——上傳紙本掃描檔（沿用既有附件機制）+ 顯示已上傳清單 */
export function ChairmanLetterUpload({ protocolId }: { protocolId: string }) {
  const queryClient = useQueryClient()
  const fileInputRef = useRef<HTMLInputElement>(null)

  const { data: attachments = [] } = useQuery({
    queryKey: ['protocol-attachments', protocolId],
    queryFn: async () =>
      (await api.get<{ id: string; file_name: string }[]>(`/protocols/${protocolId}/attachments`)).data,
    enabled: !!protocolId,
  })

  const uploadMutation = useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData()
      formData.append('file', file)
      // Content-Type 交給 axios 依 FormData 自動帶 boundary（與 AttachmentsTab 一致）
      return api.post(`/protocols/${protocolId}/attachments`, formData)
    },
    onSuccess: () => {
      toast({ title: '成功', description: '已上傳主席核准同意函' })
      queryClient.invalidateQueries({ queryKey: ['protocol-attachments', protocolId] })
    },
    onError: (err: unknown) =>
      toast({ title: '錯誤', description: getApiErrorMessage(err, '上傳失敗'), variant: 'destructive' }),
  })

  const handleFile = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) uploadMutation.mutate(file)
    if (fileInputRef.current) fileInputRef.current.value = ''
  }

  return (
    <Card>
      <CardHeader><CardTitle className="text-base">3. 倫理委員會主席核准同意函</CardTitle></CardHeader>
      <CardContent className="space-y-3">
        <div className="flex items-center justify-between">
          <p className="text-sm text-muted-foreground">上傳紙本掃描檔（PDF / 圖片）作為附件。</p>
          <input ref={fileInputRef} type="file" className="hidden" onChange={handleFile} accept=".pdf,.png,.jpg,.jpeg" />
          <Button variant="outline" disabled={uploadMutation.isPending} onClick={() => fileInputRef.current?.click()}>
            {uploadMutation.isPending ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Upload className="h-4 w-4 mr-2" />}
            上傳同意函
          </Button>
        </div>
        {attachments.length > 0 && (
          <ul className="space-y-1 text-sm text-muted-foreground">
            {attachments.map((a) => (
              <li key={a.id} className="flex items-center gap-2">
                <Paperclip className="h-3.5 w-3.5 shrink-0" />{a.file_name}
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  )
}
