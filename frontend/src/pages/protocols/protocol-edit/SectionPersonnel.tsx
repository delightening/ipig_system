// Section Personnel 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Pencil } from 'lucide-react'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import type { ProtocolPerson } from '@/types/protocol'
import type { PersonnelSectionProps } from './types'

export function SectionPersonnel({ formData, updateWorkingContent: _updateWorkingContent, setFormData, t, isIACUCStaff: _isIACUCStaff, isExternal, onAddPersonnel, onEditPersonnel }: PersonnelSectionProps) {

  // §8 職稱空值預設：內部 staff →「研究人員」、外部客戶匯入計畫 →「未填」
  // （與後端 pdf_export.apply_personnel_position_defaults 一致，網頁 / PDF 同步）。
  const defaultPosition = isExternal
    ? t('aup.personnel.defaults.unfilled')
    : t('aup.personnel.defaults.researcher')

  return (
    <Card>
      <CardHeader>
        <CardTitle>{t('aup.section8')}</CardTitle>
        <CardDescription>{t('aup.personnel.subtitle')}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <h3 className="font-semibold">{t('aup.personnel.listHeader')}</h3>
            <Button
              type="button"
              variant="outline"
              onClick={onAddPersonnel}
            >
              + {t('aup.personnel.addPersonnel')}
            </Button>
          </div>
          <div className="border rounded-md overflow-hidden">
            <div className="overflow-x-auto">
              <table className="w-full border-collapse table-fixed">
                <colgroup>
                  <col style={{ width: '70px' }} />
                  <col style={{ width: '120px' }} />
                  <col style={{ width: '150px' }} />
                  <col style={{ width: '130px' }} />
                  <col style={{ width: '100px' }} />
                  <col />
                  <col style={{ width: '96px' }} />
                </colgroup>
                <thead>
                  <tr className="bg-muted">
                    <th className="border p-2 text-center text-sm font-semibold">{t('aup.personnel.table.num')}</th>
                    <th className="border p-2 text-center text-sm font-semibold">{t('aup.personnel.table.name')}</th>
                    <th className="border p-2 text-center text-sm font-semibold">{t('aup.personnel.table.position')}</th>
                    <th className="border p-2 text-center text-sm font-semibold">{t('aup.personnel.table.roles')}</th>
                    <th className="border p-2 text-center text-sm font-semibold">{t('aup.personnel.table.experience')}</th>
                    <th className="border p-2 text-center text-sm font-semibold">{t('aup.personnel.table.trainings')}</th>
                    <th className="border p-2 text-center text-sm font-semibold">{t('aup.personnel.table.actions')}</th>
                  </tr>
                </thead>
                <tbody>
                  {(formData.working_content.personnel || []).map((person: ProtocolPerson, index: number) => (
                    <tr key={index} className="hover:bg-muted">
                      <td className="border p-2">
                        <div className="px-2 py-1 text-center font-medium">
                          {index + 1}
                        </div>
                      </td>
                      <td className="border p-2">
                        <div className="px-2 py-1 text-center break-words">
                          {person.name || '-'}
                        </div>
                      </td>
                      <td className="border p-2">
                        <div className="px-2 py-1 break-words">
                          {person.position || defaultPosition}
                        </div>
                      </td>
                      <td className="border p-2"> {/* Work Content */}
                        <div className="space-y-1 overflow-hidden">
                          <div className="flex flex-wrap gap-1">
                            {(person.roles || []).map((role: string) => (
                              <Badge key={role} variant="outline" className="text-xs">
                                {role}
                              </Badge>
                            ))}
                            {(!person.roles || person.roles.length === 0) && (
                              <span className="text-muted-foreground text-sm">-</span>
                            )}
                          </div>
                          {(person.roles || []).includes('i') && person.roles_other_text && (
                            <div className="text-xs text-muted-foreground mt-1 break-words">
                              {t('aup.personnel.roles.otherLabel')}{person.roles_other_text}
                            </div>
                          )}
                        </div>
                      </td>
                      <td className="border p-2">
                        <div className="px-2 py-1 text-center">
                          {person.years_experience ? `${person.years_experience} ${t('aup.personnel.experienceUnit')}` : '-'}
                        </div>
                      </td>
                      <td className="border p-2">
                        <div className="space-y-3 overflow-hidden">
                          {(!person.trainings || person.trainings.length === 0) && (
                            <span className="text-muted-foreground text-sm">-</span>
                          )}
                          {/* Per-training group: 全名 + 證號逐行列 (F. 其他 顯示 other_text)
                              依 A→F 字母順序排列（避免使用者勾選順序影響顯示） */}
                          {[...(person.trainings || [])].sort().map((trainingCode: string) => {
                            if (trainingCode === 'F') {
                              if (!person.trainings_other_text) return null
                              return (
                                <div key="F" className="space-y-1">
                                  <div className="text-sm font-semibold break-words">
                                    {t('aup.personnel.trainings.F')}：
                                  </div>
                                  <div className="text-sm text-muted-foreground break-words pl-4">
                                    {person.trainings_other_text}
                                  </div>
                                </div>
                              )
                            }
                            const certificates = (person.training_certificates || []).filter(
                              (cert: { training_code?: string }) => cert.training_code === trainingCode,
                            )
                            return (
                              <div key={trainingCode} className="space-y-1">
                                <div className="text-sm font-semibold break-words">
                                  {t(`aup.personnel.trainings.${trainingCode}`)}：
                                </div>
                                {certificates.length > 0 ? (
                                  certificates.map((cert: { training_code: string; certificate_no: string }, certIndex: number) => (
                                    <div key={certIndex} className="text-sm text-muted-foreground break-words pl-4">
                                      {cert.certificate_no || '-'}
                                    </div>
                                  ))
                                ) : (
                                  <div className="text-sm text-muted-foreground pl-4">-</div>
                                )}
                              </div>
                            )
                          })}
                        </div>
                      </td>
                      <td className="border p-2">
                        <div className="flex items-center justify-center gap-1">
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8"
                          aria-label="編輯"
                          onClick={() => onEditPersonnel(index)}
                        >
                          <Pencil className="h-4 w-4" />
                        </Button>
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-destructive"
                          aria-label="刪除"
                          onClick={() => {
                            const newPersonnel = [...formData.working_content.personnel]
                            newPersonnel.splice(index, 1)
                            setFormData((prev) => ({
                              ...prev,
                              working_content: {
                                ...prev.working_content,
                                personnel: newPersonnel
                              }
                            }))
                          }}
                        >
                          X
                        </Button>
                        </div>
                      </td>
                    </tr>
                  ))}
                  {(!formData.working_content.personnel || formData.working_content.personnel.length === 0) && (
                    <tr>
                      <td colSpan={8} className="border p-4 text-center text-muted-foreground">
                        {t('aup.personnel.table.noPersonnel')}
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
          </div>
          <div className="mt-4 p-4 bg-muted rounded-md">
            <p className="text-sm font-semibold mb-2">{t('aup.personnel.roles.title')}</p>
            <p className="text-xs text-muted-foreground">
              {t('aup.personnel.roles.list')}
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
