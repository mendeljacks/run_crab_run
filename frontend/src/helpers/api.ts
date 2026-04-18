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

const callHeaders = (): HeadersInit => ({
    'Content-Type': 'application/json',
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

import { store } from '../stores/root_store'