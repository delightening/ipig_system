import { useTranslation } from 'react-i18next'

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import type { ProtocolWorkingContent } from '@/types/protocol'

type PersonnelMember = ProtocolWorkingContent['personnel'][number]

interface PersonnelSectionProps {
  personnel: ProtocolWorkingContent['personnel']
}

export function PersonnelSection({ personnel }: PersonnelSectionProps) {
  const { t } = useTranslation()

  if (!personnel || personnel.length === 0) {
    return null
  }

  return (
    <section className="mb-8 border-t pt-6 section-8" data-section={t('protocols.content.sections.personnel')}>
      <h2 className="text-2xl font-bold mb-4 border-b pb-2">{t('protocols.content.sections.personnel')}</h2>

      <div className="rounded-lg border bg-card overflow-hidden @container">

        {/* Table view: container ≥ 600px */}
        <div className="hidden @[600px]:block overflow-x-auto">
          <Table className="w-full" style={{ minWidth: 450 }}>
            <TableHeader>
              <TableRow className="bg-muted/50 hover:bg-muted/50">
                <TableHead style={{ width: 40 }} className="text-center">#</TableHead>
                <TableHead style={{ width: 90 }}>{t('protocols.content.sections.piName')}</TableHead>
                <TableHead style={{ width: 90 }} className="hidden @[630px]:table-cell">{t('protocols.content.sections.position')}</TableHead>
                <TableHead style={{ width: 80 }} className="text-center hidden @[630px]:table-cell">{t('protocols.content.sections.yearsExperience')}</TableHead>
                <TableHead style={{ minWidth: 150 }}>{t('protocols.content.sections.workContent')}</TableHead>
                <TableHead style={{ minWidth: 180 }} className="hidden @[810px]:table-cell">{t('protocols.content.sections.training')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {personnel.map((person: PersonnelMember, index: number) => (
                <TableRow key={index}>
                  <TableCell style={{ width: 40 }} className="text-center font-medium">{index + 1}</TableCell>
                  <TableCell style={{ width: 90 }} className="whitespace-normal break-words">{person.name || '-'}</TableCell>
                  <TableCell style={{ width: 90 }} className="whitespace-normal break-words hidden @[630px]:table-cell">{person.position || '-'}</TableCell>
                  <TableCell style={{ width: 80 }} className="text-center hidden @[630px]:table-cell">
                    {person.years_experience ? `${person.years_experience} ${t('protocols.content.sections.years')}` : '-'}
                  </TableCell>
                  <TableCell style={{ minWidth: 150 }}>
                    <RolesDisplay person={person} />
                  </TableCell>
                  <TableCell style={{ minWidth: 180 }} className="hidden @[810px]:table-cell">
                    <TrainingsDisplay person={person} />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>

        {/* Card view: container < 600px */}
        <div className="@[600px]:hidden divide-y">
          {personnel.map((person: PersonnelMember, index: number) => (
            <div key={index} className="p-3 space-y-2">
              <div className="flex items-baseline justify-between gap-2">
                <div>
                  <span className="text-xs text-muted-foreground">#{index + 1}</span>
                  <span className="ml-2 font-semibold text-foreground">{person.name || '-'}</span>
                </div>
                {person.position && <span className="text-xs text-muted-foreground">{person.position}</span>}
              </div>
              {person.years_experience != null && (
                <div className="text-xs text-muted-foreground">
                  {t('protocols.content.sections.yearsExperience')}：{person.years_experience} {t('protocols.content.sections.years')}
                </div>
              )}
              <div>
                <div className="text-xs text-muted-foreground uppercase mb-0.5">{t('protocols.content.sections.workContent')}</div>
                <RolesDisplay person={person} />
              </div>
              {person.trainings && person.trainings.length > 0 && (
                <div>
                  <div className="text-xs text-muted-foreground uppercase mb-0.5">{t('protocols.content.sections.training')}</div>
                  <TrainingsDisplay person={person} />
                </div>
              )}
            </div>
          ))}
        </div>

      </div>
    </section>
  )
}

function RolesDisplay({ person }: { person: PersonnelMember }) {
  const { t } = useTranslation()

  if (!person.roles || person.roles.length === 0) return <span className="text-muted-foreground">-</span>

  return (
    <div className="space-y-0.5 text-sm">
      {person.roles.map((code: string) => {
        const label = t(`aup.personnel.roles.items.${code}`, code)
        if (code === 'i' && person.roles_other_text) {
          return <div key={code}>{code}.{label}（{person.roles_other_text}）</div>
        }
        return <div key={code}>{code}.{label}</div>
      })}
    </div>
  )
}

function TrainingsDisplay({ person }: { person: PersonnelMember }) {
  const { t } = useTranslation()

  if (!person.trainings || person.trainings.length === 0) return <span className="text-muted-foreground">-</span>

  return (
    <div className="space-y-1 text-sm">
      {person.trainings.map((code: string) => {
        const label = t(`aup.trainings.${code}`, code)

        if (code === 'F' && person.trainings_other_text) {
          return (
            <div key={code}>
              <span>{label}（{person.trainings_other_text}）</span>
            </div>
          )
        }

        const certs = (person.training_certificates || [])
          .filter((cert: { training_code?: string }) => cert.training_code === code)
          .filter((cert: { certificate_no?: string }) => cert.certificate_no)

        return (
          <div key={code}>
            <span>{label}</span>
            {certs.length > 0 && (
              <span className="text-muted-foreground ml-1">
                （{certs.map((cert: { certificate_no: string }) => cert.certificate_no).join('；')}）
              </span>
            )}
          </div>
        )
      })}
    </div>
  )
}
