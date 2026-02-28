import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { AnimalPathologyReport } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { FileUpload, FileInfo } from '@/components/ui/file-upload'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { toast } from '@/components/ui/use-toast'
import { getApiErrorMessage } from '@/lib/validation'
import { Upload, Download, FileText, Loader2 } from 'lucide-react'

interface PathologyTabProps {
  animalId: string
  earTag: string
}

export function PathologyTab({ animalId, earTag }: PathologyTabProps) {
  const queryClient = useQueryClient()

  const [showUploadDialog, setShowUploadDialog] = useState(false)
  const [files, setFiles] = useState<FileInfo[]>([])

  const { data: pathology } = useQuery({
    queryKey: ['animal-pathology', animalId],
    queryFn: async () => {
      const res = await api.get<AnimalPathologyReport>(`/animals/${animalId}/pathology`)
      return res.data
    },
    staleTime: 30_000,
  })

  const uploadMutation = useMutation({
    mutationFn: async (uploadFiles: FileInfo[]) => {
      return api.post(`/animals/${animalId}/pathology/upload`, {
        files: uploadFiles.map((f) => ({
          file_name: f.file_name,
          file_size: f.file_size,
        })),
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['animal-pathology', animalId] })
      toast({ title: '成功', description: '病理報告已上傳' })
      setShowUploadDialog(false)
      setFiles([])
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '上傳失敗'),
        variant: 'destructive',
      })
    },
  })

  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <div>
            <CardTitle>病理組織報告</CardTitle>
            <CardDescription>病理組織報告檔案</CardDescription>
          </div>
          <Button className="bg-purple-600 hover:bg-purple-700" onClick={() => setShowUploadDialog(true)}>
            <Upload className="h-4 w-4 mr-2" />
            上傳檔案
          </Button>
        </CardHeader>
        <CardContent>
          {!pathology || !pathology.attachments || pathology.attachments.length === 0 ? (
            <div className="text-center py-12 text-slate-500">
              <FileText className="h-12 w-12 mx-auto mb-4 text-slate-300" />
              <p>尚無病理組織報告</p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>檔案名稱</TableHead>
                  <TableHead>檔案大小</TableHead>
                  <TableHead>上傳時間</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {pathology.attachments.map((file) => (
                  <TableRow key={file.id}>
                    <TableCell className="font-medium">{file.file_name}</TableCell>
                    <TableCell>{(file.file_size / 1024).toFixed(2)} KB</TableCell>
                    <TableCell>{new Date(file.created_at).toLocaleString('zh-TW')}</TableCell>
                    <TableCell className="text-right">
                      <Button variant="outline" size="sm">
                        <Download className="h-4 w-4 mr-2" />
                        下載
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      <Dialog open={showUploadDialog} onOpenChange={setShowUploadDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>上傳病理組織報告</DialogTitle>
            <DialogDescription>耳號：{earTag}</DialogDescription>
          </DialogHeader>
          <div className="py-4">
            <FileUpload
              value={files}
              onChange={setFiles}
              accept=".pdf,.doc,.docx,.jpg,.jpeg,.png,.tiff"
              placeholder="拖曳病理報告檔案到此處，或點擊選擇檔案"
              maxSize={50}
              maxFiles={20}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowUploadDialog(false)}>
              取消
            </Button>
            <Button
              onClick={() => uploadMutation.mutate(files)}
              disabled={uploadMutation.isPending || files.length === 0}
              className="bg-green-600 hover:bg-green-700"
            >
              {uploadMutation.isPending && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              上傳
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
