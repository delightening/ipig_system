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
        showSupportNotice: false, // 關閉 Locize 贊助訊息，避免 console 干擾
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

