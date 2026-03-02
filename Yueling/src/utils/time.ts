export function formatTime(timestamp: number): string {
  // 检查是否是秒级时间戳（小于10^12），如果是则转换为毫秒级
  const msTimestamp = timestamp < 1000000000000 ? timestamp * 1000 : timestamp
  return new Date(msTimestamp).toLocaleTimeString('zh-CN', { hour: '2-digit', minute: '2-digit' })
}

export function formatDate(timestamp: number): string {
  // 检查是否是秒级时间戳（小于10^12），如果是则转换为毫秒级
  const msTimestamp = timestamp < 1000000000000 ? timestamp * 1000 : timestamp
  return new Date(msTimestamp).toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
  })
}

export function formatDateTime(timestamp: number): string {
  // 检查是否是秒级时间戳（小于10^12），如果是则转换为毫秒级
  const msTimestamp = timestamp < 1000000000000 ? timestamp * 1000 : timestamp
  return new Date(msTimestamp).toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit'
  })
}

export function isToday(timestamp: number): boolean {
  // 检查是否是秒级时间戳（小于10^12），如果是则转换为毫秒级
  const msTimestamp = timestamp < 1000000000000 ? timestamp * 1000 : timestamp
  const today = new Date()
  const date = new Date(msTimestamp)
  return (
    date.getFullYear() === today.getFullYear() &&
    date.getMonth() === today.getMonth() &&
    date.getDate() === today.getDate()
  )
}

export function isYesterday(timestamp: number): boolean {
  // 检查是否是秒级时间戳（小于10^12），如果是则转换为毫秒级
  const msTimestamp = timestamp < 1000000000000 ? timestamp * 1000 : timestamp
  const yesterday = new Date()
  yesterday.setDate(yesterday.getDate() - 1)
  const date = new Date(msTimestamp)
  return (
    date.getFullYear() === yesterday.getFullYear() &&
    date.getMonth() === yesterday.getMonth() &&
    date.getDate() === yesterday.getDate()
  )
}

export function getRelativeTime(timestamp: number): string {
  // 检查是否是秒级时间戳（小于10^12），如果是则转换为毫秒级
  const msTimestamp = timestamp < 1000000000000 ? timestamp * 1000 : timestamp
  const now = Date.now()
  const diff = now - msTimestamp
  
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) {
    return `${days}天前`
  } else if (hours > 0) {
    return `${hours}小时前`
  } else if (minutes > 0) {
    return `${minutes}分钟前`
  } else {
    return '刚刚'
  }
}