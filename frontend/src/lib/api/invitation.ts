import api from './client'
import type {
    CreateInvitationRequest,
    CreateInvitationResponse,
    InvitationListResponse,
    InvitationVerifyResponse,
    AcceptInvitationRequest,
    AcceptInvitationResponse,
} from '@/types/invitation'

export const invitationApi = {
    create: (data: CreateInvitationRequest) =>
        api.post<CreateInvitationResponse>('/invitations', data),

    list: (params: { status?: string; page?: number; per_page?: number }) =>
        api.get<InvitationListResponse>('/invitations', { params }),

    revoke: (id: string) =>
        api.delete(`/invitations/${id}`),

    resend: (id: string) =>
        api.post<CreateInvitationResponse>(`/invitations/${id}/resend`),

    verify: (token: string) =>
        api.get<InvitationVerifyResponse>(`/invitations/verify/${token}`),

    accept: (data: AcceptInvitationRequest) =>
        api.post<AcceptInvitationResponse>('/invitations/accept', data),
}
