import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { PigSource } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Badge } from '@/components/ui/badge'
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
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { toast } from '@/components/ui/use-toast'
import {
  Plus,
  Edit2,
  Trash2,
  Loader2,
  Building2,
  Phone,
  User,
  MapPin,
} from 'lucide-react'

interface SourceFormData {
  code: string
  name: string
  address?: string
  contact?: string
  phone?: string
  is_active: boolean
  sort_order: number
}

const defaultFormData: SourceFormData = {
  code: '',
  name: '',
  address: '',
  contact: '',
  phone: '',
  is_active: true,
  sort_order: 0,
}

export function PigSourcesPage() {
  const queryClient = useQueryClient()

  const [showDialog, setShowDialog] = useState(false)
  const [editingSource, setEditingSource] = useState<PigSource | null>(null)
  const [formData, setFormData] = useState<SourceFormData>(defaultFormData)

  // Query sources
  const { data: sources, isLoading } = useQuery({
    queryKey: ['pig-sources'],
    queryFn: async () => {
      const res = await api.get<PigSource[]>('/pig-sources')
      return res.data
    },
  })

  // Create mutation
  const createMutation = useMutation({
    mutationFn: async (data: SourceFormData) => {
      return api.post('/pig-sources', data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-sources'] })
      toast({ title: '成功', description: '動物來源已新增' })
      handleCloseDialog()
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '新增失敗',
        variant: 'destructive',
      })
    },
  })

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: Partial<SourceFormData> }) => {
      return api.put(`/pig-sources/${id}`, data)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-sources'] })
      toast({ title: '成功', description: '動物來源已更新' })
      handleCloseDialog()
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '更新失敗',
        variant: 'destructive',
      })
    },
  })

  // Delete mutation
  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      return api.delete(`/pig-sources/${id}`)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pig-sources'] })
      toast({ title: '成功', description: '動物來源已刪除' })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  const handleOpenDialog = (source?: PigSource) => {
    if (source) {
      setEditingSource(source)
      setFormData({
        code: source.code,
        name: source.name,
        address: source.address || '',
        contact: source.contact || '',
        phone: source.phone || '',
        is_active: source.is_active,
        sort_order: source.sort_order,
      })
    } else {
      setEditingSource(null)
      setFormData(defaultFormData)
    }
    setShowDialog(true)
  }

  const handleCloseDialog = () => {
    setShowDialog(false)
    setEditingSource(null)
    setFormData(defaultFormData)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!formData.code.trim() || !formData.name.trim()) {
      toast({ title: '錯誤', description: '代碼與名稱為必填', variant: 'destructive' })
      return
    }

    if (editingSource) {
      updateMutation.mutate({ id: editingSource.id, data: formData })
    } else {
      createMutation.mutate(formData)
    }
  }

  const handleDelete = (source: PigSource) => {
    if (confirm(`確定要刪除來源「${source.name}」嗎？`)) {
      deleteMutation.mutate(source.id)
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-slate-900">動物來源管理</h1>
          <p className="text-slate-500">管理動物的來源/供應商資訊</p>
        </div>
        <Button onClick={() => handleOpenDialog()} className="gap-2 bg-purple-600 hover:bg-purple-700">
          <Plus className="h-4 w-4" />
          新增來源
        </Button>
      </div>

      {/* Table */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Building2 className="h-5 w-5" />
            來源列表
          </CardTitle>
          <CardDescription>
            共 {sources?.length || 0} 個來源
          </CardDescription>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex items-center justify-center py-12">
              <Loader2 className="h-8 w-8 animate-spin text-slate-400" />
            </div>
          ) : !sources || sources.length === 0 ? (
            <div className="text-center py-12 text-slate-500">
              <Building2 className="h-12 w-12 mx-auto mb-4 text-slate-300" />
              <p>尚無來源資料</p>
              <Button onClick={() => handleOpenDialog()} variant="outline" className="mt-4">
                <Plus className="h-4 w-4 mr-2" />
                新增第一個來源
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>排序</TableHead>
                  <TableHead>代碼</TableHead>
                  <TableHead>名稱</TableHead>
                  <TableHead>地址</TableHead>
                  <TableHead>聯絡人</TableHead>
                  <TableHead>電話</TableHead>
                  <TableHead>狀態</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sources.map((source) => (
                  <TableRow key={source.id}>
                    <TableCell>{source.sort_order}</TableCell>
                    <TableCell className="font-mono font-medium">{source.code}</TableCell>
                    <TableCell className="font-medium">{source.name}</TableCell>
                    <TableCell>
                      {source.address ? (
                        <span className="flex items-center gap-1 text-sm text-slate-600">
                          <MapPin className="h-3 w-3" />
                          {source.address}
                        </span>
                      ) : '-'}
                    </TableCell>
                    <TableCell>
                      {source.contact ? (
                        <span className="flex items-center gap-1 text-sm text-slate-600">
                          <User className="h-3 w-3" />
                          {source.contact}
                        </span>
                      ) : '-'}
                    </TableCell>
                    <TableCell>
                      {source.phone ? (
                        <span className="flex items-center gap-1 text-sm text-slate-600">
                          <Phone className="h-3 w-3" />
                          {source.phone}
                        </span>
                      ) : '-'}
                    </TableCell>
                    <TableCell>
                      <Badge className={source.is_active ? 'bg-green-100 text-green-800' : 'bg-slate-100 text-slate-800'}>
                        {source.is_active ? '啟用' : '停用'}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex items-center justify-end gap-1">
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleOpenDialog(source)}
                        >
                          <Edit2 className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          onClick={() => handleDelete(source)}
                          disabled={deleteMutation.isPending}
                        >
                          <Trash2 className="h-4 w-4 text-red-500" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Add/Edit Dialog */}
      <Dialog open={showDialog} onOpenChange={setShowDialog}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>
              {editingSource ? '編輯動物來源' : '新增動物來源'}
            </DialogTitle>
            <DialogDescription>
              {editingSource ? '修改來源資訊' : '輸入新來源的資訊'}
            </DialogDescription>
          </DialogHeader>

          <form onSubmit={handleSubmit} className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="code">代碼 *</Label>
                <Input
                  id="code"
                  value={formData.code}
                  onChange={(e) => setFormData({ ...formData, code: e.target.value })}
                  placeholder="如：SOURCE01"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="name">名稱 *</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  placeholder="來源名稱"
                  required
                />
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="address">地址</Label>
              <Input
                id="address"
                value={formData.address}
                onChange={(e) => setFormData({ ...formData, address: e.target.value })}
                placeholder="完整地址"
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="contact">聯絡人</Label>
                <Input
                  id="contact"
                  value={formData.contact}
                  onChange={(e) => setFormData({ ...formData, contact: e.target.value })}
                  placeholder="聯絡人姓名"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="phone">電話</Label>
                <Input
                  id="phone"
                  value={formData.phone}
                  onChange={(e) => setFormData({ ...formData, phone: e.target.value })}
                  placeholder="聯絡電話"
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="sort_order">排序</Label>
                <Input
                  id="sort_order"
                  type="number"
                  value={formData.sort_order}
                  onChange={(e) => setFormData({ ...formData, sort_order: parseInt(e.target.value) || 0 })}
                />
              </div>
              <div className="space-y-2">
                <Label>狀態</Label>
                <div className="flex items-center gap-4 pt-2">
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="is_active"
                      checked={formData.is_active}
                      onChange={() => setFormData({ ...formData, is_active: true })}
                      className="w-4 h-4 text-purple-600"
                    />
                    <span className="text-sm">啟用</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="is_active"
                      checked={!formData.is_active}
                      onChange={() => setFormData({ ...formData, is_active: false })}
                      className="w-4 h-4 text-purple-600"
                    />
                    <span className="text-sm">停用</span>
                  </label>
                </div>
              </div>
            </div>

            <DialogFooter>
              <Button type="button" variant="outline" onClick={handleCloseDialog}>
                取消
              </Button>
              <Button
                type="submit"
                disabled={createMutation.isPending || updateMutation.isPending}
                className="bg-purple-600 hover:bg-purple-700"
              >
                {(createMutation.isPending || updateMutation.isPending) && (
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                )}
                {editingSource ? '更新' : '新增'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  )
}
