import type { TFunction } from 'i18next'
import type { ProtocolFormData, ProtocolWorkingContent } from '@/types/protocol'
import type { FileInfo } from '@/components/ui/file-upload'
import { defaultFormData } from './constants'
import { deepMerge } from './utils'

type TestItem = ProtocolWorkingContent['items']['test_items'][number]
type ControlItem = ProtocolWorkingContent['items']['control_items'][number]
type HazardMaterial = ProtocolWorkingContent['design']['hazards']['materials'][number]
type ControlledSubstanceItem = ProtocolWorkingContent['design']['controlled_substances']['items'][number]
type PersonnelMember = ProtocolWorkingContent['personnel'][number]

interface ServerAttachment {
  id: string
  file_name: string
  file_path?: string
  file_size?: number
  file_type?: string
  mime_type?: string
  created_at?: string
}

interface ProtocolData {
  title: string
  start_date?: string | null
  end_date?: string | null
  working_content?: ProtocolWorkingContent
}

export function mergeProtocolData(
  protocol: ProtocolData,
  prev: ProtocolFormData,
  t: TFunction,
): ProtocolFormData {
  const serverContent = protocol.working_content || {}
  const mergedWorkingContent = deepMerge(defaultFormData.working_content, serverContent) as ProtocolWorkingContent

  applyBasicDefaults(mergedWorkingContent, t)
  normalizeItems(mergedWorkingContent)
  normalizeDesign(mergedWorkingContent, t)
  normalizePersonnel(mergedWorkingContent)
  normalizeAttachments(mergedWorkingContent)

  return {
    title: protocol.title || prev.title,
    start_date: protocol.start_date || prev.start_date || '',
    end_date: protocol.end_date || prev.end_date || '',
    working_content: mergedWorkingContent as ProtocolFormData['working_content'],
  }
}

function applyBasicDefaults(content: ProtocolWorkingContent, t: TFunction) {
  if (!content.basic) return
  if (!content.basic.facility?.title?.trim()) {
    content.basic.facility = {
      ...(content.basic.facility || {}),
      title: t('aup.defaults.facilityName'),
    }
  }
  if (!content.basic.housing_location?.trim()) {
    content.basic.housing_location = t('aup.defaults.housingLocation')
  }
}

function normalizeItems(content: ProtocolWorkingContent) {
  if (!content.items) return
  if (content.items.use_test_item === undefined) {
    content.items.use_test_item = null
  }
  if (content.items.test_items) {
    content.items.test_items = content.items.test_items.map((item: TestItem) => ({
      ...item,
      photos: item.photos || [],
    }))
  }
  if (content.items.control_items) {
    content.items.control_items = content.items.control_items.map((item: ControlItem) => ({
      ...item,
      photos: item.photos || [],
    }))
  }
}

function normalizeDesign(content: ProtocolWorkingContent, t: TFunction) {
  if (!content.design) return
  if (content.design.endpoints && !content.design.endpoints.humane_endpoint?.trim()) {
    content.design.endpoints.humane_endpoint = t('aup.defaults.humaneEndpoint')
  }
  if (content.design.carcass_disposal && !content.design.carcass_disposal.method?.trim()) {
    content.design.carcass_disposal.method = t('aup.defaults.carcassDisposal')
  }
  if (content.design.hazards?.materials) {
    content.design.hazards.materials = content.design.hazards.materials.map((item: HazardMaterial) => ({
      ...item,
      photos: item.photos || [],
    }))
  }
  if (content.design.controlled_substances?.items) {
    content.design.controlled_substances.items = content.design.controlled_substances.items.map((item: ControlledSubstanceItem) => ({
      ...item,
      photos: item.photos || [],
    }))
  }
}

function normalizePersonnel(content: ProtocolWorkingContent) {
  if (!content.personnel) return
  content.personnel = content.personnel.map((person: PersonnelMember) => ({
    ...person,
    id: person.id || undefined,
    roles: person.roles || [],
    roles_other_text: person.roles_other_text || '',
    trainings: person.trainings || [],
    training_certificates: person.training_certificates || [],
  }))
}

function normalizeAttachments(content: ProtocolWorkingContent) {
  if (!content.attachments) return
  content.attachments = (content.attachments as (FileInfo | ServerAttachment)[]).map((att) => {
    if (att.id && att.file_name) {
      return {
        id: att.id,
        file_name: att.file_name,
        file_path: ('file_path' in att ? att.file_path : '') || '',
        file_size: ('file_size' in att ? att.file_size : 0) || 0,
        file_type: ('file_type' in att ? att.file_type : undefined) || ('mime_type' in att ? att.mime_type : undefined) || 'application/pdf',
        created_at: ('created_at' in att ? att.created_at : undefined),
      } satisfies FileInfo
    }
    return att as FileInfo
  })
}
