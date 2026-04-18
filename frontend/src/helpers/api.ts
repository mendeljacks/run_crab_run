import { runInAction } from 'mobx'
import { store } from '../stores/root_store'
import type { RunStatus } from '../stores/types'

const getApiBase = (): string => {
    const globalBase = (window as any).__API_BASE_URL__ as string | undefined
    if (globalBase) return globalBase
    return '/api'
}

const API_BASE = `${getApiBase()}/v1/database/run-crab-run`
const SQL_URL = `${API_BASE}/sql`
const CALL_URL = `${API_BASE}/call`

export const getToken = (): string => {
    return store.auth.token
}

// ── SpacetimeDB type deserializers ──────────────────────────────────────
// SpacetimeDB SQL returns sum types as [tag_index, payload]:
//   Unit enum variant:   [0, []]           (Running=[0], Succeeded=[1], Failed=[2])
//   Option<T>:           [0, value] or [1, []]  (Some=[0], None=[1])
//   Timestamp:           [micros_i64]
//   Option<Timestamp>:   [0, [micros]] or [1, []]

export const stdbToRunStatus = (val: unknown): RunStatus => {
    if (Array.isArray(val) && val.length >= 2) {
        const tag = val[0] as number
        if (tag === 0) return 'Running'
        if (tag === 1) return 'Succeeded'
        if (tag === 2) return 'Failed'
    }
    if (typeof val === 'string') {
        const lower = val.toLowerCase()
        if (lower === 'running') return 'Running'
        if (lower === 'succeeded') return 'Succeeded'
        if (lower === 'failed') return 'Failed'
    }
    return 'Failed'
}

export const stdbToOption = (val: unknown): string | null => {
    if (val === null || val === undefined) return null
    if (typeof val === 'string') return val
    if (Array.isArray(val) && val.length >= 2) {
        const tag = val[0] as number
        if (tag === 1) return null  // None
        if (tag === 0) return val[1] as string // Some
    }
    if (typeof val === 'object' && val !== null) {
        const obj = val as Record<string, unknown>
        if ('some' in obj) return obj.some as string
        if ('none' in obj) return null
    }
    return null
}

export const stdbToTimestamp = (val: unknown): number => {
    if (typeof val === 'number') return val
    if (Array.isArray(val) && val.length === 1) return val[0] as number
    return 0
}

export const stdbToOptionTimestamp = (val: unknown): number | null => {
    if (val === null || val === undefined) return null
    if (Array.isArray(val) && val.length >= 2) {
        const tag = val[0] as number
        if (tag === 1) return null  // None
        if (tag === 0) {
            const inner = val[1]
            if (Array.isArray(inner) && inner.length === 1) return inner[0] as number
            if (typeof inner === 'number') return inner
        }
    }
    return null
}

// ── SpacetimeDB type serializers for reducer calls ───────────────────────
// SpacetimeDB expects JSON with these formats:
//   Unit enum variant:   {"variant_name": []}
//   Option<T>:           {"some": value} or {"none": []}
//   Timestamp:           [micros_i64]

export const runStatusToStdb = (status: RunStatus): unknown => {
    switch (status) {
        case 'Running': return { running: [] }
        case 'Succeeded': return { succeeded: [] }
        case 'Failed': return { failed: [] }
    }
}

export const optionToStdb = <T>(val: T | null | undefined): unknown => {
    if (val === null || val === undefined) return { none: [] }
    return { some: val }
}

export const timestampToStdb = (micros: number): unknown[] => {
    return [micros]
}

export const optionTimestampToStdb = (micros: number | null): unknown => {
    if (micros === null || micros === undefined) return { none: [] }
    return { some: [micros] }
}

// ── SQL API ────────────────────────────────────────────────────────────

interface SqlResult {
    schema: {
        elements: Array<{
            name: { some: string }
            algebraic_type: unknown
        }>
    }
    rows: unknown[][]
    total_duration_micros: number
}

const sqlHeaders = (): HeadersInit => ({
    'Content-Type': 'text/plain',
    'Authorization': `Bearer ${getToken()}`
})

export const fetchSql = async <T>(query: string, mapRow: (row: unknown[], columns: string[]) => T): Promise<T[]> => {
    const res = await fetch(SQL_URL, {
        method: 'POST',
        headers: sqlHeaders(),
        body: query
    })

    if (!res.ok) {
        const text = await res.text()
        throw new Error(`SQL error: ${text}`)
    }

    const results: SqlResult[] = await res.json()

    if (!results.length) return []

    const result = results[0]
    const columns = result.schema.elements.map(e => e.name.some)

    return result.rows.map(row => mapRow(row, columns))
}

// ── Reducer API ─────────────────────────────────────────────────────────

const callHeaders = (): HeadersInit => ({
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${getToken()}`
})

export const callReducer = async (reducerName: string, args: unknown[]): Promise<void> => {
    const res = await fetch(`${CALL_URL}/${reducerName}`, {
        method: 'POST',
        headers: callHeaders(),
        body: JSON.stringify(args)
    })

    if (!res.ok) {
        const text = await res.text()
        throw new Error(`Reducer error: ${text}`)
    }
}