import { describe, it, expect } from 'vitest'
import {
  taxIdSchema,
  phoneSchema,
  emailOptionalSchema,
  nonEmptyString,
  partnerFormSchema,
  warehouseFormSchema,
  animalFormSchema,
  getFirstZodError,
} from '@/lib/validations'
import { z } from 'zod'

describe('taxIdSchema', () => {
  it('accepts empty string', () => {
    expect(taxIdSchema.parse('')).toBe('')
  })

  it('accepts valid 8-digit tax ID', () => {
    expect(taxIdSchema.parse('12345678')).toBe('12345678')
  })

  it('rejects non-8-digit string', () => {
    expect(taxIdSchema.safeParse('1234567').success).toBe(false)
    expect(taxIdSchema.safeParse('123456789').success).toBe(false)
  })

  it('rejects non-numeric', () => {
    expect(taxIdSchema.safeParse('abcdefgh').success).toBe(false)
  })
})

describe('phoneSchema', () => {
  it('accepts empty string', () => {
    expect(phoneSchema.parse('')).toBe('')
  })

  it('accepts 9-digit phone', () => {
    expect(phoneSchema.parse('912345678')).toBe('912345678')
  })

  it('accepts 10-digit phone', () => {
    expect(phoneSchema.parse('0912345678')).toBe('0912345678')
  })

  it('rejects short phone', () => {
    expect(phoneSchema.safeParse('12345678').success).toBe(false)
  })
})

describe('emailOptionalSchema', () => {
  it('accepts empty string', () => {
    expect(emailOptionalSchema.parse('')).toBe('')
  })

  it('accepts valid email', () => {
    expect(emailOptionalSchema.parse('user@test.com')).toBe('user@test.com')
  })

  it('rejects invalid email', () => {
    expect(emailOptionalSchema.safeParse('not-email').success).toBe(false)
  })
})

describe('nonEmptyString', () => {
  it('accepts non-empty string', () => {
    expect(nonEmptyString().parse('hello')).toBe('hello')
  })

  it('rejects empty string', () => {
    const result = nonEmptyString().safeParse('')
    expect(result.success).toBe(false)
  })
})

describe('partnerFormSchema', () => {
  it('accepts valid partner', () => {
    const result = partnerFormSchema.safeParse({
      code: 'P001',
      name: 'Test Partner',
      type: 'vendor',
    })
    expect(result.success).toBe(true)
  })

  it('rejects missing code', () => {
    const result = partnerFormSchema.safeParse({
      code: '',
      name: 'Test',
      type: 'customer',
    })
    expect(result.success).toBe(false)
  })

  it('rejects invalid type', () => {
    const result = partnerFormSchema.safeParse({
      code: 'P001',
      name: 'Test',
      type: 'invalid',
    })
    expect(result.success).toBe(false)
  })
})

describe('warehouseFormSchema', () => {
  it('accepts valid warehouse', () => {
    const result = warehouseFormSchema.safeParse({ name: 'Main Warehouse' })
    expect(result.success).toBe(true)
  })

  it('rejects empty name', () => {
    const result = warehouseFormSchema.safeParse({ name: '' })
    expect(result.success).toBe(false)
  })
})

describe('animalFormSchema', () => {
  const validAnimal = {
    ear_tag: 'E001',
    breed: 'minipig' as const,
    gender: 'male' as const,
    pen_location: 'A-1-1',
    entry_date: '2024-01-15',
    birth_date: '2023-06-01',
    pre_experiment_code: 'PRE001',
  }

  it('accepts valid animal data', () => {
    const result = animalFormSchema.safeParse(validAnimal)
    expect(result.success).toBe(true)
  })

  it('rejects invalid breed', () => {
    const result = animalFormSchema.safeParse({ ...validAnimal, breed: 'unknown' })
    expect(result.success).toBe(false)
  })

  it('rejects invalid date format', () => {
    const result = animalFormSchema.safeParse({ ...validAnimal, entry_date: '01/15/2024' })
    expect(result.success).toBe(false)
  })

  it('rejects negative entry_weight', () => {
    const result = animalFormSchema.safeParse({ ...validAnimal, entry_weight: '-5' })
    expect(result.success).toBe(false)
  })

  it('accepts valid entry_weight', () => {
    const result = animalFormSchema.safeParse({ ...validAnimal, entry_weight: '25.5' })
    expect(result.success).toBe(true)
  })
})

describe('getFirstZodError', () => {
  it('returns first error message', () => {
    const schema = z.object({ name: z.string().min(1, 'Name required') })
    const result = schema.safeParse({ name: '' })
    if (!result.success) {
      expect(getFirstZodError(result.error)).toBe('Name required')
    }
  })

  it('returns fallback for empty issues', () => {
    const emptyError = new z.ZodError([])
    expect(getFirstZodError(emptyError)).toBe('validation.unknown')
  })
})
