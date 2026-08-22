/**
 * 檔案上傳型別
 */

export interface UploadResponse {
    id: string
    file_name: string
    file_path: string
    file_size: number
    mime_type: string
}

export interface Attachment {
    id: string
    entity_type: string
    entity_id: string
    file_name: string
    file_path: string
    file_size: number
    mime_type: string
    uploaded_by: string
    created_at: string
}
