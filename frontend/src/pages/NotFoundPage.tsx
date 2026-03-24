import { Link } from 'react-router-dom'
import { FileQuestion, Home, ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useTranslation } from 'react-i18next'

export function NotFoundPage() {
  const { t } = useTranslation()

  return (
    <div className="min-h-screen flex items-center justify-center bg-gradient-to-br from-secondary to-background px-4">
      <div className="text-center max-w-md">
        <FileQuestion className="h-16 w-16 text-muted-foreground/50 mx-auto mb-6" strokeWidth={1.5} />
        <h1 className="text-6xl font-bold text-foreground mb-2">404</h1>
        <h2 className="text-xl font-semibold text-muted-foreground mb-4">
          {t('notFound.title', '找不到此頁面')}
        </h2>
        <p className="text-muted-foreground mb-8">
          {t('notFound.description', '您要找的頁面不存在、已移除，或網址輸入錯誤。')}
        </p>
        <div className="flex gap-3 justify-center">
          <Button variant="outline" onClick={() => window.history.back()}>
            <ArrowLeft className="h-4 w-4 mr-2" />
            {t('notFound.goBack', '返回上一頁')}
          </Button>
          <Link to="/">
            <Button>
              <Home className="h-4 w-4 mr-2" />
              {t('notFound.goHome', '回到首頁')}
            </Button>
          </Link>
        </div>
      </div>
    </div>
  )
}
