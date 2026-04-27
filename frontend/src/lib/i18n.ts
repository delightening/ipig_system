import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'

import zhTW from '@/locales/zh-TW.json'
import en from '@/locales/en.json'

const resources = {
    'zh-TW': { translation: zhTW },
    'en': { translation: en },
}

// Map i18n language codes to HTML lang attribute values
const langMap: Record<string, string> = {
    'zh-TW': 'zh-TW',
    'en': 'en',
}

// Update the HTML lang attribute to match the current language
// This affects native browser elements like date pickers
const updateHtmlLang = (lng: string) => {
    const htmlLang = langMap[lng] || lng
    document.documentElement.lang = htmlLang
}

i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
        resources,
        fallbackLng: 'zh-TW',
        // R29-3 i18next v26：showSupportNotice 選項已移除（連同 console support notice 一併刪除），
        // 不再需要本地關閉設定。詳見 i18next v26.0.0 release notes。
        detection: {
            order: ['localStorage', 'navigator'],
            lookupLocalStorage: 'ipig-language',
            caches: ['localStorage'],
        },
        interpolation: {
            escapeValue: false, // React already escapes values
        },
    })

// Set initial HTML lang attribute
updateHtmlLang(i18n.language)

// Listen for language changes and update HTML lang attribute
i18n.on('languageChanged', updateHtmlLang)

export default i18n

