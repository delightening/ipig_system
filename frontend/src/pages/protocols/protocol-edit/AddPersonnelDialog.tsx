import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { toast } from '@/components/ui/use-toast'
import { TrainingCertificates } from './TrainingCertificates'

interface StaffMember {
  id: string
  display_name: string
  entry_date?: string
  years_experience?: number
  trainings?: { code: string; certificate_no?: string }[]
}

interface NewPersonnelData {
  name: string
  position: string
  roles: string[]
  roles_other_text: string
  years_experience: number
  trainings: string[]
  trainings_other_text: string
  training_certificates: Array<{ training_code: string; certificate_no: string }>
}

interface AddPersonnelDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  staffMembers: StaffMember[]
  isIACUCStaff: boolean
  onAdd: (personnel: NewPersonnelData) => void
}

const INITIAL_PERSONNEL: NewPersonnelData = {
  name: '',
  position: '',
  roles: [],
  roles_other_text: '',
  years_experience: 0,
  trainings: [],
  trainings_other_text: '',
  training_certificates: [],
}

export function AddPersonnelDialog({
  open,
  onOpenChange,
  staffMembers,
  isIACUCStaff,
  onAdd,
}: AddPersonnelDialogProps) {
  const { t } = useTranslation()
  const [newPersonnel, setNewPersonnel] = useState<NewPersonnelData>({ ...INITIAL_PERSONNEL })

  const handleOpenChange = (isOpen: boolean) => {
    if (isOpen) setNewPersonnel({ ...INITIAL_PERSONNEL })
    onOpenChange(isOpen)
  }

  const handleSubmit = () => {
    const error = validateNewPersonnel(newPersonnel, t)
    if (error) {
      toast({ title: t('common.error'), description: error, variant: 'destructive' })
      return
    }
    onAdd(newPersonnel)
    onOpenChange(false)
    toast({ title: t('common.success'), description: t('aup.personnel.addDialog.messages.added') })
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="max-w-4xl">
        <DialogHeader>
          <DialogTitle>{t('aup.personnel.addDialog.title')}</DialogTitle>
          <DialogDescription>{t('aup.personnel.addDialog.description')}</DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <NamePositionRow
            newPersonnel={newPersonnel}
            setNewPersonnel={setNewPersonnel}
            staffMembers={staffMembers}
            isIACUCStaff={isIACUCStaff}
            t={t}
          />
          <RolesSection newPersonnel={newPersonnel} setNewPersonnel={setNewPersonnel} t={t} />
          <div className="space-y-2">
            <Label>{t('aup.personnel.addDialog.labels.experience')} *</Label>
            <Input
              type="number"
              min="0"
              step="1"
              value={newPersonnel.years_experience || ''}
              onChange={(e) => setNewPersonnel((prev) => ({ ...prev, years_experience: parseInt(e.target.value) || 0 }))}
              placeholder={t('aup.personnel.addDialog.placeholders.experience')}
            />
          </div>
          <TrainingsSection newPersonnel={newPersonnel} setNewPersonnel={setNewPersonnel} t={t} />
        </div>
        <DialogFooter>
          <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
            {t('aup.personnel.addDialog.buttons.cancel')}
          </Button>
          <Button type="button" onClick={handleSubmit}>
            {t('aup.personnel.addDialog.buttons.add')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

function validateNewPersonnel(p: NewPersonnelData, t: (key: string) => string): string | null {
  if (!p.name.trim()) return t('aup.personnel.addDialog.validation.nameRequired')
  if (p.roles.length === 0) return t('aup.personnel.addDialog.validation.rolesRequired')
  if (p.roles.includes('i') && !p.roles_other_text.trim()) return t('aup.personnel.addDialog.validation.rolesOtherRequired')
  if (p.years_experience <= 0) return t('aup.personnel.addDialog.validation.experienceRequired')
  if (p.trainings.length === 0) return t('aup.personnel.addDialog.validation.trainingsRequired')
  if (p.trainings.includes('F') && !p.trainings_other_text.trim()) return t('aup.personnel.addDialog.validation.trainingsOtherRequired')
  return null
}

interface SectionProps {
  newPersonnel: NewPersonnelData
  setNewPersonnel: React.Dispatch<React.SetStateAction<NewPersonnelData>>
  t: (key: string, opts?: Record<string, unknown>) => string
}

function NamePositionRow({ newPersonnel, setNewPersonnel, staffMembers, isIACUCStaff, t }: SectionProps & { staffMembers: StaffMember[]; isIACUCStaff: boolean }) {
  return (
    <div className="grid grid-cols-2 gap-4">
      <div className="space-y-2">
        <Label>{t('aup.personnel.addDialog.labels.name')} *</Label>
        {isIACUCStaff ? (
          <div className="flex gap-2">
            <Select
              onValueChange={(value) => {
                const staff = staffMembers.find((s) => s.id === value)
                if (!staff) return
                let years = staff.years_experience || 0
                if (staff.entry_date) {
                  years = new Date().getFullYear() - new Date(staff.entry_date).getFullYear()
                }
                setNewPersonnel((prev) => ({
                  ...prev,
                  name: staff.display_name,
                  position: t('aup.personnel.defaults.researcher'),
                  years_experience: years,
                  roles: ['b', 'c', 'd', 'f', 'g', 'h'],
                  trainings: (staff.trainings || []).map((tr) => tr.code),
                  training_certificates: (staff.trainings || []).map((tr) => ({
                    training_code: tr.code,
                    certificate_no: tr.certificate_no || '',
                  })),
                }))
              }}
            >
              <SelectTrigger className="w-48">
                <SelectValue placeholder={t('aup.personnel.addDialog.placeholders.name')} />
              </SelectTrigger>
              <SelectContent>
                {staffMembers.map((staff) => (
                  <SelectItem key={staff.id} value={staff.id}>{staff.display_name}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Input
              value={newPersonnel.name}
              onChange={(e) => setNewPersonnel((prev) => ({ ...prev, name: e.target.value }))}
              placeholder={t('aup.personnel.placeholders.name')}
              className="flex-1"
            />
          </div>
        ) : (
          <Input
            value={newPersonnel.name}
            onChange={(e) => setNewPersonnel((prev) => ({ ...prev, name: e.target.value }))}
            placeholder={t('aup.personnel.addDialog.placeholders.name')}
          />
        )}
      </div>
      <div className="space-y-2">
        <Label>{t('aup.personnel.addDialog.labels.position')}</Label>
        <Input
          value={newPersonnel.position}
          onChange={(e) => setNewPersonnel((prev) => ({ ...prev, position: e.target.value }))}
          placeholder={t('aup.personnel.addDialog.placeholders.position')}
        />
      </div>
    </div>
  )
}

function RolesSection({ newPersonnel, setNewPersonnel, t }: SectionProps) {
  return (
    <div className="space-y-2">
      <Label>{t('aup.personnel.addDialog.labels.roles')} *</Label>
      <div className="flex flex-wrap gap-2">
        {['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'].map(role => (
          <div key={role} className="flex items-center space-x-1">
            <Checkbox
              id={`new_role_${role}`}
              checked={newPersonnel.roles.includes(role)}
              onCheckedChange={(checked) => {
                if (checked) {
                  setNewPersonnel((prev) => ({ ...prev, roles: [...prev.roles, role] }))
                } else {
                  setNewPersonnel((prev) => ({
                    ...prev,
                    roles: prev.roles.filter(r => r !== role),
                    roles_other_text: role === 'i' ? '' : prev.roles_other_text,
                  }))
                }
              }}
            />
            <Label htmlFor={`new_role_${role}`} className="text-sm font-normal cursor-pointer">{role}</Label>
          </div>
        ))}
      </div>
      <div className="mt-2 p-3 bg-muted rounded-md">
        <p className="text-xs text-muted-foreground">
          {t('aup.personnel.roles.title')}<br />{t('aup.personnel.roles.list')}
        </p>
      </div>
      {newPersonnel.roles.includes('i') && (
        <Input
          value={newPersonnel.roles_other_text}
          onChange={(e) => setNewPersonnel((prev) => ({ ...prev, roles_other_text: e.target.value }))}
          placeholder={t('aup.personnel.addDialog.placeholders.rolesOther')}
          className="mt-2"
        />
      )}
    </div>
  )
}

function TrainingsSection({ newPersonnel, setNewPersonnel, t }: SectionProps) {
  const TRAINING_OPTIONS = [
    { value: 'A', label: t('aup.personnel.trainings.A') },
    { value: 'B', label: t('aup.personnel.trainings.B') },
    { value: 'C', label: t('aup.personnel.trainings.C') },
    { value: 'D', label: t('aup.personnel.trainings.D') },
    { value: 'E', label: t('aup.personnel.trainings.E') },
    { value: 'F', label: t('aup.personnel.trainings.F') },
  ]

  return (
    <div className="space-y-2">
      <Label>{t('aup.personnel.addDialog.labels.trainings')} *</Label>
      <div className="flex flex-wrap gap-2 mb-2">
        {TRAINING_OPTIONS.map(training => (
          <div key={training.value} className="flex items-center space-x-1">
            <Checkbox
              id={`new_training_${training.value}`}
              checked={newPersonnel.trainings.includes(training.value)}
              onCheckedChange={(checked) => {
                if (checked) {
                  setNewPersonnel((prev) => ({ ...prev, trainings: [...prev.trainings, training.value] }))
                } else {
                  setNewPersonnel((prev) => ({
                    ...prev,
                    trainings: prev.trainings.filter(tr => tr !== training.value),
                    trainings_other_text: training.value === 'F' ? '' : prev.trainings_other_text,
                    training_certificates: prev.training_certificates.filter(
                      cert => cert.training_code !== training.value,
                    ),
                  }))
                }
              }}
            />
            <Label htmlFor={`new_training_${training.value}`} className="text-xs font-normal cursor-pointer">{training.label}</Label>
          </div>
        ))}
      </div>
      {newPersonnel.trainings.includes('F') && (
        <Input
          value={newPersonnel.trainings_other_text}
          onChange={(e) => setNewPersonnel((prev) => ({ ...prev, trainings_other_text: e.target.value }))}
          placeholder={t('aup.personnel.addDialog.placeholders.trainingsOther')}
          className="mt-2"
        />
      )}
      {newPersonnel.trainings.filter(tr => tr !== 'F').map((trainingCode: string) => (
        <TrainingCertificates
          key={trainingCode}
          trainingCode={trainingCode}
          certificates={newPersonnel.training_certificates.filter(c => c.training_code === trainingCode)}
          allCertificates={newPersonnel.training_certificates}
          onCertificatesChange={(certs) => setNewPersonnel((prev) => ({ ...prev, training_certificates: certs }))}
          t={t}
        />
      ))}
    </div>
  )
}

