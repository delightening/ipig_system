import { useState, useEffect, useCallback } from 'react'

export type SystemSettings = Record<string, string>

export interface SystemSettingsFormData {
  companyName: string
  defaultWarehouseId: string
  costMethod: string
  emailHost: string
  emailPort: string
  emailUser: string
  emailPassword: string
  emailFromEmail: string
  emailFromName: string
  sessionTimeout: string
  iacucNotifyEmails: string
}

const unwrap = (val: unknown): string => {
  if (val === null || val === undefined) return ''
  if (typeof val === 'string') return val
  if (typeof val === 'number' || typeof val === 'boolean') return String(val)
  return ''
}

const defaultForm: SystemSettingsFormData = {
  companyName: 'iPig System',
  defaultWarehouseId: '',
  costMethod: 'weighted_average',
  emailHost: '',
  emailPort: '587',
  emailUser: '',
  emailPassword: '',
  emailFromEmail: '',
  emailFromName: '',
  sessionTimeout: '360',
  iacucNotifyEmails: '',
}

export function useSettingsForm(sysSettings: SystemSettings | undefined) {
  const [form, setForm] = useState<SystemSettingsFormData>(defaultForm)
  const [passwordEdited, setPasswordEdited] = useState(false)
  const [dirty, setDirty] = useState(false)

  useEffect(() => {
    if (!sysSettings) return
    setForm({
      companyName: unwrap(sysSettings.company_name) || 'iPig System',
      defaultWarehouseId: unwrap(sysSettings.default_warehouse_id) || '',
      costMethod: unwrap(sysSettings.cost_method) || 'weighted_average',
      emailHost: unwrap(sysSettings.smtp_host) || '',
      emailPort: unwrap(sysSettings.smtp_port) || '587',
      emailUser: unwrap(sysSettings.smtp_username) || '',
      emailPassword: unwrap(sysSettings.smtp_password) || '',
      emailFromEmail: unwrap(sysSettings.smtp_from_email) || '',
      emailFromName: unwrap(sysSettings.smtp_from_name) || '',
      sessionTimeout: unwrap(sysSettings.session_timeout_minutes) || '360',
      iacucNotifyEmails: unwrap(sysSettings.iacuc_notify_emails) || '',
    })
    setPasswordEdited(false)
    setDirty(false)
  }, [sysSettings])

  const updateField = useCallback(<K extends keyof SystemSettingsFormData>(
    key: K,
    value: SystemSettingsFormData[K]
  ) => {
    setForm((prev) => ({ ...prev, [key]: value }))
    setDirty(true)
  }, [])

  const clearDirty = useCallback(() => {
    setDirty(false)
  }, [])

  const buildPayload = useCallback(
    (smtpMask: string): Record<string, string> => {
      const payload: Record<string, string> = {
        company_name: form.companyName,
        default_warehouse_id: form.defaultWarehouseId,
        cost_method: form.costMethod,
        smtp_host: form.emailHost,
        smtp_port: form.emailPort,
        smtp_username: form.emailUser,
        smtp_from_email: form.emailFromEmail,
        smtp_from_name: form.emailFromName,
        session_timeout_minutes: form.sessionTimeout,
        iacuc_notify_emails: form.iacucNotifyEmails,
      }
      if (passwordEdited && form.emailPassword !== smtpMask) {
        payload.smtp_password = form.emailPassword
      }
      return payload
    },
    [form, passwordEdited]
  )

  return {
    form,
    setForm,
    passwordEdited,
    setPasswordEdited,
    dirty,
    updateField,
    clearDirty,
    buildPayload,
  }
}
