export function formatBytes(bytes: number): string {
  if (bytes < 1_000) return `${bytes} Б`
  const units = ['КБ', 'МБ', 'ГБ', 'ТБ']
  let value = bytes / 1_000
  let unitIndex = 0
  while (value >= 1_000 && unitIndex < units.length - 1) {
    value /= 1_000
    unitIndex += 1
  }
  return `${value.toLocaleString('ru-RU', { maximumFractionDigits: 1 })} ${units[unitIndex]}`
}

export function formatDate(value: string | number | null): string {
  if (value === null || value === undefined || value === '') return '—'
  const date = typeof value === 'number' ? new Date(value < 1_000_000_000_000 ? value * 1000 : value) : new Date(value)
  if (Number.isNaN(date.getTime())) return '—'
  return new Intl.DateTimeFormat('ru-RU', {
    year: 'numeric', month: '2-digit', day: '2-digit',
  }).format(date)
}
