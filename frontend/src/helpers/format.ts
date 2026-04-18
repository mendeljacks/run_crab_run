import { format, formatDistanceToNow, intervalToDuration, formatDuration } from 'date-fns'

const isoToDate = (iso: string | number): Date => {
    if (typeof iso === 'number') return new Date(iso / 1000) // Legacy microsecond support
    return new Date(iso)
}

export const formatTimeAgo = (timestamp: string | number): string => {
    return formatDistanceToNow(isoToDate(timestamp), { addSuffix: true })
}

export const formatDatetime = (timestamp: string | number): string => {
    return format(isoToDate(timestamp), 'yyyy-MM-dd HH:mm:ss')
}

export const formatElapsed = (micros: number): string => {
    const ms = micros / 1000
    const duration = intervalToDuration({ start: 0, end: ms })
    // Show up to the two largest non-zero units
    return formatDuration(duration, {
        format: ['hours', 'minutes', 'seconds'],
        zero: false,
    })
}

/** Compute elapsed microseconds between two ISO timestamps */
export const elapsedMicros = (start: string, end: string): number => {
    return (new Date(end).getTime() - new Date(start).getTime()) * 1000
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