// Section 元件共用型別定義

import type { TFunction } from 'i18next'
import type { FormData } from './constants'

export interface SectionProps {
    formData: FormData
    updateWorkingContent: (section: keyof FormData['working_content'], path: string, value: any) => void
    setFormData: React.Dispatch<React.SetStateAction<FormData>>
    t: TFunction
    isIACUCStaff?: boolean
    isNew?: boolean
}

export interface PersonnelSectionProps extends SectionProps {
    onAddPersonnel: () => void
}

export interface StaffMember {
    id: string
    display_name: string
    email: string
    phone?: string
    organization?: string
    entry_date?: string
    position?: string
    aup_roles?: string[]
    years_experience?: number
    trainings?: { code: string; certificate_no?: string; received_date?: string }[]
}
