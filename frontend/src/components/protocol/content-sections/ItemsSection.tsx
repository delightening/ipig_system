import { Label } from '@/components/ui/label'
import { useTranslation } from 'react-i18next'
import type { ProtocolWorkingContent } from '@/types/protocol'

type TestItem = ProtocolWorkingContent['items']['test_items'][number]
type ControlItem = ProtocolWorkingContent['items']['control_items'][number]

interface ItemsSectionProps {
  items: ProtocolWorkingContent['items']
}

export function ItemsSection({ items }: ItemsSectionProps) {
  const { t } = useTranslation()

  return (
    <section className="mb-8 border-t pt-6 section-3" data-section={t('protocols.content.sections.items')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.items')}</h2>

      {items.use_test_item === true ? (
        <>
          {items.test_items && items.test_items.length > 0 && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.testItems')}</h3>
              {items.test_items.map((item: TestItem, index: number) => (
                <div key={index} className="mb-4 p-4 border rounded bg-muted">
                  <h4 className="font-medium mb-2">{t('protocols.content.sections.testItems')} #{index + 1}</h4>
                  <div className="grid grid-cols-2 gap-3 text-sm">
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.itemName')}: </Label>
                      <span>{item.name || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.itemForm')}: </Label>
                      <span>{item.form || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.itemPurpose')}: </Label>
                      <span>{item.purpose || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.itemStorage')}: </Label>
                      <span>{item.storage_conditions || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.isSterile')}: </Label>
                      <span>{item.is_sterile ? t('protocols.content.sections.sterileYes') : t('protocols.content.sections.sterileNo')}</span>
                    </div>
                    {!item.is_sterile && item.non_sterile_justification && (
                      <div className="col-span-2">
                        <Label className="font-semibold">{t('protocols.content.sections.nonSterileJustification')} </Label>
                        <p className="mt-1 text-sm whitespace-pre-wrap">{item.non_sterile_justification}</p>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}

          {items.control_items && items.control_items.length > 0 && (
            <div className="mb-4">
              <h3 className="text-lg font-semibold mb-3">{t('protocols.content.sections.controlItems')}</h3>
              {items.control_items.map((item: ControlItem, index: number) => (
                <div key={index} className="mb-4 p-4 border rounded bg-muted">
                  <h4 className="font-medium mb-2">{t('protocols.content.sections.controlItems')} #{index + 1}</h4>
                  <div className="grid grid-cols-2 gap-3 text-sm">
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.itemName')}: </Label>
                      <span>{item.name || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.itemPurpose')}: </Label>
                      <span>{item.purpose || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.itemStorage')}: </Label>
                      <span>{item.storage_conditions || '-'}</span>
                    </div>
                    <div>
                      <Label className="font-semibold">{t('protocols.content.sections.isSterile')}: </Label>
                      <span>{item.is_sterile ? t('protocols.content.sections.sterileYes') : t('protocols.content.sections.sterileNo')}</span>
                    </div>
                    {!item.is_sterile && item.non_sterile_justification && (
                      <div className="col-span-2">
                        <Label className="font-semibold">{t('protocols.content.sections.nonSterileJustification')} </Label>
                        <p className="mt-1 text-sm whitespace-pre-wrap">{item.non_sterile_justification}</p>
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </>
      ) : (
        <p className="text-sm text-muted-foreground">{t('protocols.content.sections.omitted')}</p>
      )}
    </section>
  )
}
