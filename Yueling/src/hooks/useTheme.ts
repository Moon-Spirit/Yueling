import { ref, onMounted } from 'vue'
import { getItem, setItem } from '../utils/storage'

export function useTheme() {
  const isDarkMode = ref(false)

  const checkThemePreference = () => {
    const savedTheme = getItem<string>('theme')
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    isDarkMode.value = savedTheme ? savedTheme === 'dark' : prefersDark
    updateTheme()
  }

  const updateTheme = () => {
    document.documentElement.setAttribute('data-theme', isDarkMode.value ? 'dark' : 'light')
    setItem('theme', isDarkMode.value ? 'dark' : 'light')
  }

  const toggleTheme = () => {
    isDarkMode.value = !isDarkMode.value
    updateTheme()
  }

  onMounted(() => {
    checkThemePreference()
    
    // 监听系统主题变化
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = () => {
      checkThemePreference()
    }
    
    mediaQuery.addEventListener('change', handleChange)
    
    return () => {
      mediaQuery.removeEventListener('change', handleChange)
    }
  })

  return {
    isDarkMode,
    toggleTheme,
    checkThemePreference
  }
}