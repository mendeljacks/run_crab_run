import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'

const getApiBase = (): string => {
    const globalBase = (window as any).__API_BASE_URL__ as string | undefined
    if (globalBase) return globalBase
    return '/api'
}

export const authenticate = async (): Promise<void> => {
    runInAction(() => { store.auth.loading = true })
    try {
        const savedToken = localStorage.getItem('stdb_token')
        if (savedToken) {
            runInAction(() => {
                store.auth.token = savedToken
                store.auth.loading = false
            })
            return
        }
        await requestNewToken()
    } catch (e) {
        runInAction(() => {
            store.auth.loading = false
            store.auth.token = ''
        })
    }
}

export const requestNewToken = async (): Promise<void> => {
    runInAction(() => { store.auth.loading = true })
    try {
        const res = await fetch(`${getApiBase()}/v1/identity`, {
            method: 'POST'
        })

        if (!res.ok) {
            throw new Error(`Auth failed: ${res.status}`)
        }

        const data = await res.json()
        const token = data.token as string

        if (!token) {
            throw new Error('No token in response')
        }

        localStorage.setItem('stdb_token', token)
        runInAction(() => {
            store.auth.token = token
            store.auth.loading = false
        })
    } catch (e) {
        runInAction(() => {
            store.auth.loading = false
            store.auth.token = ''
        })
        throw e
    }
}