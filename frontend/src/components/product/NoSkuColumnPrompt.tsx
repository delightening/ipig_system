import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Loader2, AlertCircle, Download } from 'lucide-react'

import type { NoSkuColumnPromptProps } from './importTypes'

export function NoSkuColumnPrompt({
  previewMutationIsPending,
  importIsPending,
  hasDuplicates: _hasDuplicates,
  onSetSkuManually,
  onAutoGenerateSku,
  onDownloadTemplate,
}: NoSkuColumnPromptProps) {
  const { t } = useTranslation()

  return (
    <div className="space-y-4 p-4 border border-status-info-border bg-status-info-bg rounded-lg">
      <div className="flex items-center gap-2 text-status-info-text">
        <AlertCircle className="h-5 w-5" />
        <span className="font-medium">{t('erpMaster.import.noSku.title')}</span>
      </div>
      <p className="text-sm text-status-info-text">{t('erpMaster.import.noSku.chooseAction')}</p>
      <div className="flex flex-wrap gap-2">
        <Button
          size="sm"
          onClick={onSetSkuManually}
          disabled={previewMutationIsPending}
          variant="outline"
          className="border-blue-600 text-status-info-text hover:bg-status-info-bg"
        >
          {previewMutationIsPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
          {t('erpMaster.import.noSku.setManually')}
        </Button>
        <Button
          size="sm"
          onClick={onAutoGenerateSku}
          disabled={importIsPending}
          className="bg-primary hover:bg-primary/90"
        >
          {importIsPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
          {t('erpMaster.import.noSku.autoGenerate')}
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={onDownloadTemplate}
          className="border-blue-600 text-status-info-text hover:bg-status-info-bg"
        >
          <Download className="h-4 w-4 mr-1" />
          {t('erpMaster.import.noSku.downloadTemplate')}
        </Button>
      </div>
    </div>
  )
}
