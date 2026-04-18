export const formatTimeAgo = (micros: number): string => {
    const now = Date.now() * 1000
    const diff = now - micros
    const secs = Math.floor(diff / 1_000_000)

    if (secs < 0) return 'just now'
    if (secs < 60) return `${secs} second${secs === 1 ? '' : 's'} ago`
    if (secs < 3600) return `${Math.floor(secs / 60)} minute${Math.floor(secs / 60) === 1 ? '' : 's'} ago`
    if (secs < 86400) return `${Math.floor(secs / 3600)} hour${Math.floor(secs / 3600) === 1 ? '' : 's'} ago`
    return `${Math.floor(secs / 86400)} day${Math.floor(secs / 86400) === 1 ? '' : 's'} ago`
}

export const formatDatetime = (micros: number): string => {
    const ms = micros / 1000
    return new Date(ms).toISOString().replace('T', ' ').slice(0, 19)
}

export const shortId = (id: string): string => {
    return id.length > 8 ? id.slice(0, 8) : id
}

export const statusColor = (status: string): string => {
    switch (status) {
        case 'Running': return '#0ea5e9'
        case 'Succeeded': return '#10b981'
        case 'Failed': return '#ef4444'
        default: return '#64748b'
    }
}

export const statusBgColor = (status: string): string => {
    switch (status) {
        case 'Running': return '#e0f2fe'
        case 'Succeeded': return '#d1fae5'
        case 'Failed': return '#fee2e2'
        default: return '#f1f5f9'
    }
}

export const formatSchedule = (schedule: string | null): string => {
    if (!schedule) return 'Manual'
    if (schedule.length > 40) return schedule.slice(0, 37) + '…'
    return schedule
}

export const enabledLabel = (enabled: boolean): string => {
    return enabled ? 'Enabled' : 'Disabled'
}