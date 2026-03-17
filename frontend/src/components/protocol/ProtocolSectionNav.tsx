import { useTranslation } from 'react-i18next'

interface SectionItem {
  id: string
  label: string
}

interface ProtocolSectionNavProps {
  sections: SectionItem[]
  currentSection?: string
}

export function ProtocolSectionNav({ sections, currentSection }: ProtocolSectionNavProps) {
  const { t } = useTranslation()

  const handleClick = (id: string) => {
    const el = document.querySelector(`section.${id}`)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  }

  return (
    <nav className="sticky top-20 space-y-1">
      <h3 className="text-sm font-semibold text-muted-foreground mb-2 px-2">
        {t('protocols.content.toc', '目錄')}
      </h3>
      <ul className="space-y-0.5">
        {sections.map((section) => {
          const isActive = currentSection === section.label
          return (
            <li key={section.id}>
              <button
                onClick={() => handleClick(section.id)}
                className={`w-full text-left text-sm px-2 py-1.5 rounded transition-colors ${
                  isActive
                    ? 'bg-blue-50 text-blue-700 font-semibold'
                    : 'text-muted-foreground hover:text-foreground hover:bg-slate-50'
                }`}
              >
                {section.label}
              </button>
            </li>
          )
        })}
      </ul>
    </nav>
  )
}
