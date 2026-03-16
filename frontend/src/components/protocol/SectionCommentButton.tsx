import { MessageSquarePlus } from 'lucide-react'

import { Button } from '@/components/ui/button'

interface SectionCommentButtonProps {
  sectionName: string
  onAddComment: (sectionName: string) => void
  visible: boolean
  disabled?: boolean
}

export function SectionCommentButton({
  sectionName,
  onAddComment,
  visible,
  disabled,
}: SectionCommentButtonProps) {
  if (!visible) return null

  return (
    <Button
      variant="ghost"
      size="sm"
      className="text-muted-foreground hover:text-primary hover:bg-primary/10 transition-opacity"
      onClick={(e) => {
        e.stopPropagation()
        onAddComment(sectionName)
      }}
      disabled={disabled}
      title={`對「${sectionName}」新增審查意見`}
    >
      <MessageSquarePlus className="h-5 w-5" />
    </Button>
  )
}
