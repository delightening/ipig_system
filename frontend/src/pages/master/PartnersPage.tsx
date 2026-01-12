import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, { Partner } from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import { Plus, Search, Edit, Trash2, Loader2, Users } from 'lucide-react'

export function PartnersPage() {
  const queryClient = useQueryClient()
  const [search, setSearch] = useState('')
  const [typeFilter, setTypeFilter] = useState<string>('all')
  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingPartner, setEditingPartner] = useState<Partner | null>(null)
  const [formData, setFormData] = useState({
    partner_type: 'supplier' as 'supplier' | 'customer',
    supplier_category: '' as '' | 'drug' | 'consumable' | 'feed' | 'equipment',
    code: '',
    name: '',
    tax_id: '',
    phone: '',
    email: '',
    address: '',
  })
  const [isGeneratingCode, setIsGeneratingCode] = useState(false)

  const { data: partners, isLoading } = useQuery({
    queryKey: ['partners', search, typeFilter],
    queryFn: async () => {
      let params = ''
      if (search) params += `keyword=${encodeURIComponent(search)}&`
      if (typeFilter && typeFilter !== 'all') params += `partner_type=${typeFilter}&`
      const response = await api.get<Partner[]>(`/partners?${params}`)
      return response.data
    },
  })

  const createMutation = useMutation({
    mutationFn: (data: typeof formData) => api.post('/partners', data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['partners'] })
      toast({ title: '成功', description: '夥伴已建立' })
      setDialogOpen(false)
      resetForm()
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '建立失敗',
        variant: 'destructive',
      })
    },
  })

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: typeof formData }) =>
      api.put(`/partners/${id}`, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['partners'] })
      toast({ title: '成功', description: '夥伴已更新' })
      setDialogOpen(false)
      resetForm()
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '更新失敗',
        variant: 'destructive',
      })
    },
  })

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/partners/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['partners'] })
      toast({ title: '成功', description: '夥伴已刪除' })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  const resetForm = () => {
    setFormData({
      partner_type: 'supplier',
      supplier_category: '',
      code: '',
      name: '',
      tax_id: '',
      phone: '',
      email: '',
      address: '',
    })
    setEditingPartner(null)
  }

  // 當供應商類別改變時，自動生成代碼
  const handleSupplierCategoryChange = async (category: 'drug' | 'consumable' | 'feed' | 'equipment') => {
    setFormData({ ...formData, supplier_category: category, code: '' })
    
    if (!editingPartner && category) {
      setIsGeneratingCode(true)
      try {
        const response = await api.get<{ code: string }>(`/partners/generate-code?category=${category}`)
        setFormData({ ...formData, supplier_category: category, code: response.data.code })
      } catch (error: any) {
        toast({
          title: '錯誤',
          description: error?.response?.data?.error?.message || '生成代碼失敗',
          variant: 'destructive',
        })
      } finally {
        setIsGeneratingCode(false)
      }
    }
  }

  const handleEdit = (partner: Partner) => {
    setEditingPartner(partner)
    setFormData({
      partner_type: partner.partner_type,
      supplier_category: (partner as any).supplier_category || '',
      code: partner.code,
      name: partner.name,
      tax_id: partner.tax_id || '',
      phone: partner.phone || '',
      email: partner.email || '',
      address: partner.address || '',
    })
    setDialogOpen(true)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    // 將空字串轉換為 null，避免後端驗證錯誤
    const submitData = {
      ...formData,
      code: formData.code.trim() || null,
      supplier_category: formData.supplier_category || null,
      email: formData.email.trim() || null,
      phone: formData.phone.trim() || null,
      tax_id: formData.tax_id.trim() || null,
      address: formData.address.trim() || null,
    }
    if (editingPartner) {
      updateMutation.mutate({ id: editingPartner.id, data: submitData })
    } else {
      createMutation.mutate(submitData)
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold tracking-tight">供應商/客戶管理</h1>
          <p className="text-muted-foreground">管理系統中的供應商與客戶資料</p>
        </div>
        <Button onClick={() => { resetForm(); setDialogOpen(true) }}>
          <Plus className="mr-2 h-4 w-4" />
          新增夥伴
        </Button>
      </div>

      <div className="flex gap-4">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜尋夥伴..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-9"
          />
        </div>
        <Select value={typeFilter} onValueChange={setTypeFilter}>
          <SelectTrigger className="w-40">
            <SelectValue placeholder="全部類型" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">全部類型</SelectItem>
            <SelectItem value="supplier">供應商</SelectItem>
            <SelectItem value="customer">客戶</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>類型</TableHead>
              <TableHead>代碼</TableHead>
              <TableHead>名稱</TableHead>
              <TableHead>統編</TableHead>
              <TableHead>電話</TableHead>
              <TableHead>狀態</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isLoading ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin mx-auto text-muted-foreground" />
                </TableCell>
              </TableRow>
            ) : partners && partners.length > 0 ? (
              partners.map((partner) => (
                <TableRow key={partner.id}>
                  <TableCell>
                    <Badge variant={partner.partner_type === 'supplier' ? 'default' : 'secondary'}>
                      {partner.partner_type === 'supplier' ? '供應商' : '客戶'}
                    </Badge>
                  </TableCell>
                  <TableCell className="font-mono">{partner.code}</TableCell>
                  <TableCell className="font-medium">{partner.name}</TableCell>
                  <TableCell>{partner.tax_id || '-'}</TableCell>
                  <TableCell>{partner.phone || '-'}</TableCell>
                  <TableCell>
                    {partner.is_active ? (
                      <Badge variant="success">啟用</Badge>
                    ) : (
                      <Badge variant="destructive">停用</Badge>
                    )}
                  </TableCell>
                  <TableCell className="text-right">
                    <Button variant="ghost" size="icon" onClick={() => handleEdit(partner)}>
                      <Edit className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => {
                        if (confirm('確定要刪除此夥伴嗎？')) {
                          deleteMutation.mutate(partner.id)
                        }
                      }}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell colSpan={7} className="text-center py-8">
                  <Users className="h-12 w-12 mx-auto mb-2 text-muted-foreground" />
                  <p className="text-muted-foreground">尚無夥伴資料</p>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{editingPartner ? '編輯夥伴' : '新增夥伴'}</DialogTitle>
            <DialogDescription>
              {editingPartner ? '修改夥伴資料' : '建立新的供應商或客戶'}
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={handleSubmit}>
            <div className="grid gap-4 py-4">
              <div className="grid grid-cols-4 items-center gap-4">
                <Label className="text-right">類型</Label>
                <Select
                  value={formData.partner_type}
                  onValueChange={(value: 'supplier' | 'customer') =>
                    setFormData({ ...formData, partner_type: value, supplier_category: '', code: '' })
                  }
                  disabled={!!editingPartner}
                >
                  <SelectTrigger className="col-span-3">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="supplier">供應商</SelectItem>
                    <SelectItem value="customer">客戶</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              {formData.partner_type === 'supplier' && (
                <div className="grid grid-cols-4 items-center gap-4">
                  <Label className="text-right">提供商類型</Label>
                  <Select
                    value={formData.supplier_category}
                    onValueChange={(value: 'drug' | 'consumable' | 'feed' | 'equipment') =>
                      handleSupplierCategoryChange(value)
                    }
                    disabled={!!editingPartner || isGeneratingCode}
                  >
                    <SelectTrigger className="col-span-3">
                      <SelectValue placeholder="請選擇提供商類型" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="drug">藥物</SelectItem>
                      <SelectItem value="consumable">耗材</SelectItem>
                      <SelectItem value="feed">飼料</SelectItem>
                      <SelectItem value="equipment">儀器</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              )}
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="code" className="text-right">代碼</Label>
                <div className="col-span-3 flex gap-2">
                  <Input
                    id="code"
                    value={formData.code}
                    onChange={(e) => setFormData({ ...formData, code: e.target.value })}
                    disabled={!!editingPartner || isGeneratingCode}
                    required
                    placeholder={isGeneratingCode ? '生成中...' : '將根據提供商類型自動生成'}
                  />
                  {isGeneratingCode && (
                    <Loader2 className="h-4 w-4 animate-spin text-muted-foreground self-center" />
                  )}
                </div>
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="name" className="text-right">名稱</Label>
                <Input
                  id="name"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  className="col-span-3"
                  required
                />
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="tax_id" className="text-right">統編</Label>
                <Input
                  id="tax_id"
                  value={formData.tax_id}
                  onChange={(e) => setFormData({ ...formData, tax_id: e.target.value })}
                  className="col-span-3"
                />
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="phone" className="text-right">電話</Label>
                <Input
                  id="phone"
                  value={formData.phone}
                  onChange={(e) => setFormData({ ...formData, phone: e.target.value })}
                  className="col-span-3"
                />
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="email" className="text-right">Email</Label>
                <Input
                  id="email"
                  type="email"
                  value={formData.email}
                  onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                  className="col-span-3"
                />
              </div>
              <div className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor="address" className="text-right">地址</Label>
                <Input
                  id="address"
                  value={formData.address}
                  onChange={(e) => setFormData({ ...formData, address: e.target.value })}
                  className="col-span-3"
                />
              </div>
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setDialogOpen(false)}>
                取消
              </Button>
              <Button type="submit" disabled={createMutation.isPending || updateMutation.isPending}>
                {(createMutation.isPending || updateMutation.isPending) && (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                )}
                {editingPartner ? '更新' : '建立'}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  )
}
