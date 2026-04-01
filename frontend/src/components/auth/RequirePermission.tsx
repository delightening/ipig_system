import { Navigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/auth'
import { AlertTriangle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

interface RequirePermissionProps {
    children: React.ReactNode
    /** 需要的權限代碼 */
    permission?: string
    /** 需要的角色 */
    role?: string
    /** 任一符合即可的權限或角色列表 */
    anyOf?: { permission?: string; role?: string }[]
    /** 無權限時的行為：redirect 導向首頁，message 顯示提示 */
    fallback?: 'redirect' | 'message'
    /** 自定義導向路徑 */
    redirectTo?: string
}

/**
 * 權限保護組件
 * 用於在頁面級別檢查使用者是否有權限訪問
 */
export function RequirePermission({
    children,
    permission,
    role,
    anyOf,
    fallback = 'message',
    redirectTo = '/dashboard',
}: RequirePermissionProps) {
    const { hasPermission, hasRole, isGuest } = useAuthStore()

    // 檢查權限
    const checkAccess = (): boolean => {
        // Guest 全通行
        if (isGuest()) return true

        // 如果指定了 anyOf，只要任一符合即可
        if (anyOf && anyOf.length > 0) {
            return anyOf.some(item => {
                if (item.permission && hasPermission(item.permission)) return true
                if (item.role && hasRole(item.role)) return true
                return false
            })
        }

        // 檢查單一權限
        if (permission && hasPermission(permission)) return true
        // 檢查單一角色
        if (role && hasRole(role)) return true
        // 如果沒有指定任何條件，預設有權限
        if (!permission && !role && (!anyOf || anyOf.length === 0)) return true

        return false
    }

    const hasAccess = checkAccess()

    if (!hasAccess) {
        if (fallback === 'redirect') {
            return <Navigate to={redirectTo} replace />
        }

        // 顯示無權限訊息
        return (
            <div className="container mx-auto py-12 flex items-center justify-center">
                <Card className="max-w-md">
                    <CardHeader className="text-center">
                        <div className="mx-auto mb-4 w-16 h-16 rounded-full bg-status-warning-bg flex items-center justify-center">
                            <AlertTriangle className="h-8 w-8 text-status-warning-text" />
                        </div>
                        <CardTitle>無權限訪問</CardTitle>
                        <CardDescription>
                            您沒有權限訪問此頁面。如需訪問請聯繫系統管理員。
                        </CardDescription>
                    </CardHeader>
                    <CardContent className="text-center">
                        <Button
                            variant="outline"
                            onClick={() => window.history.back()}
                        >
                            返回上一頁
                        </Button>
                    </CardContent>
                </Card>
            </div>
        )
    }

    return <>{children}</>
}

export default RequirePermission
