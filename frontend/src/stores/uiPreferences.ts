import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type FontSizePreference = 'default' | 'large' | 'xl'

interface UIPreferencesState {
    // 開發者模式：顯示系統號等技術資訊
    developerMode: boolean
    toggleDeveloperMode: () => void
    setDeveloperMode: (enabled: boolean) => void
    // 字體大小偏好（只能放大，不縮小）
    fontSize: FontSizePreference
    setFontSize: (size: FontSizePreference) => void
}

export const useUIPreferences = create<UIPreferencesState>()(
    persist(
        (set) => ({
            developerMode: false,
            fontSize: 'default',

            toggleDeveloperMode: () => {
                set((state) => ({ developerMode: !state.developerMode }))
            },

            setDeveloperMode: (enabled: boolean) => {
                set({ developerMode: enabled })
            },

            setFontSize: (size: FontSizePreference) => {
                set({ fontSize: size })
            },
        }),
        {
            name: 'ui-preferences',
        }
    )
)
