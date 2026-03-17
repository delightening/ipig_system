import { MessageSquare, X } from 'lucide-react'

interface FloatingCommentButtonProps {
  onClick: () => void
  isOpen: boolean
}

export function FloatingCommentButton({ onClick, isOpen }: FloatingCommentButtonProps) {
  return (
    <button
      onClick={onClick}
      className="fixed bottom-6 right-6 z-40 h-14 w-14 rounded-full shadow-lg bg-primary text-primary-foreground hover:bg-primary/90 flex items-center justify-center transition-transform hover:scale-105 active:scale-95"
      title={isOpen ? '關閉審查意見' : '開啟審查意見'}
    >
      {isOpen ? <X className="h-6 w-6" /> : <MessageSquare className="h-6 w-6" />}
    </button>
  )
}
