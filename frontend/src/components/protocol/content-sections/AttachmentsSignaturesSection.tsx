import { useTranslation } from 'react-i18next'
import type { FileInfo } from '@/components/ui/file-upload'

interface AttachmentsSignaturesSectionProps {
  attachments: FileInfo[]
  signature: FileInfo[]
}

export function AttachmentsSignaturesSection({ attachments, signature }: AttachmentsSignaturesSectionProps) {
  const { t } = useTranslation()

  return (
    <>
      {/* 9. Attachments */}
      {attachments.length > 0 && (
        <section className="mb-8 border-t pt-6 section-9" data-section={t('protocols.content.sections.attachments')}>
          <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.attachments')}</h2>

          <div className="space-y-2">
            {attachments.map((attachment: FileInfo, index: number) => (
              <div key={index} className="p-3 border rounded">
                <p className="text-sm">{index + 1}. {attachment.file_name || '-'}</p>
              </div>
            ))}
          </div>
        </section>
      )}

      {/* 10. Electronic Signatures */}
      {signature.length > 0 && (
        <section className="mb-8 border-t pt-6 section-10" data-section={t('protocols.content.sections.signatures')}>
          <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.signatures')}</h2>

          <div className="space-y-4">
            {signature.map((sig: FileInfo, index: number) => (
              <div key={index} className="p-4 border rounded">
                {sig.preview_url ? (
                  <img src={sig.preview_url} alt={sig.file_name || `${t('protocols.content.sections.signatures')} ${index + 1}`} className="max-w-xs max-h-32 object-contain" />
                ) : (
                  <p className="text-sm">{index + 1}. {sig.file_name || '-'}</p>
                )}
              </div>
            ))}
          </div>
        </section>
      )}
    </>
  )
}
