import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'

import zhTW from '@/locales/zh-TW.json'
import en from '@/locales/en.json'

const resources = {
    'zh-TW': { translation: zhTW },
    'en': { translation: en },
}

i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init({
        resources,
        fallbackLng: 'zh-TW',
        detection: {
            order: ['localStorage', 'navigator'],
            lookupLocalStorage: 'ipig-language',
            caches: ['localStorage'],
        },
        interpolation: {
            escapeValue: false, // React already escapes values
        },
    })

export default i18n
