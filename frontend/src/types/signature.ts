export interface SignRecordRequest {
  password?: string
  signature_type?: string
  handwriting_svg?: string
  stroke_data?: object[]
}

export interface SignRecordResponse {
  signature_id: string
  signed_at: string
  is_locked: boolean
}

export interface SignatureInfo {
  id: string
  signature_type: string
  signer_name: string | null
  signed_at: string
  signature_method: string | null
  handwriting_svg: string | null
}

export interface SignatureStatusResponse {
  is_signed: boolean
  is_locked: boolean
  signatures: SignatureInfo[]
}
