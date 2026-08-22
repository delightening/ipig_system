import { renderToStaticMarkup } from 'react-dom/server'
import { I18nextProvider } from 'react-i18next'

import i18n from '@/lib/i18n'
import { openPrintWindow } from '@/lib/printHtml'
import { toast } from '@/components/ui/use-toast'
import type { ProtocolWorkingContent } from '@/types/protocol'

import { ProtocolPrintableContent } from '../ProtocolPrintableContent'

interface UseProtocolBrowserPrintOptions {
  workingContent?: ProtocolWorkingContent
  protocolTitle: string
  startDate?: string
  endDate?: string
}

/**
 * AUP 計畫書「瀏覽器列印」：純前端把計畫內容以 React 渲染成 HTML，開新分頁後由
 * 瀏覽器原生列印（不經 print-pdf / WeasyPrint，亦不下載 PDF）。
 *
 * 用 `renderToStaticMarkup` + `I18nextProvider`（已初始化的全域 i18n instance）渲染
 * 共用的 `ProtocolPrintableContent`，故版面與 live 計畫內容分頁同源。
 */
export function useProtocolBrowserPrint({
  workingContent,
  protocolTitle,
  startDate,
  endDate,
}: UseProtocolBrowserPrintOptions) {
  const handleBrowserPrint = () => {
    if (!workingContent) return

    const bodyHtml = renderToStaticMarkup(
      <I18nextProvider i18n={i18n}>
        <ProtocolPrintableContent
          workingContent={workingContent}
          protocolTitle={protocolTitle}
          startDate={startDate}
          endDate={endDate}
          className="max-w-none"
        />
      </I18nextProvider>,
    )

    try {
      openPrintWindow({
        bodyHtml,
        title: `${protocolTitle || i18n.t('protocols.content.title')}_AUP`,
      })
    } catch {
      toast({
        variant: 'destructive',
        title: i18n.t('protocols.content.browserPrintBlocked'),
      })
    }
  }

  return { handleBrowserPrint }
}
