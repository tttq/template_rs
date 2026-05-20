import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export type ThemeMode = 'light' | 'dark' | 'system'
export type FontSize = 'small' | 'default' | 'large'
export type Locale = 'zh-CN' | 'en-US'

export const FONT_SIZE_PX: Record<FontSize, number> = {
  small: 12,
  default: 14,
  large: 16,
}

function getSystemTheme(): 'light' | 'dark' {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function applyTheme(mode: ThemeMode) {
  const effective = mode === 'system' ? getSystemTheme() : mode
  document.documentElement.setAttribute('data-theme', effective)
}

function applyFontSize(size: FontSize) {
  document.documentElement.style.setProperty('--app-font-size', `${FONT_SIZE_PX[size]}px`)
}

export const useAppStore = defineStore('app', () => {
  const collapsed = ref(false)
  const themeMode = ref<ThemeMode>((localStorage.getItem('themeMode') as ThemeMode) || 'light')
  const fontSize = ref<FontSize>((localStorage.getItem('fontSize') as FontSize) || 'default')
  const locale = ref<Locale>((localStorage.getItem('locale') as Locale) || 'zh-CN')

  const isDark = ref(themeMode.value === 'dark' || (themeMode.value === 'system' && getSystemTheme() === 'dark'))

  function toggleCollapsed() {
    collapsed.value = !collapsed.value
  }

  function setThemeMode(mode: ThemeMode) {
    themeMode.value = mode
    localStorage.setItem('themeMode', mode)
    applyTheme(mode)
    isDark.value = mode === 'dark' || (mode === 'system' && getSystemTheme() === 'dark')
  }

  function setFontSize(size: FontSize) {
    fontSize.value = size
    localStorage.setItem('fontSize', size)
    applyFontSize(size)
  }

  function setLocale(lang: Locale) {
    locale.value = lang
    localStorage.setItem('locale', lang)
  }

  applyTheme(themeMode.value)
  applyFontSize(fontSize.value)

  if (themeMode.value === 'system') {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
      isDark.value = e.matches
      applyTheme('system')
    })
  }

  watch(themeMode, (mode) => {
    if (mode === 'system') {
      window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
        isDark.value = e.matches
        applyTheme('system')
      })
    }
  })

  return {
    collapsed,
    themeMode,
    fontSize,
    locale,
    isDark,
    toggleCollapsed,
    setThemeMode,
    setFontSize,
    setLocale,
  }
})
