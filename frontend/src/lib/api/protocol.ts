import api from './client'

import type { ProtocolActivity } from '@/types'

export const getProtocolActivities = async (id: string): Promise<ProtocolActivity[]> => {
  const response = await api.get<ProtocolActivity[]>(`/protocols/${id}/activities`)
  return response.data
}
