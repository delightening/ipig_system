import { useState } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  ProtocolResponse,
  ProtocolStatus,
} from '@/lib/api'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { EmptyState } from '@/components/ui/empty-state'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useToast } from '@/components/ui/use-toast'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import {
  ArrowLeft,
  FileText,
  User,
  Building,
  Calendar,
  Loader2,
  AlertTriangle,
  Download,
  ClipboardList,
} from 'lucide-react'
import { formatDate } from '@/lib/utils'
import { getApiErrorMessage } from '@/lib/validation'
import type { ProtocolWorkingContent, ProtocolAnimalItem } from '@/types/protocol'
import type { AnimalListItem } from '@/types/animal'
import { animalStatusNames, animalBreedNames, animalGenderNames } from '@/types/animal'
import type { PaginatedResponse } from '@/types/common'

const statusColors: Record<ProtocolStatus, 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'> = {
  DRAFT: 'secondary',
  SUBMITTED: 'default',
  PRE_REVIEW: 'default',
  PRE_REVIEW_REVISION_REQUIRED: 'destructive',
  VET_REVIEW: 'warning',
  VET_REVISION_REQUIRED: 'destructive',
  UNDER_REVIEW: 'warning',
  REVISION_REQUIRED: 'destructive',
  RESUBMITTED: 'default',
  APPROVED: 'success',
  APPROVED_WITH_CONDITIONS: 'success',
  DEFERRED: 'secondary',
  REJECTED: 'destructive',
  SUSPENDED: 'destructive',
  CLOSED: 'outline',
  DELETED: 'destructive',
}

// 輔助函數：判斷欄位顯示文字
const getPenLocationDisplay = (animal: AnimalListItem) => {
  if (animal.status === 'completed' && !animal.pen_location) {
    return '犧牲'
  }
  return animal.pen_location || '-'
}

export function MyProjectDetailPage() {
  const { t } = useTranslation()
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { toast } = useToast()
  const [activeTab, setActiveTab] = useState<'application' | 'animals'>('application')
  const [showCloseDialog, setShowCloseDialog] = useState(false)

  // 取得計畫詳情
  const { data: protocol, isLoading } = useQuery({
    queryKey: ['my-project', id],
    queryFn: async () => {
      const response = await api.get<ProtocolResponse>(`/protocols/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  // 結案 mutation
  const closeProtocolMutation = useMutation({
    mutationFn: async () => {
      return api.post(`/protocols/${id}/status`, {
        to_status: 'CLOSED',
        remark: '計畫結案',
      })
    },
    onSuccess: () => {
      toast({
        title: '成功',
        description: '計畫已結案',
      })
      queryClient.invalidateQueries({ queryKey: ['my-project', id] })
      queryClient.invalidateQueries({ queryKey: ['my-projects'] })
      setShowCloseDialog(false)
    },
    onError: (error: unknown) => {
      toast({
        title: '錯誤',
        description: getApiErrorMessage(error, '結案失敗'),
        variant: 'destructive',
      })
    },
  })

  const iacucNo = protocol?.protocol.iacuc_no
  const { data: animalsData } = useQuery({
    queryKey: ['my-project-animals', iacucNo],
    queryFn: async () => {
      const res = await api.get<PaginatedResponse<AnimalListItem>>(
        `/animals?iacuc_no=${encodeURIComponent(iacucNo!)}&per_page=200`
      )
      return res.data
    },
    enabled: !!iacucNo,
  })
  const animals = animalsData?.data ?? []

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (!protocol) {
    return (
      <div className="text-center py-12">
        <AlertTriangle className="h-12 w-12 mx-auto mb-4 text-yellow-500" />
        <h2 className="text-xl font-semibold mb-2">找不到計劃</h2>
        <p className="text-muted-foreground mb-4">此計劃不存在或您沒有權限查看</p>
        <Button asChild>
          <Link to="/my-projects">返回我的計劃</Link>
        </Button>
      </div>
    )
  }

  const workingContent = protocol.protocol.working_content as unknown as ProtocolWorkingContent

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={() => navigate(-1)}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <div className="flex items-center gap-3">
              <h1 className="text-2xl font-bold">{protocol.protocol.title}</h1>
              <Badge variant={statusColors[protocol.protocol.status]} className="text-sm">
                {t(`protocols.status.${protocol.protocol.status}`)}
              </Badge>
            </div>
            <p className="text-muted-foreground mt-1">
              {protocol.protocol.iacuc_no ? `IACUC No.: ${protocol.protocol.iacuc_no}` : 'IACUC No.: 尚未核發'}
            </p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button variant="outline">
            <Download className="mr-2 h-4 w-4" />
            下載 PDF
          </Button>
          {(protocol.protocol.status === 'APPROVED' || protocol.protocol.status === 'APPROVED_WITH_CONDITIONS') && (
            <Button
              variant="outline"
              onClick={() => setShowCloseDialog(true)}
            >
              結案
            </Button>
          )}
        </div>
      </div>

      {/* Info Cards */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <FileText className="h-4 w-4 text-blue-500" />
              {protocol.protocol.iacuc_no?.startsWith('APIG-') ? 'APIG 編號' : 'IACUC 編號'}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold text-orange-600">
              {protocol.protocol.iacuc_no || '尚未核發'}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <User className="h-4 w-4 text-green-500" />
              計畫主持人
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-lg font-semibold">{protocol.pi_name || '-'}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Building className="h-4 w-4 text-purple-500" />
              委託單位
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-lg font-semibold">{protocol.pi_organization || '-'}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Calendar className="h-4 w-4 text-yellow-500" />
              執行期間
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-lg font-semibold">
              {protocol.protocol.start_date && protocol.protocol.end_date
                ? `${formatDate(protocol.protocol.start_date)} ~ ${formatDate(protocol.protocol.end_date)}`
                : '尚未設定'}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <div className="border-b">
        <nav className="flex gap-4">
          <button
            onClick={() => setActiveTab('application')}
            className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors ${activeTab === 'application'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-muted-foreground hover:text-foreground'
              }`}
          >
            <ClipboardList className="h-4 w-4" />
            申請表
          </button>
          <button
            onClick={() => setActiveTab('animals')}
            className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors ${activeTab === 'animals'
              ? 'border-blue-600 text-blue-600'
              : 'border-transparent text-muted-foreground hover:text-foreground'
              }`}
          >
            <FileText className="h-4 w-4" />
            動物紀錄
          </button>
        </nav>
      </div>

      {/* Tab Content */}
      {activeTab === 'application' && (
        <div className="space-y-6">
          {/* 基本資料 */}
          <Card>
            <CardHeader>
              <CardTitle>基本資料</CardTitle>
            </CardHeader>
            <CardContent>
              <dl className="grid gap-4 md:grid-cols-2">
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">計畫類型</dt>
                  <dd className="mt-1">{workingContent?.basic?.project_type || '-'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">計畫種類</dt>
                  <dd className="mt-1">
                    {workingContent?.basic?.project_category || '-'}
                    {workingContent?.basic?.project_category_other && ` (${workingContent.basic.project_category_other})`}
                  </dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">GLP 符合性</dt>
                  <dd className="mt-1">
                    {workingContent?.basic?.is_glp ? (
                      <Badge variant="success">是</Badge>
                    ) : (
                      <Badge variant="secondary">否</Badge>
                    )}
                  </dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">經費來源</dt>
                  <dd className="mt-1">
                    {workingContent?.basic?.funding_sources?.length > 0
                      ? workingContent.basic.funding_sources.join('、')
                      : '-'}
                  </dd>
                </div>
              </dl>
            </CardContent>
          </Card>

          {/* 3Rs 原則 */}
          <Card>
            <CardHeader>
              <CardTitle>3Rs 原則說明</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <h4 className="font-medium text-sm text-muted-foreground mb-1">替代 (Replacement)</h4>
                <p className="text-sm whitespace-pre-wrap">
                  {workingContent?.purpose?.replacement?.rationale || '未填寫'}
                </p>
              </div>
              <div>
                <h4 className="font-medium text-sm text-muted-foreground mb-1">減量 (Reduction)</h4>
                <p className="text-sm whitespace-pre-wrap">
                  {workingContent?.purpose?.reduction?.design || '未填寫'}
                </p>
              </div>
              <div>
                <h4 className="font-medium text-sm text-muted-foreground mb-1">研究目的及重要性</h4>
                <p className="text-sm whitespace-pre-wrap">
                  {workingContent?.purpose?.significance || '未填寫'}
                </p>
              </div>
            </CardContent>
          </Card>

          {/* 試驗物質 */}
          <Card>
            <CardHeader>
              <CardTitle>試驗物質與對照組</CardTitle>
            </CardHeader>
            <CardContent>
              {workingContent?.items?.use_test_item === true ? (
                <div className="space-y-4">
                  {workingContent?.items?.test_items?.length > 0 && (
                    <div>
                      <h4 className="font-medium mb-2">試驗物質</h4>
                      {workingContent.items.test_items.map((item, index) => (
                        <div key={index} className="p-3 border rounded mb-2">
                          <dl className="grid gap-2 md:grid-cols-2 text-sm">
                            <div>
                              <dt className="text-muted-foreground">物質名稱</dt>
                              <dd>{String(item.name ?? '-')}</dd>
                            </div>
                            <div>
                              <dt className="text-muted-foreground">用途</dt>
                              <dd>{String(item.purpose ?? '-')}</dd>
                            </div>
                            <div>
                              <dt className="text-muted-foreground">劑型</dt>
                              <dd>{String(item.form ?? '-')}</dd>
                            </div>
                            <div>
                              <dt className="text-muted-foreground">保存環境</dt>
                              <dd>{String(item.storage_conditions ?? '-')}</dd>
                            </div>
                          </dl>
                        </div>
                      ))}
                    </div>
                  )}
                  {workingContent?.items?.control_items?.length > 0 && (
                    <div>
                      <h4 className="font-medium mb-2">對照物質</h4>
                      {workingContent.items.control_items.map((item, index) => (
                        <div key={index} className="p-3 border rounded mb-2">
                          <dl className="grid gap-2 md:grid-cols-2 text-sm">
                            <div>
                              <dt className="text-muted-foreground">對照名稱</dt>
                              <dd>{String(item.name ?? '-')}</dd>
                            </div>
                            <div>
                              <dt className="text-muted-foreground">目的</dt>
                              <dd>{String(item.purpose ?? '-')}</dd>
                            </div>
                          </dl>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ) : (
                <p className="text-muted-foreground">未使用試驗物質</p>
              )}
            </CardContent>
          </Card>

          {/* 動物資訊 */}
          <Card>
            <CardHeader>
              <CardTitle>動物資訊</CardTitle>
            </CardHeader>
            <CardContent>
              {workingContent?.animals?.animals?.length > 0 ? (
                <div className="space-y-3">
                  {workingContent.animals.animals.map((animal: ProtocolAnimalItem, index: number) => (
                    <div key={index} className="p-3 border rounded">
                      <h4 className="font-medium mb-2">動物群組 #{index + 1}</h4>
                      <dl className="grid gap-3 md:grid-cols-3 text-sm">
                        <div>
                          <dt className="text-muted-foreground">物種</dt>
                          <dd>
                            {animal.species === 'pig' ? '豬' : animal.species === 'other' ? String(animal.species_other ?? '-') : '-'}
                          </dd>
                        </div>
                        <div>
                          <dt className="text-muted-foreground">品系</dt>
                          <dd>
                            {animal.strain === 'white_pig' ? '一般白豬' :
                              animal.strain === 'mini_pig' ? '迷你豬' :
                                String(animal.strain_other ?? '-')}
                          </dd>
                        </div>
                        <div>
                          <dt className="text-muted-foreground">性別</dt>
                          <dd>
                            {animal.sex === 'male' ? '公' :
                              animal.sex === 'female' ? '母' :
                                animal.sex === 'both' ? '公母均可' : '-'}
                          </dd>
                        </div>
                        <div>
                          <dt className="text-muted-foreground">數量</dt>
                          <dd>{String(animal.number ?? '-')}</dd>
                        </div>
                        <div>
                          <dt className="text-muted-foreground">月齡範圍</dt>
                          <dd>
                            {animal.age_unlimited ? '不限' :
                              `${String(animal.age_min ?? '-')} ~ ${String(animal.age_max ?? '-')} 月`}
                          </dd>
                        </div>
                        <div>
                          <dt className="text-muted-foreground">體重範圍</dt>
                          <dd>
                            {animal.weight_unlimited ? '不限' :
                              `${String(animal.weight_min ?? '-')} ~ ${String(animal.weight_max ?? '-')} kg`}
                          </dd>
                        </div>
                      </dl>
                    </div>
                  ))}
                  {workingContent.animals.total_animals && (
                    <div className="mt-2 font-medium">
                      總動物數: {workingContent.animals.total_animals} 頭
                    </div>
                  )}
                </div>
              ) : (
                <p className="text-muted-foreground">尚未填寫動物資訊</p>
              )}
            </CardContent>
          </Card>

          {/* 試驗流程 */}
          <Card>
            <CardHeader>
              <CardTitle>試驗流程與麻醉止痛</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <h4 className="font-medium text-sm text-muted-foreground mb-1">試驗流程描述</h4>
                <p className="text-sm whitespace-pre-wrap">
                  {workingContent?.design?.procedures || '未填寫'}
                </p>
              </div>
              <div>
                <h4 className="font-medium text-sm text-muted-foreground mb-1">麻醉方案</h4>
                <p className="text-sm whitespace-pre-wrap">
                  {workingContent?.design?.anesthesia?.is_under_anesthesia === true
                    ? `是 - ${workingContent?.design?.anesthesia?.anesthesia_type || ''}`
                    : workingContent?.design?.anesthesia?.is_under_anesthesia === false
                      ? '否'
                      : '未填寫'}
                </p>
              </div>
              <div>
                <h4 className="font-medium text-sm text-muted-foreground mb-1">止痛管理</h4>
                <p className="text-sm whitespace-pre-wrap">
                  {workingContent?.design?.pain?.management_plan ||
                    (workingContent?.design?.pain?.category ? `疼痛類別: ${workingContent.design.pain.category}` : '未填寫')}
                </p>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {activeTab === 'animals' && (
        <Card>
          <CardHeader>
            <CardTitle>動物紀錄</CardTitle>
            <CardDescription>此計劃下所有已分配動物清單</CardDescription>
          </CardHeader>
          <CardContent>
            {animals.length > 0 ? (
              <>
                <div className="flex gap-2 mb-4">
                  <Button variant="outline" size="sm">
                    <Download className="mr-2 h-4 w-4" />
                    下載病歷總表
                  </Button>
                  <Button variant="outline" size="sm">
                    <Download className="mr-2 h-4 w-4" />
                    下載觀察試驗紀錄
                  </Button>
                  <Button variant="outline" size="sm">
                    <Download className="mr-2 h-4 w-4" />
                    下載手術紀錄
                  </Button>
                </div>
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>系統號</TableHead>
                      <TableHead>耳號</TableHead>
                      <TableHead>欄位</TableHead>
                      <TableHead>動物狀態</TableHead>
                      <TableHead>品種</TableHead>
                      <TableHead>性別</TableHead>
                      <TableHead>進場日期</TableHead>
                      <TableHead className="text-right">動作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {animals.map((animal) => (
                      <TableRow key={animal.id}>
                        <TableCell>{animal.animal_no ?? animal.id.slice(0, 8)}</TableCell>
                        <TableCell className="text-orange-600 font-medium">{animal.ear_tag}</TableCell>
                        <TableCell>{getPenLocationDisplay(animal)}</TableCell>
                        <TableCell>
                          <Badge variant="warning">{animalStatusNames[animal.status] ?? animal.status}</Badge>
                        </TableCell>
                        <TableCell>{animalBreedNames[animal.breed] ?? animal.breed}</TableCell>
                        <TableCell>{animalGenderNames[animal.gender] ?? animal.gender}</TableCell>
                        <TableCell>{formatDate(animal.entry_date)}</TableCell>
                        <TableCell className="text-right">
                          <Button variant="ghost" size="sm" asChild>
                            <Link to={`/animals/${animal.id}`}>檢視</Link>
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </>
            ) : (
              <EmptyState
                icon={FileText}
                title="尚無動物紀錄"
                description="此計劃目前尚未分配動物"
              />
            )}
          </CardContent>
        </Card>
      )}

      {/* 結案確認對話框 */}
      <Dialog open={showCloseDialog} onOpenChange={setShowCloseDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>確認結案</DialogTitle>
            <DialogDescription>
              確定要將此計畫結案嗎？結案後將無法再進行修改。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCloseDialog(false)}>
              取消
            </Button>
            <Button
              variant="destructive"
              onClick={() => closeProtocolMutation.mutate()}
              disabled={closeProtocolMutation.isPending}
            >
              {closeProtocolMutation.isPending ? '處理中...' : '確認結案'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
