import { useTranslation } from 'react-i18next'
import { Input, Textarea } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Repeater } from '@/components/ui/repeater'
import { Checkbox } from '@/components/ui/checkbox'
import { DrugCombobox } from '@/components/animal/DrugCombobox'
import { CollapsibleSection, DrugCheckInput } from './SurgeryFormComponents'
import { POSTURE_OPTIONS, type SurgeryFormData, type MedicationItem, type VitalSign } from './useSurgeryForm'

interface Props {
  formData: SurgeryFormData
  onChange: (data: SurgeryFormData) => void
}

export function SurgeryAnesthesiaSection({ formData, onChange }: Props) {
  const { t } = useTranslation()
  return (
    <>
      {/* 誘導麻醉 */}
      <CollapsibleSection title={t('animalRecords.surgeries.inductionAnesthesia')}>
        <div className="space-y-3">
          <DrugCheckInput
            label="Atropine"
            drug={formData.induction.atropine}
            onChange={(drug) =>
              onChange({
                ...formData,
                induction: { ...formData.induction, atropine: drug },
              })
            }
          />
          <DrugCheckInput
            label="Stroless"
            drug={formData.induction.stroless}
            onChange={(drug) =>
              onChange({
                ...formData,
                induction: { ...formData.induction, stroless: drug },
              })
            }
          />
          <DrugCheckInput
            label="Zoletil-50"
            drug={formData.induction.zoletil50}
            onChange={(drug) =>
              onChange({
                ...formData,
                induction: { ...formData.induction, zoletil50: drug },
              })
            }
          />
          <div className="pt-2">
            <Label className="text-sm text-muted-foreground">{t('animalRecords.surgeries.otherAgents')}</Label>
            <Repeater<MedicationItem>
              value={formData.induction.others}
              onChange={(others) =>
                onChange({
                  ...formData,
                  induction: { ...formData.induction, others },
                })
              }
              defaultItem={() => ({ name: '', dose: '', drug_option_id: undefined, dosage_unit: '' })}
              addLabel={t('animalRecords.surgeries.addAgent')}
              renderItem={(item, _index, onItemChange) => (
                <div className="grid grid-cols-2 gap-2">
                  <DrugCombobox
                    value={{
                      drug_option_id: item.drug_option_id,
                      drug_name: item.name,
                      dosage_value: item.dose,
                      dosage_unit: item.dosage_unit || '',
                    }}
                    onChange={(sel) =>
                      onItemChange({
                        ...item,
                        name: sel.drug_name,
                        dose: sel.dosage_value,
                        drug_option_id: sel.drug_option_id,
                        dosage_unit: sel.dosage_unit,
                      })
                    }
                  />
                </div>
              )}
            />
          </div>
        </div>
      </CollapsibleSection>

      {/* 術前給藥 */}
      <CollapsibleSection title={t('animalRecords.surgeries.preSurgeryMedication')} defaultOpen={false}>
        <Repeater<MedicationItem>
          value={formData.pre_surgery.medications}
          onChange={(medications) =>
            onChange({
              ...formData,
              pre_surgery: { ...formData.pre_surgery, medications },
            })
          }
          defaultItem={() => ({ name: '', dose: '', drug_option_id: undefined, dosage_unit: '' })}
          addLabel={t('animalRecords.surgeries.addPreOpDrug')}
          renderItem={(item, _index, onItemChange) => (
            <div className="grid grid-cols-2 gap-2">
              <DrugCombobox
                value={{
                  drug_option_id: item.drug_option_id,
                  drug_name: item.name,
                  dosage_value: item.dose,
                  dosage_unit: item.dosage_unit || '',
                }}
                onChange={(sel) =>
                  onItemChange({
                    ...item,
                    name: sel.drug_name,
                    dose: sel.dosage_value,
                    drug_option_id: sel.drug_option_id,
                    dosage_unit: sel.dosage_unit,
                  })
                }
              />
            </div>
          )}
        />
      </CollapsibleSection>

      {/* 固定姿勢（可複選） */}
      <CollapsibleSection title={t('animalRecords.surgeries.positioning')} defaultOpen={false}>
        <div className="space-y-3">
          <Label>{t('animalRecords.surgeries.positioningMulti')}</Label>
          <div className="grid grid-cols-2 gap-3">
            {POSTURE_OPTIONS.map(({ value: posture, labelKey }) => (
              <Checkbox
                key={posture}
                label={t(labelKey)}
                checked={formData.positioning.includes(posture)}
                onCheckedChange={(checked) => {
                  const next = checked
                    ? [...formData.positioning, posture]
                    : formData.positioning.filter((p) => p !== posture)
                  onChange({ ...formData, positioning: next })
                }}
              />
            ))}
          </div>
        </div>
      </CollapsibleSection>

      {/* 麻醉維持 */}
      <CollapsibleSection title={t('animalRecords.surgeries.anesthesiaMaintenance')}>
        <div className="space-y-3">
          <DrugCheckInput
            label="O2"
            drug={formData.maintenance.o2}
            onChange={(drug) =>
              onChange({
                ...formData,
                maintenance: { ...formData.maintenance, o2: drug },
              })
            }
          />
          <DrugCheckInput
            label="N2O"
            drug={formData.maintenance.n2o}
            onChange={(drug) =>
              onChange({
                ...formData,
                maintenance: { ...formData.maintenance, n2o: drug },
              })
            }
          />
          <DrugCheckInput
            label="Isoflurane"
            drug={formData.maintenance.isoflurane}
            onChange={(drug) =>
              onChange({
                ...formData,
                maintenance: { ...formData.maintenance, isoflurane: drug },
              })
            }
          />
          <div className="pt-2">
            <Label className="text-sm text-muted-foreground">{t('animalRecords.surgeries.otherAgents')}</Label>
            <Repeater<MedicationItem>
              value={formData.maintenance.others}
              onChange={(others) =>
                onChange({
                  ...formData,
                  maintenance: { ...formData.maintenance, others },
                })
              }
              defaultItem={() => ({ name: '', dose: '', drug_option_id: undefined, dosage_unit: '' })}
              addLabel={t('animalRecords.surgeries.addAgent')}
              renderItem={(item, _index, onItemChange) => (
                <div className="grid grid-cols-2 gap-2">
                  <DrugCombobox
                    value={{
                      drug_option_id: item.drug_option_id,
                      drug_name: item.name,
                      dosage_value: item.dose,
                      dosage_unit: item.dosage_unit || '',
                    }}
                    onChange={(sel) =>
                      onItemChange({
                        ...item,
                        name: sel.drug_name,
                        dose: sel.dosage_value,
                        drug_option_id: sel.drug_option_id,
                        dosage_unit: sel.dosage_unit,
                      })
                    }
                  />
                </div>
              )}
            />
          </div>
        </div>
      </CollapsibleSection>

      {/* 監測與恢復 */}
      <CollapsibleSection title={t('animalRecords.surgeries.monitoringRecovery')}>
        <div className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="anesthesia_observation">{t('animalRecords.surgeries.anesthesiaObservation')}</Label>
            <Textarea
              id="anesthesia_observation"
              value={formData.anesthesia_observation}
              onChange={(e) =>
                onChange({ ...formData, anesthesia_observation: e.target.value })
              }
              placeholder={t('animalRecords.surgeries.anesthesiaObservationPlaceholder')}
            />
          </div>

          <div className="space-y-2">
            <Label>{t('animalRecords.surgeries.vitalSigns')}</Label>
            <Repeater<VitalSign>
              value={formData.vital_signs}
              onChange={(vital_signs) => onChange({ ...formData, vital_signs })}
              defaultItem={() => ({
                time: '',
                breathing_method: '',
                heart_rate: '',
                respiration_rate: '',
                temperature: '',
                spo2: '',
              })}
              addLabel={t('animalRecords.surgeries.addMeasurement')}
              maxItems={100}
              renderItem={(item, _index, onItemChange) => (
                <div className="grid grid-cols-6 gap-2 p-2 bg-muted rounded">
                  <Input
                    type="time"
                    placeholder={t('animalRecords.surgeries.vitalTime')}
                    value={item.time}
                    onChange={(e) => onItemChange({ ...item, time: e.target.value })}
                  />
                  <Input
                    placeholder={t('animalRecords.surgeries.breathingMethod')}
                    value={item.breathing_method}
                    onChange={(e) =>
                      onItemChange({ ...item, breathing_method: e.target.value })
                    }
                  />
                  <Input
                    type="number"
                    placeholder={t('animalRecords.surgeries.heartRatePlaceholder')}
                    value={item.heart_rate}
                    onChange={(e) => onItemChange({ ...item, heart_rate: e.target.value })}
                  />
                  <Input
                    type="number"
                    placeholder={t('animalRecords.surgeries.respirationPlaceholder')}
                    value={item.respiration_rate}
                    onChange={(e) =>
                      onItemChange({ ...item, respiration_rate: e.target.value })
                    }
                  />
                  <Input
                    type="number"
                    step="0.1"
                    placeholder={t('animalRecords.surgeries.temperaturePlaceholder')}
                    value={item.temperature}
                    onChange={(e) => onItemChange({ ...item, temperature: e.target.value })}
                  />
                  <Input
                    type="number"
                    placeholder="SPO2%"
                    value={item.spo2}
                    onChange={(e) => onItemChange({ ...item, spo2: e.target.value })}
                  />
                </div>
              )}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="reflex_recovery">{t('animalRecords.surgeries.reflexRecovery')}</Label>
            <Textarea
              id="reflex_recovery"
              value={formData.reflex_recovery}
              onChange={(e) => onChange({ ...formData, reflex_recovery: e.target.value })}
              placeholder={t('animalRecords.surgeries.reflexRecoveryPlaceholder')}
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="respiration_auto">{t('animalRecords.surgeries.spontaneousRespiration')}</Label>
            <Input
              id="respiration_auto"
              type="number"
              value={formData.respiration_rate_auto}
              onChange={(e) =>
                onChange({ ...formData, respiration_rate_auto: e.target.value })
              }
              className="w-32"
            />
          </div>

          <Checkbox
            label={t('animalRecords.surgeries.postOintment')}
            checked={formData.post_ointment}
            onCheckedChange={(checked) =>
              onChange({ ...formData, post_ointment: checked === true })
            }
          />

          <div className="space-y-2">
            <Label htmlFor="remark">{t('animalRecords.shared.remark')}</Label>
            <Textarea
              id="remark"
              value={formData.remark}
              onChange={(e) => onChange({ ...formData, remark: e.target.value })}
              placeholder={t('animalRecords.shared.otherRemarkPlaceholder')}
            />
          </div>
        </div>
      </CollapsibleSection>
    </>
  )
}
