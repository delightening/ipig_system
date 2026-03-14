import api from './client'

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

export const signatureApi = {
  signSacrifice: (sacrificeId: string, data: SignRecordRequest) =>
    api.post<SignRecordResponse>(`/signatures/sacrifice/${sacrificeId}`, data),
  getSacrificeStatus: (sacrificeId: string) =>
    api.get<SignatureStatusResponse>(`/signatures/sacrifice/${sacrificeId}`),

  signObservation: (observationId: number, data: SignRecordRequest) =>
    api.post<SignRecordResponse>(`/signatures/observation/${observationId}`, data),

  signEuthanasia: (orderId: string, data: SignRecordRequest) =>
    api.post<SignRecordResponse>(`/signatures/euthanasia/${orderId}`, data),
  getEuthanasiaStatus: (orderId: string) =>
    api.get<SignatureStatusResponse>(`/signatures/euthanasia/${orderId}`),

  signTransfer: (transferId: string, data: SignRecordRequest) =>
    api.post<SignRecordResponse>(`/signatures/transfer/${transferId}`, data),
  getTransferStatus: (transferId: string) =>
    api.get<SignatureStatusResponse>(`/signatures/transfer/${transferId}`),

  signProtocol: (protocolId: string, data: SignRecordRequest) =>
    api.post<SignRecordResponse>(`/signatures/protocol/${protocolId}`, data),
  getProtocolStatus: (protocolId: string) =>
    api.get<SignatureStatusResponse>(`/signatures/protocol/${protocolId}`),
}
