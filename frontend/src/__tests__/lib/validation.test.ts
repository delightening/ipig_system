import { describe, it, expect } from 'vitest'
import {
  requiredString,
  emailField,
  passwordField,
  optionalString,
  positiveNumber,
  uuidField,
  loginSchema,
  forgotPasswordSchema,
  changePasswordSchema,
  partnerSchema,
  productSchema,
  createUserSchema,
  amendmentSchema,
  getApiErrorMessage,
} from '@/lib/validation'
import { AxiosError, AxiosHeaders } from 'axios'

// ─── Field-level schemas ───

describe('requiredString', () => {
  it('passes for non-empty string', () => {
    expect(requiredString('Name').parse('hello')).toBe('hello')
  })

  it('fails for empty string', () => {
    const result = requiredString('Name').safeParse('')
    expect(result.success).toBe(false)
  })
})

describe('emailField', () => {
  it('accepts valid email', () => {
    expect(emailField.parse('user@example.com')).toBe('user@example.com')
  })

  it('rejects invalid email', () => {
    expect(emailField.safeParse('not-email').success).toBe(false)
  })
})

describe('passwordField', () => {
  it('accepts 10+ char password', () => {
    expect(passwordField.parse('Abcdef1234')).toBe('Abcdef1234')
  })

  it('rejects short password (< 10 chars)', () => {
    expect(passwordField.safeParse('12345').success).toBe(false)
    expect(passwordField.safeParse('123456789').success).toBe(false)
  })
})

describe('optionalString', () => {
  it('returns string as-is', () => {
    expect(optionalString.parse('hello')).toBe('hello')
  })

  it('converts empty string to undefined', () => {
    expect(optionalString.parse('')).toBeUndefined()
  })

  it('accepts undefined', () => {
    expect(optionalString.parse(undefined)).toBeUndefined()
  })
})

describe('positiveNumber', () => {
  it('accepts positive numbers', () => {
    expect(positiveNumber('qty').parse(5)).toBe(5)
  })

  it('rejects zero', () => {
    expect(positiveNumber('qty').safeParse(0).success).toBe(false)
  })

  it('rejects negative', () => {
    expect(positiveNumber('qty').safeParse(-1).success).toBe(false)
  })
})

describe('uuidField', () => {
  it('accepts valid UUID', () => {
    const uuid = '550e8400-e29b-41d4-a716-446655440000'
    expect(uuidField.parse(uuid)).toBe(uuid)
  })

  it('rejects invalid UUID', () => {
    expect(uuidField.safeParse('not-a-uuid').success).toBe(false)
  })
})

// ─── Module schemas ───

describe('loginSchema', () => {
  it('accepts valid login data', () => {
    const result = loginSchema.safeParse({ email: 'user@test.com', password: 'abc' })
    expect(result.success).toBe(true)
  })

  it('rejects missing email', () => {
    const result = loginSchema.safeParse({ password: 'abc' })
    expect(result.success).toBe(false)
  })
})

describe('forgotPasswordSchema', () => {
  it('accepts valid email', () => {
    expect(forgotPasswordSchema.safeParse({ email: 'a@b.com' }).success).toBe(true)
  })
})

describe('changePasswordSchema', () => {
  it('accepts matching passwords', () => {
    const result = changePasswordSchema.safeParse({
      current_password: 'OldPass123!',
      new_password: 'NewPass1234',
      confirm_password: 'NewPass1234',
    })
    expect(result.success).toBe(true)
  })

  it('rejects mismatched passwords', () => {
    const result = changePasswordSchema.safeParse({
      current_password: 'OldPass123!',
      new_password: 'NewPass1234',
      confirm_password: 'DifferentXx',
    })
    expect(result.success).toBe(false)
  })
})

describe('partnerSchema', () => {
  it('accepts valid partner data', () => {
    const result = partnerSchema.safeParse({
      partner_type: 'supplier',
      name: 'Test Corp',
    })
    expect(result.success).toBe(true)
  })

  it('rejects invalid partner_type', () => {
    const result = partnerSchema.safeParse({
      partner_type: 'invalid',
      name: 'Test',
    })
    expect(result.success).toBe(false)
  })
})

describe('productSchema', () => {
  it('accepts valid product', () => {
    const result = productSchema.safeParse({
      name: 'Test Product',
      sku: 'SKU-001',
      base_uom: 'EA',
    })
    expect(result.success).toBe(true)
  })

  it('rejects missing required fields', () => {
    const result = productSchema.safeParse({ name: 'Test' })
    expect(result.success).toBe(false)
  })
})

describe('createUserSchema', () => {
  it('accepts valid user data', () => {
    const result = createUserSchema.safeParse({
      email: 'user@test.com',
      password: 'Abcdef1234',
      display_name: 'Test User',
      role_ids: ['role-1'],
    })
    expect(result.success).toBe(true)
  })

  it('rejects empty role_ids', () => {
    const result = createUserSchema.safeParse({
      email: 'user@test.com',
      password: 'Abcdef1234',
      display_name: 'Test',
      role_ids: [],
    })
    expect(result.success).toBe(false)
  })
})

describe('amendmentSchema', () => {
  it('accepts valid amendment', () => {
    const result = amendmentSchema.safeParse({
      protocol_id: '550e8400-e29b-41d4-a716-446655440000',
      title: 'Test Amendment',
    })
    expect(result.success).toBe(true)
  })

  it('rejects title over 200 chars', () => {
    const result = amendmentSchema.safeParse({
      protocol_id: '550e8400-e29b-41d4-a716-446655440000',
      title: 'a'.repeat(201),
    })
    expect(result.success).toBe(false)
  })
})

// ─── getApiErrorMessage ───

describe('getApiErrorMessage', () => {
  it('returns fallback for unknown error', () => {
    expect(getApiErrorMessage(null)).toBe('操作失敗，請稍後再試')
  })

  it('returns custom fallback', () => {
    expect(getApiErrorMessage(null, 'Custom')).toBe('Custom')
  })

  it('returns Error.message for plain Error', () => {
    expect(getApiErrorMessage(new Error('boom'))).toBe('boom')
  })

  it('extracts message from AxiosError with response.data.error.message', () => {
    const error = new AxiosError('test', 'ERR', undefined, undefined, {
      data: { error: { message: 'Server says no' } },
      status: 400,
      statusText: 'Bad Request',
      headers: {},
      config: { headers: new AxiosHeaders() },
    })
    expect(getApiErrorMessage(error)).toBe('Server says no')
  })

  it('extracts message from AxiosError with response.data.message', () => {
    const error = new AxiosError('test', 'ERR', undefined, undefined, {
      data: { message: 'Simple message' },
      status: 400,
      statusText: 'Bad Request',
      headers: {},
      config: { headers: new AxiosHeaders() },
    })
    expect(getApiErrorMessage(error)).toBe('Simple message')
  })

  it('extracts string response data', () => {
    const error = new AxiosError('test', 'ERR', undefined, undefined, {
      data: 'Plain text error',
      status: 400,
      statusText: 'Bad Request',
      headers: {},
      config: { headers: new AxiosHeaders() },
    })
    expect(getApiErrorMessage(error)).toBe('Plain text error')
  })

  it('returns status-based message for 401', () => {
    const error = new AxiosError('test', 'ERR', undefined, undefined, {
      data: {},
      status: 401,
      statusText: 'Unauthorized',
      headers: {},
      config: { headers: new AxiosHeaders() },
    })
    expect(getApiErrorMessage(error)).toBe('登入已過期，請重新登入')
  })

  it('returns status-based message for 429', () => {
    const error = new AxiosError('test', 'ERR', undefined, undefined, {
      data: {},
      status: 429,
      statusText: 'Too Many Requests',
      headers: {},
      config: { headers: new AxiosHeaders() },
    })
    expect(getApiErrorMessage(error)).toBe('操作過於頻繁，請稍後再試')
  })

  it('handles network error (no response)', () => {
    const error = new AxiosError('Network Error', 'ERR_NETWORK')
    expect(getApiErrorMessage(error)).toBe('無法連線至伺服器，請確認網路狀態')
  })

  it('handles timeout error', () => {
    const error = new AxiosError('timeout', 'ECONNABORTED')
    expect(getApiErrorMessage(error)).toBe('請求逾時，請檢查網路連線後再試')
  })
})
