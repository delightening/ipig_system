import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { VetRecommendation } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/input'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import {
  Loader2,
  MessageCircle,
  Plus,
  User,
  Clock,
  FileText,
  Stethoscope,
} from 'lucide-react'

type RecordType = 'observation' | 'surgery'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  recordType: RecordType
  recordId: number
  pigEarTag: string
}

const recordTypeNames: Record<RecordType, string> = {
  observation: '觀察試驗紀錄',
  surgery: '手術紀錄',
}

export function VetRecommendationDialog({ open, onOpenChange, recordType, recordId, pigEarTag }: Props) {
  const queryClient = useQueryClient()
  const [showAddForm, setShowAddForm] = useState(false)
  const [content, setContent] = useState('')
  const [attachments, setAttachments] = useState<FileInfo[]>([])

  // Query recommendations
  const { data: recommendations, isLoading } = useQuery({
    queryKey: ['vet-recommendations', recordType, recordId],
    queryFn: async () => {
      const endpoint = recordType === 'observation'
        ? `/observations/${recordId}/recommendations`
        : `/surgeries/${recordId}/recommendations`
      const res = await api.get<VetRecommendation[]>(endpoint)
      return res.data
    },
    enabled: open,
  })

  // Add recommendation mutation
  const addMutation = useMutation({
    mutationFn: async () => {
      const endpoint = recordType === 'observation'
        ? `/observations/${recordId}/recommendations`
        : `/surgeries/${recordId}/recommendations`
      return api.post(endpoint, {
        content,
        // TODO: Handle file upload
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['vet-recommendations', recordType, recordId] })
      toast({ title: '成功', description: '獸醫師建議已新增' })
      setShowAddForm(false)
      setContent('')
      setAttachments([])
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '新增失敗',
        variant: 'destructive',
      })
    },
  })

  const formatDateTime = (dateStr: string) => {
    return new Date(dateStr).toLocaleString('zh-TW', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!content.trim()) {
      toast({ title: '錯誤', description: '請填寫建議內容', variant: 'destructive' })
      return
    }
    addMutation.mutate()
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Stethoscope className="h-5 w-5" />
            獸醫師建議
          </DialogTitle>
          <DialogDescription>
            {recordTypeNames[recordType]} - 耳號：{pigEarTag}
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto py-4">
          {/* Add Button */}
          {!showAddForm && (
            <div className="mb-4">
              <Button
                onClick={() => setShowAddForm(true)}
                className="gap-2 bg-green-600 hover:bg-green-700"
              >
                <Plus className="h-4 w-4" />
                新增建議
              </Button>
            </div>
          )}

          {/* Add Form */}
          {showAddForm && (
            <form onSubmit={handleSubmit} className="mb-6 p-4 bg-green-50 rounded-lg border border-green-200">
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="content">建議內容 *</Label>
                  <Textarea
                    id="content"
                    value={content}
                    onChange={(e) => setContent(e.target.value)}
                    placeholder="輸入獸醫師建議..."
                    className="min-h-[100px]"
                    required
                  />
                </div>

                <div className="space-y-2">
                  <Label>附件</Label>
                  <FileUpload
                    value={attachments}
                    onChange={setAttachments}
                    accept=".pdf,.doc,.docx,.jpg,.jpeg,.png"
                    placeholder="拖曳檔案到此處，或點擊選擇"
                    maxSize={10}
                    maxFiles={5}
                  />
                </div>

                <div className="flex justify-end gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    onClick={() => {
                      setShowAddForm(false)
                      setContent('')
                      setAttachments([])
                    }}
                  >
                    取消
                  </Button>
                  <Button
                    type="submit"
                    disabled={addMutation.isPending}
                    className="bg-green-600 hover:bg-green-700"
                  >
                    {addMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
                    送出建議
                  </Button>
                </div>
              </div>
            </form>
          )}

          {/* Recommendations List */}
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
            </div>
          ) : !recommendations || recommendations.length === 0 ? (
            <div className="text-center py-12 text-slate-500">
              <MessageCircle className="h-12 w-12 mx-auto mb-4 text-slate-300" />
              <p>尚無獸醫師建議</p>
            </div>
          ) : (
            <div className="space-y-4">
              {recommendations.map((rec) => (
                <div key={rec.id} className="p-4 bg-slate-50 rounded-lg border">
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex items-center gap-2 text-sm text-slate-600">
                      <User className="h-4 w-4" />
                      <span className="font-medium">{rec.created_by_name || '獸醫師'}</span>
                    </div>
                    <div className="flex items-center gap-1 text-sm text-slate-400">
                      <Clock className="h-4 w-4" />
                      <span>{formatDateTime(rec.created_at)}</span>
                    </div>
                  </div>
                  <p className="text-slate-700 whitespace-pre-wrap">{rec.content}</p>
                  
                  {/* Attachments */}
                  {rec.attachments && typeof rec.attachments === 'object' && Object.keys(rec.attachments).length > 0 && (
                    <div className="mt-3 pt-3 border-t">
                      <div className="flex items-center gap-1 text-sm text-slate-500 mb-2">
                        <FileText className="h-4 w-4" />
                        <span>附件</span>
                      </div>
                      <div className="flex flex-wrap gap-2">
                        {Object.entries(rec.attachments).map(([key, value]) => (
                          <Button
                            key={key}
                            variant="outline"
                            size="sm"
                            className="text-xs"
                            onClick={() => {
                              // TODO: Download attachment
                            }}
                          >
                            <FileText className="h-3 w-3 mr-1" />
                            {typeof value === 'string' ? value : key}
                          </Button>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="flex justify-end pt-4 border-t">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            關閉
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
