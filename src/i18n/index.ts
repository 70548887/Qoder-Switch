import { ref } from 'vue'
import zh from './zh'
import en from './en'

type Messages = typeof zh
const messages: Record<string, Messages> = { zh, en }

export const locale = ref<string>('zh')

export function useI18n() {
  const t = (key: string): string => {
    const keys = key.split('.')
    let val: any = messages[locale.value] || messages.zh
    for (const k of keys) {
      val = val?.[k]
      if (val === undefined) return key
    }
    return typeof val === 'string' ? val : key
  }

  const setLocale = (lang: string) => {
    if (messages[lang]) {
      locale.value = lang
    }
  }

  return { t, locale, setLocale }
}
