import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Loader2, AlertTriangle } from 'lucide-react'

import type { DuplicateWarningProps } from './importTypes'

export function DuplicateWarning({
  checkResult,
  importIsPending,
  onSkipDuplicates,
  onImportWithNewSku,
  onImportAnyway,
}: DuplicateWarningProps) {
  const { t } = useTranslation()

  return (
    <div className="space-y-4 p-4 border border-status-warning-border bg-status-warning-bg rounded-lg">
      <div className="flex items-center gap-2 text-status-warning-text">
        <AlertTriangle className="h-5 w-5" />
        <span className="font-medium">{t('erpMaster.import.duplicate.found', { count: checkResult.duplicate_count })}</span>
      </div>
      <p className="text-sm text-status-warning-text">
        {t('erpMaster.import.duplicate.description')}
      </p>
      <div className="max-h-32 overflow-y-auto border border-status-warning-border rounded bg-white">
        <table className="w-full text-sm">
          <thead className="bg-status-warning-bg sticky top-0">
            <tr>
              <th className="px-3 py-2 text-left font-medium">{t('erpMaster.import.result.row')}</th>
              <th className="px-3 py-2 text-left font-medium">{t('erpMaster.common.name')}</th>
              <th className="px-3 py-2 text-left font-medium">{t('erpMaster.common.spec')}</th>
              <th className="px-3 py-2 text-left font-medium">{t('erpMaster.import.duplicate.existingSku')}</th>
            </tr>
          </thead>
          <tbody>
            {checkResult.duplicates.map((d, i) => (
              <tr key={`dup-${d.row}-${i}`} className="border-t">
                <td className="px-3 py-2">{d.row}</td>
                <td className="px-3 py-2">{d.name}</td>
                <td className="px-3 py-2">{d.spec ?? '-'}</td>
                <td className="px-3 py-2 font-mono">{d.existing_sku}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="flex flex-wrap gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={onSkipDuplicates}
          disabled={importIsPending}
          className="border-amber-600 text-status-warning-text hover:bg-status-warning-bg"
        >
          {importIsPending ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : null}
          {t('erpMaster.import.duplicate.skip')}
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={onImportWithNewSku}
          disabled={importIsPending}
          className="border-amber-600 text-status-warning-text hover:bg-status-warning-bg"
        >
          {t('erpMaster.import.duplicate.importWithNewSku')}
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={onImportAnyway}
          disabled={importIsPending}
        >
          {t('erpMaster.import.duplicate.importAnyway')}
        </Button>
      </div>
    </div>
  )
}
