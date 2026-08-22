export interface TrainingRecordWithUser {
  id: string
  user_id: string
  user_email: string
  user_name: string
  course_name: string
  completed_at: string
  expires_at: string | null
  notes: string | null
  created_at: string
}

export interface TrainingUser {
  id: string
  email: string
  display_name: string
}

export interface TrainingForm {
  user_id: string
  course_name: string
  completed_at: string
  expires_at: string
  notes: string
}

export const EXPIRING_DAYS = 30
