import { useState } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  ProtocolResponse,
  ProtocolStatus,
  protocolStatusNames,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
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

const statusColors: Record<ProtocolStatus, 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'> = {
  DRAFT: 'secondary',
  SUBMITTED: 'default',
  PRE_REVIEW: 'default',
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

// 模擬豬隻資料（實際應從 API 取得）
interface PigRecord {
  id: number
  earTag: string
  penLocation?: string | null
  status: string
  breed: string
  gender: string
  entryDate: string
}

// 輔助函數：判斷欄位顯示文字
const getPenLocationDisplay = (pig: { status: string; penLocation?: string | null }) => {
  if (pig.status === 'completed' && !pig.penLocation) {
    return '犧牲'
  }
  return pig.penLocation || '-'
}

export function MyProjectDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { toast } = useToast()
  const [activeTab, setActiveTab] = useState<'application' | 'pigs'>('application')
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
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '結案失敗',
        variant: 'destructive',
      })
    },
  })

  // TODO: 取得豬隻紀錄
  const pigs: PigRecord[] = [] // 實際應從 /my-projects/{id}/pigs 取得

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

  const workingContent = protocol.working_content as any

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
              <h1 className="text-2xl font-bold">{protocol.title}</h1>
              <Badge variant={statusColors[protocol.status]} className="text-sm">
                {protocolStatusNames[protocol.status]}
              </Badge>
            </div>
            <p className="text-muted-foreground mt-1">
              {protocol.iacuc_no ? `IACUC No.: ${protocol.iacuc_no}` : 'IACUC No.: 尚未核發'}
            </p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button variant="outline">
            <Download className="mr-2 h-4 w-4" />
            下載 PDF
          </Button>
          {(protocol.status === 'APPROVED' || protocol.status === 'APPROVED_WITH_CONDITIONS') && (
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
              {protocol.iacuc_no?.startsWith('APIG-') ? 'APIG 編號' : 'IACUC 編號'}
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-xl font-bold text-orange-600">
              {protocol.iacuc_no || '尚未核發'}
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
              {protocol.start_date && protocol.end_date
                ? `${formatDate(protocol.start_date)} ~ ${formatDate(protocol.end_date)}`
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
            onClick={() => setActiveTab('pigs')}
            className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors ${activeTab === 'pigs'
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-muted-foreground hover:text-foreground'
              }`}
          >
            <FileText className="h-4 w-4" />
            豬隻紀錄
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
                  <dt className="text-sm font-medium text-muted-foreground">GLP 合規</dt>
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
                      {workingContent.items.test_items.map((item: any, index: number) => (
                        <div key={index} className="p-3 border rounded mb-2">
                          <dl className="grid gap-2 md:grid-cols-2 text-sm">
                            <div>
                              <dt className="text-muted-foreground">物質名稱</dt>
                              <dd>{item.name || '-'}</dd>
                            </div>
                            <div>
                              <dt className="text-muted-foreground">用途</dt>
                              <dd>{item.purpose || '-'}</dd>
                            </div>
                            <div>
                              <dt className="text-muted-foreground">劑型</dt>
                              <dd>{item.form || '-'}</dd>
                            </div>
                            <div>
                              <dt className="text-muted-foreground">保存環境</dt>
                              <dd>{item.storage_conditions || '-'}</dd>
                            </div>
                          </dl>
                        </div>
                      ))}
                    </div>
                  )}
                  {workingContent?.items?.control_items?.length > 0 && (
                    <div>
                      <h4 className="font-medium mb-2">對照物質</h4>
                      {workingContent.items.control_items.map((item: any, index: number) => (
                        <div key={index} className="p-3 border rounded mb-2">
                          <dl className="grid gap-2 md:grid-cols-2 text-sm">
                            <div>
                              <dt className="text-muted-foreground">對照名稱</dt>
                              <dd>{item.name || '-'}</dd>
                            </div>
                            <div>
                              <dt className="text-muted-foreground">目的</dt>
                              <dd>{item.purpose || '-'}</dd>
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
                  {workingContent.animals.animals.map((animal: any, index: number) => (
                    <div key={index} className="p-3 border rounded">
                      <h4 className="font-medium mb-2">動物群組 #{index + 1}</h4>
                      <dl className="grid gap-3 md:grid-cols-3 text-sm">
                        <div>
                          <dt className="text-muted-foreground">物種</dt>
                          <dd>
                            {animal.species === 'pig' ? '豬' : animal.species === 'other' ? animal.species_other : '-'}
                          </dd>
                        </div>
                        <div>
                          <dt className="text-muted-foreground">品系</dt>
                          <dd>
                            {animal.strain === 'white_pig' ? '一般白豬' :
                              animal.strain === 'mini_pig' ? '迷你豬' :
                                animal.strain_other || '-'}
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
                          <dd>{animal.number || '-'}</dd>
                        </div>
                        <div>
                          <dt className="text-muted-foreground">月齡範圍</dt>
                          <dd>
                            {animal.age_unlimited ? '不限' :
                              `${animal.age_min || '-'} ~ ${animal.age_max || '-'} 月`}
                          </dd>
                        </div>
                        <div>
                          <dt className="text-muted-foreground">體重範圍</dt>
                          <dd>
                            {animal.weight_unlimited ? '不限' :
                              `${animal.weight_min || '-'} ~ ${animal.weight_max || '-'} kg`}
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

      {activeTab === 'pigs' && (
        <Card>
          <CardHeader>
            <CardTitle>豬隻紀錄</CardTitle>
            <CardDescription>此計劃下所有已分配豬隻清單</CardDescription>
          </CardHeader>
          <CardContent>
            {pigs.length > 0 ? (
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
                      <TableHead>豬隻狀態</TableHead>
                      <TableHead>品種</TableHead>
                      <TableHead>性別</TableHead>
                      <TableHead>進場日期</TableHead>
                      <TableHead className="text-right">動作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {pigs.map((pig) => (
                      <TableRow key={pig.id}>
                        <TableCell>{pig.id}</TableCell>
                        <TableCell className="text-orange-600 font-medium">{pig.earTag}</TableCell>
                        <TableCell>{getPenLocationDisplay(pig)}</TableCell>
                        <TableCell>
                          <Badge variant="warning">{pig.status}</Badge>
                        </TableCell>
                        <TableCell>{pig.breed}</TableCell>
                        <TableCell>{pig.gender}</TableCell>
                        <TableCell>{formatDate(pig.entryDate)}</TableCell>
                        <TableCell className="text-right">
                          <Button variant="ghost" size="sm">
                            檢視
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </>
            ) : (
              <div className="text-center py-12">
                <FileText className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
                <h3 className="text-lg font-semibold mb-2">尚無豬隻紀錄</h3>
                <p className="text-muted-foreground">
                  此計劃目前尚未分配豬隻
                </p>
              </div>
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
