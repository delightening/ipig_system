import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from '@/components/ui/table'
import { BloodTestListItem } from '@/lib/api'
import {
    Loader2,
    Plus,
    Eye,
    Edit2,
    Trash2,
    AlertCircle,
    FileText,
} from 'lucide-react'

interface BloodTestListCardProps {
    bloodTests: BloodTestListItem[]
    isLoading: boolean
    onCreateClick: () => void
    onViewClick: (id: string) => void
    onEditClick: (id: string) => void
    onDeleteClick: (id: string, date: string) => void
}

export function BloodTestListCard({
    bloodTests,
    isLoading,
    onCreateClick,
    onViewClick,
    onEditClick,
    onDeleteClick,
}: BloodTestListCardProps) {
    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between">
                <div>
                    <CardTitle>血液檢查紀錄</CardTitle>
                    <CardDescription>記錄實驗動物的血液檢查結果與檢驗數據</CardDescription>
                </div>
                <Button className="bg-red-600 hover:bg-red-700 shrink-0" onClick={onCreateClick}>
                    <Plus className="h-4 w-4 mr-2" />
                    新增血液檢查
                </Button>
            </CardHeader>
            <CardContent>
                {isLoading ? (
                    <div className="flex justify-center py-8">
                        <Loader2 className="h-8 w-8 animate-spin text-gray-400" />
                    </div>
                ) : bloodTests.length === 0 ? (
                    <div className="text-center py-12 text-slate-500">
                        <FileText className="h-12 w-12 mx-auto mb-3 text-slate-300" />
                        <p>尚無血液檢查紀錄</p>
                        <p className="text-sm mt-1">點擊上方按鈕新增</p>
                    </div>
                ) : (
                    <Table>
                        <TableHeader>
                            <TableRow>
                                <TableHead>檢查日期</TableHead>
                                <TableHead>檢驗機構</TableHead>
                                <TableHead className="text-center">項目數</TableHead>
                                <TableHead className="text-center">異常項目</TableHead>
                                <TableHead>建立者</TableHead>
                                <TableHead className="text-right">操作</TableHead>
                            </TableRow>
                        </TableHeader>
                        <TableBody>
                            {bloodTests.map((test) => (
                                <TableRow key={test.id}>
                                    <TableCell className="font-medium">{test.test_date}</TableCell>
                                    <TableCell>{test.lab_name || '-'}</TableCell>
                                    <TableCell className="text-center">{test.item_count}</TableCell>
                                    <TableCell className="text-center">
                                        {test.abnormal_count > 0 ? (
                                            <Badge variant="destructive" className="gap-1">
                                                <AlertCircle className="h-3 w-3" />
                                                {test.abnormal_count}
                                            </Badge>
                                        ) : (
                                            <span className="text-green-600">0</span>
                                        )}
                                    </TableCell>
                                    <TableCell>{test.created_by_name || '-'}</TableCell>
                                    <TableCell className="text-right space-x-1">
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            onClick={() => onViewClick(test.id)}
                                        >
                                            <Eye className="h-4 w-4" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            onClick={() => onEditClick(test.id)}
                                        >
                                            <Edit2 className="h-4 w-4" />
                                        </Button>
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            className="text-red-600"
                                            onClick={() => onDeleteClick(test.id, test.test_date)}
                                        >
                                            <Trash2 className="h-4 w-4" />
                                        </Button>
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                )}
            </CardContent>
        </Card>
    )
}
