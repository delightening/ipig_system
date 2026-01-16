import { useState, useRef } from 'react'
import { useParams, Link, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import api, {
  ProtocolResponse,
  ProtocolVersion,
  ProtocolStatusHistory,
  ReviewCommentResponse,
  ReviewAssignmentResponse,
  ProtocolAttachment,
  ProtocolStatus,
  protocolStatusNames,
  ChangeStatusRequest,
  CreateCommentRequest,
  AssignReviewerRequest,
  AssignCoEditorRequest,
  CoEditorAssignmentResponse,
  UserSimple,
  User,
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
import { Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { toast } from '@/components/ui/use-toast'
import {
  ArrowLeft,
  Edit,
  Send,
  CheckCircle,
  XCircle,
  Clock,
  History,
  MessageSquare,
  FileText,
  User,
  Building,
  Calendar,
  Loader2,
  AlertTriangle,
  UserPlus,
  Paperclip,
  Upload,
  Download,
  Trash2,
  Eye,
  CheckCircle2,
  Users,
} from 'lucide-react'
import { formatDate, formatDateTime, formatFileSize } from '@/lib/utils'
import { useAuthStore } from '@/stores/auth'
import { ProtocolContentView } from '@/components/protocol/ProtocolContentView'

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

// 狀態轉換規則
const allowedTransitions: Record<ProtocolStatus, ProtocolStatus[]> = {
  DRAFT: ['SUBMITTED'],
  SUBMITTED: ['PRE_REVIEW'],
  PRE_REVIEW: ['UNDER_REVIEW'],
  UNDER_REVIEW: ['REVISION_REQUIRED', 'APPROVED', 'APPROVED_WITH_CONDITIONS', 'REJECTED', 'DEFERRED'],
  REVISION_REQUIRED: ['RESUBMITTED'],
  RESUBMITTED: ['PRE_REVIEW', 'UNDER_REVIEW'],
  APPROVED: ['SUSPENDED', 'CLOSED'],
  APPROVED_WITH_CONDITIONS: ['SUSPENDED', 'CLOSED'],
  DEFERRED: ['UNDER_REVIEW', 'CLOSED'],
  REJECTED: ['CLOSED'],
  SUSPENDED: ['UNDER_REVIEW', 'CLOSED'],
  CLOSED: [],
}

export function ProtocolDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const { user } = useAuthStore()
  const fileInputRef = useRef<HTMLInputElement>(null)
  
  const [activeTab, setActiveTab] = useState<'content' | 'versions' | 'history' | 'comments' | 'reviewers' | 'coeditors' | 'attachments'>('content')
  const [showStatusDialog, setShowStatusDialog] = useState(false)
  const [showCommentDialog, setShowCommentDialog] = useState(false)
  const [showAssignDialog, setShowAssignDialog] = useState(false)
  const [showVersionDialog, setShowVersionDialog] = useState(false)
  const [selectedVersion, setSelectedVersion] = useState<ProtocolVersion | null>(null)
  const [newStatus, setNewStatus] = useState<ProtocolStatus | ''>('')
  const [statusRemark, setStatusRemark] = useState('')
  const [commentContent, setCommentContent] = useState('')
  const [selectedReviewerId, setSelectedReviewerId] = useState('')
  const [selectedCoEditorId, setSelectedCoEditorId] = useState('')

  // 取得計畫詳情
  const { data: protocol, isLoading } = useQuery({
    queryKey: ['protocol', id],
    queryFn: async () => {
      const response = await api.get<ProtocolResponse>(`/protocols/${id}`)
      return response.data
    },
    enabled: !!id,
  })

  // 取得版本列表
  const { data: versions } = useQuery({
    queryKey: ['protocol-versions', id],
    queryFn: async () => {
      const response = await api.get<ProtocolVersion[]>(`/protocols/${id}/versions`)
      return response.data
    },
    enabled: !!id && activeTab === 'versions',
  })

  // 取得狀態歷程
  const { data: statusHistory } = useQuery({
    queryKey: ['protocol-status-history', id],
    queryFn: async () => {
      const response = await api.get<ProtocolStatusHistory[]>(`/protocols/${id}/status-history`)
      return response.data
    },
    enabled: !!id && activeTab === 'history',
  })

  // 取得審查意見
  const { data: comments, isLoading: commentsLoading } = useQuery({
    queryKey: ['protocol-comments', id, versions?.[0]?.id],
    queryFn: async () => {
      if (!versions || versions.length === 0) return []
      const latestVersionId = versions[0].id
      const response = await api.get<ReviewCommentResponse[]>(`/reviews/comments`, {
        params: { protocol_version_id: latestVersionId }
      })
      return response.data
    },
    enabled: !!id && !!versions && versions.length > 0 && activeTab === 'comments',
  })

  // 取得審查指派
  const { data: reviewers, isLoading: reviewersLoading } = useQuery({
    queryKey: ['protocol-reviewers', id],
    queryFn: async () => {
      const response = await api.get<ReviewAssignmentResponse[]>(`/reviews/assignments`, {
        params: { protocol_id: id }
      })
      return response.data
    },
    enabled: !!id && activeTab === 'reviewers',
  })

  // 取得附件
  const { data: attachments, isLoading: attachmentsLoading } = useQuery({
    queryKey: ['protocol-attachments', id],
    queryFn: async () => {
      const response = await api.get<ProtocolAttachment[]>(`/attachments`, {
        params: { protocol_id: id }
      })
      return response.data
    },
    enabled: !!id && activeTab === 'attachments',
  })

  // 取得 co-editor 列表
  const { data: coEditors, isLoading: coEditorsLoading } = useQuery({
    queryKey: ['protocol-co-editors', id],
    queryFn: async () => {
      const response = await api.get<CoEditorAssignmentResponse[]>(`/protocols/${id}/co-editors`)
      return response.data
    },
    enabled: !!id && activeTab === 'coeditors',
  })

  // 取得可指派的審查人員
  const { data: availableReviewers } = useQuery({
    queryKey: ['available-reviewers'],
    queryFn: async () => {
      // 获取所有用户，然后在前端过滤出 REVIEWER 和 VET 角色
      const response = await api.get<User[]>('/users')
      // 过滤出具有 REVIEWER 或 VET 角色的用户
      return response.data
        .filter(user => user.roles?.some(role => ['REVIEWER', 'VET'].includes(role)))
        .map(user => ({
          id: user.id,
          email: user.email,
          display_name: user.display_name || user.email,
        }))
    },
    enabled: showAssignDialog,
  })

  // 取得可指派的試驗工作人員（co-editor）
  const { data: availableExperimentStaff } = useQuery({
    queryKey: ['available-experiment-staff'],
    queryFn: async () => {
      // 获取所有用户，然后在前端过滤出 EXPERIMENT_STAFF 角色
      const response = await api.get<User[]>('/users')
      // 过滤出具有 EXPERIMENT_STAFF 角色的用户
      return response.data
        .filter(user => user.roles?.includes('EXPERIMENT_STAFF'))
        .map(user => ({
          id: user.id,
          email: user.email,
          display_name: user.display_name || user.email,
        }))
    },
    enabled: showStatusDialog && newStatus === 'PRE_REVIEW',
  })

  // 提交計畫
  const submitMutation = useMutation({
    mutationFn: async () => {
      return api.post(`/protocols/${id}/submit`)
    },
    onSuccess: () => {
      toast({ title: '成功', description: '計畫書已提交審查' })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '提交失敗',
        variant: 'destructive',
      })
    },
  })

  // 變更狀態
  const changeStatusMutation = useMutation({
    mutationFn: async (data: ChangeStatusRequest) => {
      return api.post(`/protocols/${id}/status`, data)
    },
    onSuccess: () => {
      toast({ title: '成功', description: '狀態已變更' })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      queryClient.invalidateQueries({ queryKey: ['protocol-status-history', id] })
      setShowStatusDialog(false)
      setNewStatus('')
      setStatusRemark('')
      setSelectedCoEditorId('')
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '狀態變更失敗',
        variant: 'destructive',
      })
    },
  })

  // 新增審查意見
  const addCommentMutation = useMutation({
    mutationFn: async (data: CreateCommentRequest) => {
      return api.post('/reviews/comments', data)
    },
    onSuccess: () => {
      toast({ title: '成功', description: '審查意見已新增' })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', id] })
      setShowCommentDialog(false)
      setCommentContent('')
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '新增意見失敗',
        variant: 'destructive',
      })
    },
  })

  // 解決審查意見
  const resolveCommentMutation = useMutation({
    mutationFn: async (commentId: string) => {
      return api.post(`/reviews/comments/${commentId}/resolve`)
    },
    onSuccess: () => {
      toast({ title: '成功', description: '意見已標記為已解決' })
      queryClient.invalidateQueries({ queryKey: ['protocol-comments', id] })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '操作失敗',
        variant: 'destructive',
      })
    },
  })

  // 指派審查人員
  const assignReviewerMutation = useMutation({
    mutationFn: async (data: AssignReviewerRequest) => {
      return api.post('/reviews/assignments', data)
    },
    onSuccess: () => {
      toast({ title: '成功', description: '審查人員已指派' })
      queryClient.invalidateQueries({ queryKey: ['protocol-reviewers', id] })
      setShowAssignDialog(false)
      setSelectedReviewerId('')
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '指派失敗',
        variant: 'destructive',
      })
    },
  })

  // 指派 co-editor
  const assignCoEditorMutation = useMutation({
    mutationFn: async (data: AssignCoEditorRequest) => {
      return api.post(`/protocols/${id}/co-editors`, data)
    },
    onSuccess: () => {
      toast({ title: '成功', description: 'Co-editor 已指派' })
      queryClient.invalidateQueries({ queryKey: ['protocol', id] })
      queryClient.invalidateQueries({ queryKey: ['protocol-co-editors', id] })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '指派 co-editor 失敗',
        variant: 'destructive',
      })
    },
  })

  // 移除 co-editor
  const removeCoEditorMutation = useMutation({
    mutationFn: async (userId: string) => {
      return api.delete(`/protocols/${id}/co-editors/${userId}`)
    },
    onSuccess: () => {
      toast({ title: '成功', description: 'Co-editor 已移除' })
      queryClient.invalidateQueries({ queryKey: ['protocol-co-editors', id] })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '移除 co-editor 失敗',
        variant: 'destructive',
      })
    },
  })

  // 上傳附件
  const uploadAttachmentMutation = useMutation({
    mutationFn: async (file: File) => {
      const formData = new FormData()
      formData.append('file', file)
      return api.post(`/attachments?protocol_id=${id}`, formData, {
        headers: { 'Content-Type': 'multipart/form-data' }
      })
    },
    onSuccess: () => {
      toast({ title: '成功', description: '附件已上傳' })
      queryClient.invalidateQueries({ queryKey: ['protocol-attachments', id] })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '上傳失敗',
        variant: 'destructive',
      })
    },
  })

  // 刪除附件
  const deleteAttachmentMutation = useMutation({
    mutationFn: async (attachmentId: string) => {
      return api.delete(`/attachments/${attachmentId}`)
    },
    onSuccess: () => {
      toast({ title: '成功', description: '附件已刪除' })
      queryClient.invalidateQueries({ queryKey: ['protocol-attachments', id] })
    },
    onError: (error: any) => {
      toast({
        title: '錯誤',
        description: error?.response?.data?.error?.message || '刪除失敗',
        variant: 'destructive',
      })
    },
  })

  const handleSubmit = () => {
    if (confirm('確定要提交此計畫書進行審查嗎？')) {
      submitMutation.mutate()
    }
  }

  const handleChangeStatus = async () => {
    if (!newStatus) return
    
    // 先變更狀態
    try {
      await changeStatusMutation.mutateAsync({
        to_status: newStatus,
        remark: statusRemark || undefined,
      })
      
      // 如果目標狀態是行政預審且選擇了 co-editor，則指派 co-editor
      if (newStatus === 'PRE_REVIEW' && selectedCoEditorId && id) {
        try {
          await assignCoEditorMutation.mutateAsync({
            protocol_id: id,
            user_id: selectedCoEditorId,
          })
        } catch (error) {
          // co-editor 指派失敗不影響狀態變更，只顯示警告
          toast({
            title: '警告',
            description: '狀態已變更，但指派 co-editor 失敗',
            variant: 'destructive',
          })
        }
      }
    } catch (error) {
      // 錯誤已在 mutation 中處理
    }
  }

  const handleAddComment = () => {
    if (!commentContent.trim() || !versions || versions.length === 0) return
    addCommentMutation.mutate({
      protocol_version_id: versions[0].id,
      content: commentContent.trim(),
    })
  }

  const handleAssignReviewer = () => {
    if (!selectedReviewerId || !id) return
    assignReviewerMutation.mutate({
      protocol_id: id,
      reviewer_id: selectedReviewerId,
    })
  }

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      uploadAttachmentMutation.mutate(file)
    }
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }

  const handleDownloadAttachment = async (attachment: ProtocolAttachment) => {
    try {
      const response = await api.get(`/attachments/${attachment.id}/download`, {
        responseType: 'blob'
      })
      const url = window.URL.createObjectURL(new Blob([response.data]))
      const link = document.createElement('a')
      link.href = url
      link.setAttribute('download', attachment.file_name)
      document.body.appendChild(link)
      link.click()
      link.remove()
    } catch (error) {
      toast({ title: '錯誤', description: '下載失敗', variant: 'destructive' })
    }
  }

  const getAvailableTransitions = () => {
    if (!protocol) return []
    return allowedTransitions[protocol.status] || []
  }

  const canAddComment = user?.roles?.some(r => ['REVIEWER', 'VET', 'CHAIR', 'IACUC_STAFF', 'SYSTEM_ADMIN'].includes(r))
  const canAssignReviewer = user?.roles?.some(r => ['IACUC_STAFF', 'CHAIR', 'SYSTEM_ADMIN'].includes(r))
  const canManageAttachments = protocol?.status === 'DRAFT' || protocol?.status === 'REVISION_REQUIRED'

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
        <h2 className="text-xl font-semibold mb-2">找不到計畫書</h2>
        <p className="text-muted-foreground mb-4">此計畫書不存在或您沒有權限查看</p>
        <Button asChild>
          <Link to="/protocols">返回列表</Link>
        </Button>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={() => navigate('/my-projects')}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <div className="flex items-center gap-3">
              <h1 className="text-2xl font-bold">{protocol.title}</h1>
              <Badge variant={statusColors[protocol.status]} className="text-sm">
                {protocolStatusNames[protocol.status]}
              </Badge>
            </div>
          </div>
        </div>
        <div className="flex gap-2">
          {protocol.status === 'DRAFT' && (
            <>
              <Button variant="outline" asChild>
                <Link to={`/protocols/${id}/edit`}>
                  <Edit className="mr-2 h-4 w-4" />
                  編輯
                </Link>
              </Button>
              <Button onClick={handleSubmit} disabled={submitMutation.isPending}>
                {submitMutation.isPending ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <Send className="mr-2 h-4 w-4" />
                )}
                提交審查
              </Button>
            </>
          )}
          {protocol.status === 'REVISION_REQUIRED' && (
            <Button variant="outline" asChild>
              <Link to={`/protocols/${id}/edit`}>
                <Edit className="mr-2 h-4 w-4" />
                修訂計畫
              </Link>
            </Button>
          )}
          {getAvailableTransitions().length > 0 && protocol.status !== 'DRAFT' && (
            <Button variant="outline" onClick={() => setShowStatusDialog(true)}>
              變更狀態
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
            <p className="text-sm text-muted-foreground">{protocol.pi_email}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Building className="h-4 w-4 text-purple-500" />
              所屬單位
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
        <nav className="flex gap-4 overflow-x-auto">
          {[
            { key: 'content', label: '計畫內容', icon: FileText },
            { key: 'versions', label: '版本記錄', icon: History },
            { key: 'history', label: '狀態歷程', icon: Clock },
            { key: 'comments', label: '審查意見', icon: MessageSquare },
            { key: 'reviewers', label: '審查人員', icon: Users },
            { key: 'coeditors', label: 'Co-Editor', icon: UserPlus },
            { key: 'attachments', label: '附件', icon: Paperclip },
          ].map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveTab(tab.key as typeof activeTab)}
              className={`flex items-center gap-2 px-4 py-3 border-b-2 transition-colors whitespace-nowrap ${
                activeTab === tab.key
                  ? 'border-blue-600 text-blue-600'
                  : 'border-transparent text-muted-foreground hover:text-foreground'
              }`}
            >
              <tab.icon className="h-4 w-4" />
              {tab.label}
            </button>
          ))}
        </nav>
      </div>

      {/* Tab Content */}
      {activeTab === 'content' && (
        <Card>
          <CardHeader>
            <CardTitle>計畫書內容</CardTitle>
            <CardDescription>AUP 動物試驗計畫書詳細內容</CardDescription>
          </CardHeader>
          <CardContent>
            <ProtocolContentView 
              workingContent={(() => {
                if (!protocol.working_content) return null
                // 創建一個副本，去除 apply_study_number 字段
                const cleanedContent = JSON.parse(JSON.stringify(protocol.working_content))
                if (cleanedContent.basic && cleanedContent.basic.apply_study_number !== undefined) {
                  delete cleanedContent.basic.apply_study_number
                }
                return cleanedContent
              })()}
              protocolTitle={protocol.title}
              startDate={protocol.start_date}
              endDate={protocol.end_date}
            />
          </CardContent>
        </Card>
      )}

      {activeTab === 'versions' && (
        <Card>
          <CardHeader>
            <CardTitle>版本記錄</CardTitle>
            <CardDescription>計畫書每次提交的版本快照</CardDescription>
          </CardHeader>
          <CardContent>
            {versions && versions.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>版本號</TableHead>
                    <TableHead>提交時間</TableHead>
                    <TableHead>操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {versions.map((version) => (
                    <TableRow key={version.id}>
                      <TableCell className="font-medium">v{version.version_no}</TableCell>
                      <TableCell>{formatDateTime(version.submitted_at)}</TableCell>
                      <TableCell>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => {
                            setSelectedVersion(version)
                            setShowVersionDialog(true)
                          }}
                        >
                          <Eye className="mr-1 h-4 w-4" />
                          查看內容
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <History className="h-12 w-12 mx-auto mb-2" />
                <p>尚無版本記錄</p>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === 'history' && (
        <Card>
          <CardHeader>
            <CardTitle>狀態歷程</CardTitle>
            <CardDescription>計畫書狀態變更記錄</CardDescription>
          </CardHeader>
          <CardContent>
            {statusHistory && statusHistory.length > 0 ? (
              <div className="relative">
                <div className="absolute left-4 top-0 bottom-0 w-0.5 bg-slate-200" />
                <ul className="space-y-4">
                  {statusHistory.map((history, index) => (
                    <li key={history.id} className="relative pl-10">
                      <div className="absolute left-2 w-4 h-4 rounded-full bg-blue-600 border-2 border-white" />
                      <div className="bg-slate-50 p-4 rounded-lg">
                        <div className="flex items-center gap-2 mb-2">
                          {history.from_status && (
                            <>
                              <Badge variant={statusColors[history.from_status]}>
                                {protocolStatusNames[history.from_status]}
                              </Badge>
                              <span className="text-muted-foreground">→</span>
                            </>
                          )}
                          <Badge variant={statusColors[history.to_status]}>
                            {protocolStatusNames[history.to_status]}
                          </Badge>
                        </div>
                        <p className="text-sm text-muted-foreground">
                          {formatDateTime(history.created_at)}
                        </p>
                        {history.remark && (
                          <p className="text-sm mt-2 text-slate-600">{history.remark}</p>
                        )}
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <Clock className="h-12 w-12 mx-auto mb-2" />
                <p>尚無狀態變更記錄</p>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === 'comments' && (
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle>審查意見</CardTitle>
              <CardDescription>審查委員的意見與回覆</CardDescription>
            </div>
            {canAddComment && protocol.status !== 'DRAFT' && (
              <Button onClick={() => setShowCommentDialog(true)}>
                <MessageSquare className="mr-2 h-4 w-4" />
                新增意見
              </Button>
            )}
          </CardHeader>
          <CardContent>
            {commentsLoading ? (
              <div className="flex justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : comments && comments.length > 0 ? (
              <div className="space-y-4">
                {comments.map((comment) => (
                  <div
                    key={comment.id}
                    className={`p-4 rounded-lg border ${comment.is_resolved ? 'bg-green-50 border-green-200' : 'bg-white border-slate-200'}`}
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex items-center gap-2">
                        <div className="h-8 w-8 rounded-full bg-blue-100 flex items-center justify-center">
                          <User className="h-4 w-4 text-blue-600" />
                        </div>
                        <div>
                          <p className="font-medium">{comment.reviewer_name || comment.reviewer_email}</p>
                          <p className="text-xs text-muted-foreground">{formatDateTime(comment.created_at)}</p>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        {comment.is_resolved ? (
                          <Badge variant="success" className="flex items-center gap-1">
                            <CheckCircle2 className="h-3 w-3" />
                            已解決
                          </Badge>
                        ) : (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => resolveCommentMutation.mutate(comment.id)}
                            disabled={resolveCommentMutation.isPending}
                          >
                            {resolveCommentMutation.isPending ? (
                              <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                              <>
                                <CheckCircle className="mr-1 h-4 w-4" />
                                標記已解決
                              </>
                            )}
                          </Button>
                        )}
                      </div>
                    </div>
                    <p className="mt-3 text-sm text-slate-700 whitespace-pre-wrap">{comment.content}</p>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <MessageSquare className="h-12 w-12 mx-auto mb-2" />
                <p>尚無審查意見</p>
                {canAddComment && protocol.status !== 'DRAFT' && (
                  <Button variant="link" onClick={() => setShowCommentDialog(true)} className="mt-2">
                    新增第一條審查意見
                  </Button>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === 'reviewers' && (
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle>審查人員</CardTitle>
              <CardDescription>指派的審查委員與獸醫師</CardDescription>
            </div>
            {canAssignReviewer && protocol.status !== 'DRAFT' && protocol.status !== 'CLOSED' && (
              <Button onClick={() => setShowAssignDialog(true)}>
                <UserPlus className="mr-2 h-4 w-4" />
                指派審查人員
              </Button>
            )}
          </CardHeader>
          <CardContent>
            {reviewersLoading ? (
              <div className="flex justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : reviewers && reviewers.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>審查人員</TableHead>
                    <TableHead>指派時間</TableHead>
                    <TableHead>指派者</TableHead>
                    <TableHead>完成時間</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {reviewers.map((reviewer) => (
                    <TableRow key={reviewer.id}>
                      <TableCell>
                        <div>
                          <p className="font-medium">{reviewer.reviewer_name || '-'}</p>
                          <p className="text-sm text-muted-foreground">{reviewer.reviewer_email}</p>
                        </div>
                      </TableCell>
                      <TableCell>{formatDateTime(reviewer.assigned_at)}</TableCell>
                      <TableCell>{reviewer.assigned_by_name || '-'}</TableCell>
                      <TableCell>
                        {reviewer.completed_at ? (
                          <Badge variant="success">{formatDateTime(reviewer.completed_at)}</Badge>
                        ) : (
                          <Badge variant="secondary">審查中</Badge>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <Users className="h-12 w-12 mx-auto mb-2" />
                <p>尚未指派審查人員</p>
                {canAssignReviewer && protocol.status !== 'DRAFT' && (
                  <Button variant="link" onClick={() => setShowAssignDialog(true)} className="mt-2">
                    指派第一位審查人員
                  </Button>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === 'coeditors' && (
        <Card>
          <CardHeader>
            <CardTitle>Co-Editor</CardTitle>
            <CardDescription>指派的試驗工作人員（co-editor）列表</CardDescription>
          </CardHeader>
          <CardContent>
            {coEditorsLoading ? (
              <div className="flex justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : coEditors && coEditors.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Co-Editor</TableHead>
                    <TableHead>指派時間</TableHead>
                    <TableHead>指派者</TableHead>
                    <TableHead>操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {coEditors.map((coEditor) => (
                    <TableRow key={coEditor.user_id}>
                      <TableCell>
                        <div>
                          <p className="font-medium">{coEditor.user_name}</p>
                          <p className="text-sm text-muted-foreground">{coEditor.user_email}</p>
                        </div>
                      </TableCell>
                      <TableCell>{formatDateTime(coEditor.granted_at)}</TableCell>
                      <TableCell>{coEditor.granted_by_name || '-'}</TableCell>
                      <TableCell>
                        {canAssignReviewer && (
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                              if (confirm('確定要移除此 co-editor 嗎？')) {
                                removeCoEditorMutation.mutate(coEditor.user_id)
                              }
                            }}
                            disabled={removeCoEditorMutation.isPending}
                          >
                            {removeCoEditorMutation.isPending ? (
                              <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                              <>
                                <Trash2 className="h-4 w-4 text-destructive" />
                              </>
                            )}
                          </Button>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <UserPlus className="h-12 w-12 mx-auto mb-2" />
                <p>尚未指派 co-editor</p>
                <p className="text-sm mt-2">可在狀態變更為「行政預審」時指派 co-editor</p>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {activeTab === 'attachments' && (
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle>附件</CardTitle>
              <CardDescription>計畫書相關文件與檔案</CardDescription>
            </div>
            {canManageAttachments && (
              <>
                <input
                  type="file"
                  ref={fileInputRef}
                  onChange={handleFileUpload}
                  className="hidden"
                />
                <Button onClick={() => fileInputRef.current?.click()} disabled={uploadAttachmentMutation.isPending}>
                  {uploadAttachmentMutation.isPending ? (
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  ) : (
                    <Upload className="mr-2 h-4 w-4" />
                  )}
                  上傳附件
                </Button>
              </>
            )}
          </CardHeader>
          <CardContent>
            {attachmentsLoading ? (
              <div className="flex justify-center py-8">
                <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              </div>
            ) : attachments && attachments.length > 0 ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>檔案名稱</TableHead>
                    <TableHead>大小</TableHead>
                    <TableHead>上傳者</TableHead>
                    <TableHead>上傳時間</TableHead>
                    <TableHead>操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {attachments.map((attachment) => (
                    <TableRow key={attachment.id}>
                      <TableCell>
                        <div className="flex items-center gap-2">
                          <Paperclip className="h-4 w-4 text-muted-foreground" />
                          <span className="font-medium">{attachment.file_name}</span>
                        </div>
                      </TableCell>
                      <TableCell>{formatFileSize(attachment.file_size)}</TableCell>
                      <TableCell>{attachment.uploaded_by_name || '-'}</TableCell>
                      <TableCell>{formatDateTime(attachment.created_at)}</TableCell>
                      <TableCell>
                        <div className="flex gap-2">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleDownloadAttachment(attachment)}
                          >
                            <Download className="h-4 w-4" />
                          </Button>
                          {canManageAttachments && (
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => {
                                if (confirm('確定要刪除此附件嗎？')) {
                                  deleteAttachmentMutation.mutate(attachment.id)
                                }
                              }}
                              disabled={deleteAttachmentMutation.isPending}
                            >
                              <Trash2 className="h-4 w-4 text-destructive" />
                            </Button>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <div className="text-center py-8 text-muted-foreground">
                <Paperclip className="h-12 w-12 mx-auto mb-2" />
                <p>尚無附件</p>
                {canManageAttachments && (
                  <Button variant="link" onClick={() => fileInputRef.current?.click()} className="mt-2">
                    上傳第一個附件
                  </Button>
                )}
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* 狀態變更對話框 */}
      <Dialog open={showStatusDialog} onOpenChange={setShowStatusDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>變更計畫狀態</DialogTitle>
            <DialogDescription>
              選擇要變更的目標狀態，並可選填備註說明
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>目前狀態</Label>
              <Badge variant={statusColors[protocol.status]} className="text-sm">
                {protocolStatusNames[protocol.status]}
              </Badge>
            </div>
            <div className="space-y-2">
              <Label>目標狀態</Label>
              <Select value={newStatus} onValueChange={(v) => setNewStatus(v as ProtocolStatus)}>
                <SelectTrigger>
                  <SelectValue placeholder="請選擇目標狀態" />
                </SelectTrigger>
                <SelectContent>
                  {getAvailableTransitions().map((status) => (
                    <SelectItem key={status} value={status}>
                      {protocolStatusNames[status]}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            {newStatus === 'PRE_REVIEW' && (
              <div className="space-y-2">
                <Label>指定 Co-Editor（選填）</Label>
                <Select value={selectedCoEditorId} onValueChange={setSelectedCoEditorId}>
                  <SelectTrigger>
                    <SelectValue placeholder="請選擇試驗工作人員作為 co-editor" />
                  </SelectTrigger>
                  <SelectContent>
                    {availableExperimentStaff?.map((staff) => (
                      <SelectItem key={staff.id} value={staff.id}>
                        {staff.display_name || staff.email}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  可選擇一位試驗工作人員作為 co-editor，協助編輯此計畫
                </p>
              </div>
            )}
            <div className="space-y-2">
              <Label>備註說明（選填）</Label>
              <Textarea
                value={statusRemark}
                onChange={(e) => setStatusRemark(e.target.value)}
                placeholder="輸入狀態變更的原因或說明..."
                rows={3}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowStatusDialog(false)}>
              取消
            </Button>
            <Button
              onClick={handleChangeStatus}
              disabled={!newStatus || changeStatusMutation.isPending || assignCoEditorMutation.isPending}
            >
              {(changeStatusMutation.isPending || assignCoEditorMutation.isPending) && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              確認變更
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 新增審查意見對話框 */}
      <Dialog open={showCommentDialog} onOpenChange={setShowCommentDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>新增審查意見</DialogTitle>
            <DialogDescription>
              請輸入您對此計畫書的審查意見
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>審查意見</Label>
              <Textarea
                value={commentContent}
                onChange={(e) => setCommentContent(e.target.value)}
                placeholder="請輸入詳細的審查意見..."
                rows={5}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCommentDialog(false)}>
              取消
            </Button>
            <Button
              onClick={handleAddComment}
              disabled={!commentContent.trim() || addCommentMutation.isPending}
            >
              {addCommentMutation.isPending && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              提交意見
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 指派審查人員對話框 */}
      <Dialog open={showAssignDialog} onOpenChange={setShowAssignDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>指派審查人員</DialogTitle>
            <DialogDescription>
              選擇要指派的審查委員或獸醫師
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>審查人員</Label>
              <Select value={selectedReviewerId} onValueChange={setSelectedReviewerId}>
                <SelectTrigger>
                  <SelectValue placeholder="請選擇審查人員" />
                </SelectTrigger>
                <SelectContent>
                  {availableReviewers?.map((reviewer) => (
                    <SelectItem key={reviewer.id} value={reviewer.id}>
                      {reviewer.display_name || reviewer.email}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAssignDialog(false)}>
              取消
            </Button>
            <Button
              onClick={handleAssignReviewer}
              disabled={!selectedReviewerId || assignReviewerMutation.isPending}
            >
              {assignReviewerMutation.isPending && (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              )}
              確認指派
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 版本內容檢視對話框 */}
      <Dialog open={showVersionDialog} onOpenChange={setShowVersionDialog}>
        <DialogContent className="max-w-3xl max-h-[80vh]">
          <DialogHeader>
            <DialogTitle>版本內容 - v{selectedVersion?.version_no}</DialogTitle>
            <DialogDescription>
              提交時間：{selectedVersion && formatDateTime(selectedVersion.submitted_at)}
            </DialogDescription>
          </DialogHeader>
          <div className="overflow-auto max-h-[60vh]">
            {selectedVersion?.content_snapshot ? (
              <pre className="bg-slate-50 p-4 rounded-lg text-sm">
                {JSON.stringify(selectedVersion.content_snapshot, null, 2)}
              </pre>
            ) : (
              <p className="text-center text-muted-foreground py-8">無內容</p>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowVersionDialog(false)}>
              關閉
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
