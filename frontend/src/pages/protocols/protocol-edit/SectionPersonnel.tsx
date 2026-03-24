// Section Personnel 元件
// 自動從 ProtocolEditPage.tsx 提取

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import type { ProtocolPerson } from '@/types/protocol'
import type { PersonnelSectionProps } from './types'

export function SectionPersonnel({ formData, updateWorkingContent: _updateWorkingContent, setFormData, t, isIACUCStaff: _isIACUCStaff, onAddPersonnel }: PersonnelSectionProps) {

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
              <table className="w-full border-collapse">
                <thead>
                  <tr className="bg-slate-100">
                    <th className="border p-2 text-center text-sm font-semibold w-16">{t('aup.personnel.table.num')}</th>
                    <th className="border p-2 text-center text-sm font-semibold w-24">{t('aup.personnel.table.name')}</th>
                    <th className="border p-2 text-center text-sm font-semibold w-24">{t('aup.personnel.table.position')}</th>
                    <th className="border p-2 text-center text-sm font-semibold w-32">{t('aup.personnel.table.roles')}</th>
                    <th className="border p-2 text-center text-sm font-semibold w-24">{t('aup.personnel.table.experience')}</th>
                    <th className="border p-2 text-center text-sm font-semibold">{t('aup.personnel.table.trainings')}</th>
                    <th className="border p-2 text-center text-sm font-semibold w-16">{t('aup.personnel.table.actions')}</th>
                  </tr>
                </thead>
                <tbody>
                  {(formData.working_content.personnel || []).map((person: ProtocolPerson, index: number) => (
                    <tr key={index} className="hover:bg-slate-50">
                      <td className="border p-2 w-8">
                        <div className="px-2 py-1 text-center font-medium">
                          {index + 1}
                        </div>
                      </td>
                      <td className="border p-2 w-24">
                        <div className="px-2 py-1 text-center truncate">
                          {person.name || '-'}
                        </div>
                      </td>
                      <td className="border p-2 w-24">
                        <div className="px-2 py-1 truncate">
                          {t('aup.personnel.defaults.researcher')}
                        </div>
                      </td>
                      <td className="border p-2 w-32"> {/* Work Content */}
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
                            <div className="text-xs text-muted-foreground mt-1 truncate">
                              {t('aup.personnel.roles.otherLabel')}{person.roles_other_text}
                            </div>
                          )}
                        </div>
                      </td>
                      <td className="border p-2 w-24">
                        <div className="px-2 py-1 text-center">
                          {person.years_experience ? `${person.years_experience} ${t('aup.personnel.experienceUnit')}` : '-'}
                        </div>
                      </td>
                      <td className="border p-2">
                        <div className="space-y-2 overflow-hidden">
                          <div className="flex flex-wrap gap-1 mb-2">
                            {(person.trainings || []).map((trainingCode: string) => (
                              <Badge key={trainingCode} variant="outline" className="text-xs">
                                {trainingCode}
                              </Badge>
                            ))}
                            {(!person.trainings || person.trainings.length === 0) && (
                              <span className="text-muted-foreground text-sm">-</span>
                            )}
                          </div>
                          {/* Show explanation for F. Other */}
                          {(person.trainings || []).includes('F') && person.trainings_other_text && (
                            <div className="space-y-1 pl-4 border-l-2 border-slate-200">
                              <div className="text-xs font-semibold truncate">F:</div>
                              <div className="text-xs text-muted-foreground truncate">
                                {person.trainings_other_text}
                              </div>
                            </div>
                          )}
                          {/* Show certificate number list for each selected training */}
                          {(person.trainings || []).filter((t: string) => t !== 'F').map((trainingCode: string) => {
                            const certificates = (person.training_certificates || []).filter((cert: { training_code?: string }) => cert.training_code === trainingCode)
                            if (certificates.length === 0) return null
                            return (
                              <div key={trainingCode} className="space-y-1 pl-4 border-l-2 border-slate-200">
                                <div className="text-xs font-semibold whitespace-nowrap truncate">{trainingCode}:</div>
                                {certificates.map((cert: { training_code: string; certificate_no: string }, certIndex: number) => (
                                  <div key={certIndex} className="text-xs text-muted-foreground whitespace-nowrap truncate">
                                    {cert.certificate_no || '-'}
                                  </div>
                                ))}
                              </div>
                            )
                          })}
                        </div>
                      </td>
                      <td className="border p-2 text-center w-16">
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 text-red-500"
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
          <div className="mt-4 p-4 bg-slate-50 rounded-md">
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
