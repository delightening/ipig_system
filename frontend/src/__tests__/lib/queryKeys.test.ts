import { describe, it, expect } from 'vitest'
import { queryKeys } from '@/lib/queryKeys'

describe('queryKeys', () => {
  describe('animals', () => {
    it('has static all key', () => {
      expect(queryKeys.animals.all).toEqual(['animals'])
    })

    it('list creates unique key per filter', () => {
      const key1 = queryKeys.animals.list({ status: 'active' })
      const key2 = queryKeys.animals.list({ status: 'inactive' })
      expect(key1).not.toEqual(key2)
      expect(key1[0]).toBe('animals')
    })

    it('detail creates id-based key', () => {
      const key = queryKeys.animals.detail('abc-123')
      expect(key).toEqual(['animal', 'abc-123'])
    })

    it('observations includes optional after param', () => {
      const key = queryKeys.animals.observations('id-1', '2024-01-01')
      expect(key).toEqual(['animal-observations', 'id-1', '2024-01-01'])
    })

    it('observations works without after param', () => {
      const key = queryKeys.animals.observations('id-1')
      expect(key).toEqual(['animal-observations', 'id-1', undefined])
    })
  })

  describe('protocols', () => {
    it('has static all key', () => {
      expect(queryKeys.protocols.all).toEqual(['protocols'])
    })

    it('detail creates id-based key', () => {
      expect(queryKeys.protocols.detail('p-1')).toEqual(['protocol', 'p-1'])
    })

    it('versions creates id-based key', () => {
      expect(queryKeys.protocols.versions('p-1')).toEqual(['protocol-versions', 'p-1'])
    })

    it('has static approvedList', () => {
      expect(queryKeys.protocols.approvedList).toEqual(['approved-protocols'])
    })
  })

  describe('users', () => {
    it('has static all key', () => {
      expect(queryKeys.users.all).toEqual(['users'])
    })

    it('preferences creates key-based key', () => {
      expect(queryKeys.users.preferences('theme')).toEqual(['user-preferences', 'theme'])
    })
  })

  describe('documents', () => {
    it('list uses filters object', () => {
      const key = queryKeys.documents.list({ type: 'invoice' })
      expect(key[0]).toBe('documents')
      expect(key[1]).toEqual({ type: 'invoice' })
    })

    it('has static recent key', () => {
      expect(queryKeys.documents.recent).toEqual(['recent-documents'])
    })
  })

  describe('notifications', () => {
    it('has static unreadCount', () => {
      expect(queryKeys.notifications.unreadCount).toEqual(['notifications-unread-count'])
    })
  })

  describe('hr', () => {
    it('allOvertime uses filters', () => {
      const key = queryKeys.hr.allOvertime({ month: '2024-03' })
      expect(key[0]).toBe('hr-all-overtime')
    })

    it('has static myLeaves', () => {
      expect(queryKeys.hr.myLeaves).toEqual(['hr-my-leaves'])
    })
  })

  describe('admin', () => {
    it('auditLogs uses filters', () => {
      const key = queryKeys.admin.auditLogs({ page: 1 })
      expect(key).toEqual(['audit-logs', { page: 1 }])
    })
  })

  describe('key prefix uniqueness', () => {
    it('all static keys start with unique prefixes', () => {
      const allKeys = [
        queryKeys.animals.all,
        queryKeys.protocols.all,
        queryKeys.users.all,
        queryKeys.documents.all,
        queryKeys.notifications.all,
        queryKeys.products.all,
        queryKeys.warehouses.all,
      ]
      const prefixes = allKeys.map(k => k[0])
      const uniquePrefixes = new Set(prefixes)
      expect(uniquePrefixes.size).toBe(allKeys.length)
    })
  })
})
