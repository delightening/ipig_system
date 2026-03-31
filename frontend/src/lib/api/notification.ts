import api, { deleteResource } from './client'

import type {
  NotificationRouting, CreateNotificationRoutingRequest,
  UpdateNotificationRoutingRequest, EventTypeCategory, RoleInfo,
  ExpiryNotificationConfig, UpdateExpiryNotificationConfigRequest,
} from '@/types/notification'

export const notificationRoutingApi = {
  list: () =>
    api.get<NotificationRouting[]>('/admin/notification-routing'),
  create: (data: CreateNotificationRoutingRequest) =>
    api.post<NotificationRouting>('/admin/notification-routing', data),
  update: (id: string, data: UpdateNotificationRoutingRequest) =>
    api.put<NotificationRouting>(`/admin/notification-routing/${id}`, data),
  delete: (id: string) =>
    deleteResource(`/admin/notification-routing/${id}`),
  getEventTypes: () =>
    api.get<EventTypeCategory[]>('/admin/notification-routing/event-types'),
  getRoles: () =>
    api.get<RoleInfo[]>('/admin/notification-routing/roles'),
}

export const expiryConfigApi = {
  get: () =>
    api.get<ExpiryNotificationConfig>('/admin/expiry-config'),
  update: (data: UpdateExpiryNotificationConfigRequest) =>
    api.put<ExpiryNotificationConfig>('/admin/expiry-config', data),
}
