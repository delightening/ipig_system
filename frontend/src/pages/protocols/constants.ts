import type { ProtocolStatus } from '@/types/aup'

export type StatusColorVariant = 'default' | 'secondary' | 'success' | 'warning' | 'destructive' | 'outline'

export const statusColors: Record<ProtocolStatus, StatusColorVariant> = {
  DRAFT: 'secondary',
  SUBMITTED: 'default',
  PRE_REVIEW: 'default',
  PRE_REVIEW_REVISION_REQUIRED: 'destructive',
  VET_REVIEW: 'warning',
  VET_REVISION_REQUIRED: 'destructive',
  UNDER_REVIEW: 'warning',
  REVISION_REQUIRED: 'destructive',
  RESUBMITTED: 'default',
  APPROVED: 'success',
  APPROVED_WITH_CONDITIONS: 'success',
  DEFERRED: 'secondary',
  REJECTED: 'destructive',
  SUSPENDED: 'destructive',
  CLOSED: 'outline',
  DELETED: 'outline',
}

export const allowedTransitions: Record<ProtocolStatus, ProtocolStatus[]> = {
  DRAFT: ['SUBMITTED'],
  SUBMITTED: ['PRE_REVIEW', 'VET_REVIEW'],
  PRE_REVIEW: ['VET_REVIEW', 'PRE_REVIEW_REVISION_REQUIRED'],
  PRE_REVIEW_REVISION_REQUIRED: ['PRE_REVIEW'],
  VET_REVIEW: ['UNDER_REVIEW', 'VET_REVISION_REQUIRED'],
  VET_REVISION_REQUIRED: ['VET_REVIEW'],
  UNDER_REVIEW: ['REVISION_REQUIRED', 'APPROVED', 'APPROVED_WITH_CONDITIONS', 'REJECTED', 'DEFERRED'],
  REVISION_REQUIRED: ['RESUBMITTED'],
  RESUBMITTED: ['UNDER_REVIEW'],
  APPROVED: ['SUSPENDED', 'CLOSED'],
  APPROVED_WITH_CONDITIONS: ['SUSPENDED', 'CLOSED'],
  DEFERRED: ['UNDER_REVIEW', 'CLOSED'],
  REJECTED: ['CLOSED'],
  SUSPENDED: ['UNDER_REVIEW', 'CLOSED'],
  CLOSED: [],
  DELETED: [],
}

export const REVIEWABLE_STATUSES: ProtocolStatus[] = [
  'SUBMITTED', 'PRE_REVIEW', 'VET_REVIEW', 'UNDER_REVIEW', 'APPROVED', 'APPROVED_WITH_CONDITIONS',
]
