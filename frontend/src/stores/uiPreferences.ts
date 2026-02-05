import { create } from 'zustand'
import { persist } from 'zustand/middleware'

interface UIPreferencesState {
    // 開發者模式：顯示系統號等技術資訊
    developerMode: boolean
    toggleDeveloperMode: () => void
    setDeveloperMode: (enabled: boolean) => void
}

export const useUIPreferences = create<UIPreferencesState>()(
    persist(
        (set) => ({
            developerMode: false,

            toggleDeveloperMode: () => {
                set((state) => ({ developerMode: !state.developerMode }))
            },

            setDeveloperMode: (enabled: boolean) => {
                set({ developerMode: enabled })
            },
        }),
        {
            name: 'ui-preferences',
        }
    )
)
