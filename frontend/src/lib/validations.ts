import { z } from 'zod'

// ─── Reusable field schemas ───

export const taxIdSchema = z
  .string()
  .refine(
    (v) => v === '' || /^\d{8}$/.test(v),
    { message: 'validation.taxId' },
  )

export const phoneSchema = z
  .string()
  .refine(
    (v) => v === '' || /^\d{9,10}$/.test(v),
    { message: 'validation.phone' },
  )

export const emailOptionalSchema = z
  .string()
  .refine(
    (v) => v === '' || /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v),
    { message: 'validation.email' },
  )

export const nonEmptyString = (messageKey = 'validation.required') =>
  z.string().min(1, { message: messageKey })

// ─── Partner form schema ───

export const partnerFormSchema = z.object({
  code: nonEmptyString('validation.required'),
  name: nonEmptyString('validation.required'),
  type: z.enum(['customer', 'vendor', 'both']),
  tax_id: taxIdSchema.optional().default(''),
  contact_person: z.string().optional().default(''),
  phone: phoneSchema.optional().default(''),
  email: emailOptionalSchema.optional().default(''),
  address: z.string().optional().default(''),
  bank_account: z.string().optional().default(''),
  notes: z.string().optional().default(''),
})

export type PartnerFormData = z.infer<typeof partnerFormSchema>

// ─── Warehouse form schema ───

export const warehouseFormSchema = z.object({
  name: nonEmptyString('validation.required'),
  code: z.string().optional().default(''),
  address: z.string().optional().default(''),
  description: z.string().optional().default(''),
  is_active: z.boolean().default(true),
})

export type WarehouseFormData = z.infer<typeof warehouseFormSchema>

// ─── Animal form schema ───

export const animalFormSchema = z.object({
  ear_tag: nonEmptyString('validation.required'),
  breed: z.enum(['minipig', 'miniature', 'white', 'LYD', 'other']),
  breed_other: z.string().optional().default(''),
  gender: z.enum(['male', 'female']),
  source_id: z.string().optional().default(''),
  pen_location: nonEmptyString('validation.required'),
  entry_date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, { message: 'validation.dateFormat' }),
  birth_date: z.string().regex(/^\d{4}-\d{2}-\d{2}$/, { message: 'validation.dateFormat' }),
  entry_weight: z
    .string()
    .optional()
    .default('')
    .refine(
      (v) => v === '' || (!isNaN(parseFloat(v)) && parseFloat(v) > 0),
      { message: 'validation.positiveNumber' },
    ),
  pre_experiment_code: nonEmptyString('validation.required'),
  remark: z.string().optional().default(''),
})

export type AnimalFormData = z.infer<typeof animalFormSchema>

// ─── Helper: extract first error message from ZodError ───

export function getFirstZodError(error: z.ZodError): string {
  const first = error.issues[0]
  return (first && 'message' in first ? String(first.message) : undefined) ?? 'validation.unknown'
}
