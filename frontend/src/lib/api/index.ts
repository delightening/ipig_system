export { default } from './client'
export { deleteResource, formatEarTag, confirmPassword } from './client'

export { bloodTestApi, bloodTestTemplateApi, bloodTestPanelApi, bloodTestPresetApi, bloodTestAnalysisApi } from './bloodTest'
export { getProtocolActivities } from './protocol'
export { notificationRoutingApi } from './notification'
export { transferApi } from './transfer'
export { animalFieldCorrectionApi } from './animalFieldCorrection'
export { signatureApi } from './signature'
export type { SignRecordRequest, SignRecordResponse, SignatureInfo, SignatureStatusResponse } from './signature'
export { treatmentDrugApi } from './treatmentDrug'
export { getPoReceiptStatus } from './document'
export type { PoReceiptItem, PoReceiptStatus } from './document'

// ============================================
// 型別 re-export（向後相容）
// ============================================
export type * from '@/types/auth'
export type * from '@/types/erp'
export type * from '@/types/animal'
export type * from '@/types/aup'
export type * from '@/types/report'
export type * from '@/types/audit'
export type * from '@/types/notification'
export type * from '@/types/amendment'
export type * from '@/types/upload'
export type { ProtocolWorkingContent } from '@/types/protocol'

// 常值 re-export
export {
  storageLocationTypeNames,
} from '@/types/erp'
export {
  animalStatusNames, allAnimalStatusNames, animalBreedNames, animalGenderNames, recordTypeNames,
  CORRECTABLE_FIELDS,
} from '@/types/animal'
export {
  protocolStatusNames,
} from '@/types/aup'
export {
  notificationTypeNames,
  eventTypeNames, channelNames,
} from '@/types/notification'
export {
  amendmentStatusNames, amendmentStatusColors, amendmentTypeNames,
  AMENDMENT_CHANGE_ITEM_OPTIONS,
} from '@/types/amendment'
export {
  transferStatusNames,
  transferTypeNames,
} from '@/types/animal'
