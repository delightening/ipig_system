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

      <div className="border rounded-md overflow-hidden">
        <Table>
          <TableHeader>
            <TableRow className="bg-slate-100">
              <TableHead className="text-center w-12">#</TableHead>
              <TableHead className="w-24">{t('protocols.content.sections.piName')}</TableHead>
              <TableHead className="w-24">{t('protocols.content.sections.position')}</TableHead>
              <TableHead className="text-center w-16">{t('protocols.content.sections.yearsExperience')}</TableHead>
              <TableHead>{t('protocols.content.sections.workContent')}</TableHead>
              <TableHead>{t('protocols.content.sections.training')}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {personnel.map((person: PersonnelMember, index: number) => (
              <TableRow key={index}>
                <TableCell className="text-center font-medium">{index + 1}</TableCell>
                <TableCell>{person.name || '-'}</TableCell>
                <TableCell>{person.position || '-'}</TableCell>
                <TableCell className="text-center">
                  {person.years_experience ? `${person.years_experience} ${t('protocols.content.sections.years')}` : '-'}
                </TableCell>
                <TableCell>
                  <RolesDisplay person={person} />
                </TableCell>
                <TableCell>
                  <TrainingsDisplay person={person} />
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
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
