import { useEffect, useState } from 'react'

/** 使用 IntersectionObserver 偵測目前可見的 section[data-section] 元素 */
export function useCurrentSection() {
  const [currentSection, setCurrentSection] = useState<string | undefined>()

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            const name = (entry.target as HTMLElement).dataset.section
            if (name) setCurrentSection(name)
          }
        }
      },
      { rootMargin: '-20% 0px -60% 0px' }
    )

    const sections = document.querySelectorAll('section[data-section]')
    sections.forEach((s) => observer.observe(s))

    return () => observer.disconnect()
  }, [])

  return currentSection
}
