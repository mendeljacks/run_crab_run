// API helper module — authentication + HTTP wrappers for the backend

export const getApiBase = (): string => {
    const globalBase = (window as any).__API_BASE_URL__ as string | undefined
    if (globalBase) return globalBase
    return '/api'
}

export const getToken = async (): Promise<string> => {
    const { data: { session } } = await supabase.auth.getSession()
    if (!session?.access_token) {
        throw new Error('Not authenticated')
    }
    return session.access_token
}

// Re-export supabase for convenience
import { supabase } from '../config/supabase'

// ── API helpers ──────────────────────────────────────────────────────────

const apiHeaders = async (): Promise<HeadersInit> => {
    const token = await getToken()
    return {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${token}`
    }
}

export const apiGet = async <T>(path: string): Promise<T> => {
    const headers = await apiHeaders()
    const res = await fetch(`${getApiBase()}${path}`, { headers })

    if (!res.ok) {
        const text = await res.text()
        throw new Error(`API error (${res.status}): ${text}`)
    }

    return res.json()
}

export const apiPost = async <T>(path: string, body: unknown): Promise<T> => {
    const headers = await apiHeaders()
    const res = await fetch(`${getApiBase()}${path}`, {
        method: 'POST',
        headers,
        body: JSON.stringify(body)
    })

    if (!res.ok) {
        const text = await res.text()
        throw new Error(`API error (${res.status}): ${text}`)
    }

    return res.json()
}

export const apiPatch = async <T>(path: string, body: unknown): Promise<T> => {
    const headers = await apiHeaders()
    const res = await fetch(`${getApiBase()}${path}`, {
        method: 'PATCH',
        headers,
        body: JSON.stringify(body)
    })

    if (!res.ok) {
        const text = await res.text()
        throw new Error(`API error (${res.status}): ${text}`)
    }

    return res.json()
}

export const apiDelete = async <T>(path: string): Promise<T> => {
    const headers = await apiHeaders()
    const res = await fetch(`${getApiBase()}${path}`, {
        method: 'DELETE',
        headers
    })

    if (!res.ok) {
        const text = await res.text()
        throw new Error(`API error (${res.status}): ${text}`)
    }

    return res.json()
}