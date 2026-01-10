import { useQuery } from '@tanstack/react-query'
import api from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Loader2,
  History,
  Clock,
  User,
  ChevronRight,
} from 'lucide-react'

type RecordType = 'observation' | 'surgery'

interface VersionItem {
  id: number
  version_no: number
  record_snapshot: Record<string, any>
  changed_by_name: string
  created_at: string
}

interface VersionHistoryResponse {
  current_version: number
  versions: VersionItem[]
}

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
  recordType: RecordType
  recordId: number
}

const recordTypeNames: Record<RecordType, string> = {
  observation: '觀察試驗紀錄',
  surgery: '手術紀錄',
}

export function VersionHistoryDialog({ open, onOpenChange, recordType, recordId }: Props) {
  const { data, isLoading } = useQuery({
    queryKey: ['record-versions', recordType, recordId],
    queryFn: async () => {
      const endpoint = recordType === 'observation' 
        ? `/observations/${recordId}/versions`
        : `/surgeries/${recordId}/versions`
      const res = await api.get<VersionHistoryResponse>(endpoint)
      return res.data
    },
    enabled: open,
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

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-hidden flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <History className="h-5 w-5" />
            版本歷史
          </DialogTitle>
          <DialogDescription>
            {recordTypeNames[recordType]} - 檢視紀錄的歷史版本
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto py-4">
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
            </div>
          ) : !data || data.versions.length === 0 ? (
            <div className="text-center py-12 text-slate-500">
              <History className="h-12 w-12 mx-auto mb-4 text-slate-300" />
              <p>尚無版本歷史</p>
            </div>
          ) : (
            <div className="space-y-3">
              {/* Current Version */}
              <div className="flex items-center gap-2 px-3 py-2 bg-purple-50 rounded-lg">
                <Badge className="bg-purple-600">目前版本 v{data.current_version}</Badge>
              </div>

              {/* Version Timeline */}
              <div className="relative">
                {data.versions.map((version, index) => (
                  <div key={version.id} className="relative flex gap-4 pb-6 last:pb-0">
                    {/* Timeline Line */}
                    {index < data.versions.length - 1 && (
                      <div className="absolute left-[11px] top-6 bottom-0 w-0.5 bg-slate-200" />
                    )}

                    {/* Timeline Dot */}
                    <div className="relative z-10 flex h-6 w-6 items-center justify-center rounded-full bg-slate-200 ring-4 ring-white">
                      <span className="text-xs font-medium text-slate-600">{version.version_no}</span>
                    </div>

                    {/* Content */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between mb-2">
                        <div className="flex items-center gap-2 text-sm">
                          <Clock className="h-4 w-4 text-slate-400" />
                          <span className="text-slate-600">{formatDateTime(version.created_at)}</span>
                        </div>
                        <div className="flex items-center gap-1 text-sm text-slate-500">
                          <User className="h-4 w-4" />
                          <span>{version.changed_by_name || '系統'}</span>
                        </div>
                      </div>

                      {/* Snapshot Preview */}
                      <div className="p-3 bg-slate-50 rounded-lg text-sm">
                        <div className="grid grid-cols-2 gap-2">
                          {recordType === 'observation' && (
                            <>
                              <div>
                                <span className="text-slate-500">紀錄性質：</span>
                                <span className="ml-1">{version.record_snapshot.record_type || '-'}</span>
                              </div>
                              <div>
                                <span className="text-slate-500">事件日期：</span>
                                <span className="ml-1">{version.record_snapshot.event_date || '-'}</span>
                              </div>
                              <div className="col-span-2">
                                <span className="text-slate-500">內容：</span>
                                <span className="ml-1 line-clamp-2">
                                  {version.record_snapshot.content || '-'}
                                </span>
                              </div>
                            </>
                          )}
                          {recordType === 'surgery' && (
                            <>
                              <div>
                                <span className="text-slate-500">手術日期：</span>
                                <span className="ml-1">{version.record_snapshot.surgery_date || '-'}</span>
                              </div>
                              <div>
                                <span className="text-slate-500">手術部位：</span>
                                <span className="ml-1">{version.record_snapshot.surgery_site || '-'}</span>
                              </div>
                              <div>
                                <span className="text-slate-500">首次實驗：</span>
                                <span className="ml-1">
                                  {version.record_snapshot.is_first_experiment ? '是' : '否'}
                                </span>
                              </div>
                            </>
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
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
