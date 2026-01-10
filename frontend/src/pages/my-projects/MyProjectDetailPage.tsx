import { useState } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import api, {
  ProtocolResponse,
  ProtocolStatus,
  protocolStatusNames,
} from '@/lib/api'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
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
}

// 模擬豬隻資料（實際應從 API 取得）
interface PigRecord {
  id: number
  earTag: string
  penLocation: string
  status: string
  breed: string
  gender: string
  entryDate: string
}

export function MyProjectDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [activeTab, setActiveTab] = useState<'application' | 'pigs'>('application')

  // 取得計畫詳情
  const { data: protocol, isLoading } = useQuery({
    queryKey: ['my-project', id],
    queryFn: async () => {
      const response = await api.get<ProtocolResponse>(`/protocols/${id}`)
      return response.data
    },
    enabled: !!id,
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
              <h1 className="text-2xl font-bold">{protocol.protocol_no}</h1>
              <Badge variant={statusColors[protocol.status]} className="text-sm">
                {protocolStatusNames[protocol.status]}
              </Badge>
            </div>
            <p className="text-muted-foreground mt-1">{protocol.title}</p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button variant="outline">
            <Download className="mr-2 h-4 w-4" />
            下載 PDF
          </Button>
        </div>
      </div>

      {/* Info Cards */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <FileText className="h-4 w-4 text-blue-500" />
              IACUC 編號
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
            className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors ${
              activeTab === 'application'
                ? 'border-blue-600 text-blue-600'
                : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
          >
            <ClipboardList className="h-4 w-4" />
            申請表
          </button>
          <button
            onClick={() => setActiveTab('pigs')}
            className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors ${
              activeTab === 'pigs'
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
                  <dt className="text-sm font-medium text-muted-foreground">經費來源</dt>
                  <dd className="mt-1">{workingContent?.basic?.funding_source || '-'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">GLP 合規</dt>
                  <dd className="mt-1">
                    {workingContent?.basic?.glp_compliant ? (
                      <Badge variant="success">是</Badge>
                    ) : (
                      <Badge variant="secondary">否</Badge>
                    )}
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
                  {workingContent?.threeRs?.replacement || '未填寫'}
                </p>
              </div>
              <div>
                <h4 className="font-medium text-sm text-muted-foreground mb-1">減量 (Reduction)</h4>
                <p className="text-sm whitespace-pre-wrap">
                  {workingContent?.threeRs?.reduction || '未填寫'}
                </p>
              </div>
              <div>
                <h4 className="font-medium text-sm text-muted-foreground mb-1">精緻化 (Refinement)</h4>
                <p className="text-sm whitespace-pre-wrap">
                  {workingContent?.threeRs?.refinement || '未填寫'}
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
              <dl className="grid gap-4 md:grid-cols-2">
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">試驗物質名稱</dt>
                  <dd className="mt-1">{workingContent?.testSubstance?.name || '-'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">來源</dt>
                  <dd className="mt-1">{workingContent?.testSubstance?.source || '-'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">給藥途徑</dt>
                  <dd className="mt-1">{workingContent?.testSubstance?.route || '-'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">劑量</dt>
                  <dd className="mt-1">{workingContent?.testSubstance?.dose || '-'}</dd>
                </div>
              </dl>
              {workingContent?.testSubstance?.control_group && (
                <div className="mt-4">
                  <h4 className="font-medium text-sm text-muted-foreground mb-1">對照組設計</h4>
                  <p className="text-sm whitespace-pre-wrap">{workingContent.testSubstance.control_group}</p>
                </div>
              )}
            </CardContent>
          </Card>

          {/* 動物資訊 */}
          <Card>
            <CardHeader>
              <CardTitle>動物資訊</CardTitle>
            </CardHeader>
            <CardContent>
              <dl className="grid gap-4 md:grid-cols-3">
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">物種</dt>
                  <dd className="mt-1">{workingContent?.animalInfo?.species || '-'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">品種</dt>
                  <dd className="mt-1">{workingContent?.animalInfo?.breed || '-'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">性別</dt>
                  <dd className="mt-1">{workingContent?.animalInfo?.gender || '-'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">數量</dt>
                  <dd className="mt-1">{workingContent?.animalInfo?.count || '-'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">年齡</dt>
                  <dd className="mt-1">{workingContent?.animalInfo?.age || '-'}</dd>
                </div>
                <div>
                  <dt className="text-sm font-medium text-muted-foreground">體重範圍</dt>
                  <dd className="mt-1">{workingContent?.animalInfo?.weight || '-'}</dd>
                </div>
              </dl>
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
                  {workingContent?.procedure?.description || '未填寫'}
                </p>
              </div>
              <div>
                <h4 className="font-medium text-sm text-muted-foreground mb-1">麻醉方案</h4>
                <p className="text-sm whitespace-pre-wrap">
                  {workingContent?.procedure?.anesthesia || '未填寫'}
                </p>
              </div>
              <div>
                <h4 className="font-medium text-sm text-muted-foreground mb-1">止痛管理</h4>
                <p className="text-sm whitespace-pre-wrap">
                  {workingContent?.procedure?.pain_management || '未填寫'}
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
                        <TableCell>{pig.penLocation}</TableCell>
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
    </div>
  )
}
