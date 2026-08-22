// 獸醫巡場報告 Dialog 共用型別（R82-7 由 VetPatrolReportDialog.tsx 抽出）

export interface AnimalOption {
    id: string
    ear_tag: string
    pen_location?: string
}

export interface EntryRow {
    /** 後端 UUID（送過 server 後拿到）；新加列為 undefined */
    id?: string
    /** 前端臨時鍵；用於 React key 穩定 */
    tempKey: string
    category: string
    /** R39+++ 多動物支援：同條觀察可關聯多隻豬 */
    animal_ids: string[]
    observation: string
    suggestion: string
    follow_up: string
}

export interface PatrolReport {
    id: string
    patrol_date: string
    accompanying_personnel: string | null
    /** 'draft' / 'awaiting_follow_up' / 'completed' */
    status: 'draft' | 'awaiting_follow_up' | 'completed'
    created_by: string | null
    follow_up_user_id: string | null
    follow_up_submitted_at: string | null
    entries: EntryWithAnimal[]
    photos: PatrolPhoto[]
    entry_photos: EntryPhoto[]
}

export interface PatrolPhoto {
    id: string
    file_name: string
    file_path: string
    mime_type: string
    caption: string
    sort_order: number
}

export interface EntryPhoto {
    id: string
    entry_id: string
    file_name: string
    mime_type: string
    caption: string
    sort_order: number
}

export interface EntryWithAnimal {
    id: string
    category: string
    /** @deprecated 向後相容；前端優先用 animal_ids */
    animal_id: string | null
    ear_tag: string | null
    animal_ids: string[]
    ear_tags: string[]
    observation: string
    suggestion: string
    follow_up: string
    sort_order: number
}
