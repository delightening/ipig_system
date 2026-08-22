// Section 元件共用型別定義

import type { TFunction } from 'i18next'

import type { ProtocolFormVersion } from '@/lib/constants/protocolVersionManifests'

import type { FormData } from './constants'

export interface SectionProps {
    formData: FormData
    updateWorkingContent: (section: keyof FormData['working_content'], path: string, value: unknown) => void
    setFormData: React.Dispatch<React.SetStateAction<FormData>>
    t: TFunction
    isIACUCStaff?: boolean
    isNew?: boolean
    /** 版本名冊：本計畫的表單版本鍵，各 Section 據此 gate 版本相依欄位（fieldVisibleForVersion） */
    formVersion: ProtocolFormVersion
    /** 變更升級：原為 C/D/E、現升級至 F 時，於精緻化欄提示原文併於研究設計節 */
    refinementUpgradeHint?: boolean
}

export interface PersonnelSectionProps extends SectionProps {
    onAddPersonnel: () => void
    onEditPersonnel: (index: number) => void
    /** 匯入計畫（外部客戶，imported_at 非 null）：空職稱顯示「未填」而非「研究人員」 */
    isExternal?: boolean
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
