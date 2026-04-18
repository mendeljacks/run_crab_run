import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { supabase } from '../../config/supabase'

export const authenticate = async (): Promise<void> => {
    runInAction(() => { store.auth.loading = true })
    try {
        // Check for existing session
        const { data: { session } } = await supabase.auth.getSession()

        if (session?.access_token) {
            runInAction(() => {
                store.auth.token = session.access_token
                store.auth.loading = false
            })
            return
        }

        // Try to restore from localStorage (Supabase persists sessions)
        const { data: { session: restoredSession } } = await supabase.auth.getSession()
        if (restoredSession?.access_token) {
            runInAction(() => {
                store.auth.token = restoredSession.access_token
                store.auth.loading = false
            })
            return
        }

        // No existing session — user needs to sign in
        runInAction(() => {
            store.auth.loading = false
            store.auth.token = ''
        })
    } catch (e) {
        runInAction(() => {
            store.auth.loading = false
            store.auth.token = ''
        })
    }
}

export const signInAnonymously = async (): Promise<void> => {
    runInAction(() => { store.auth.loading = true })
    try {
        const { data, error } = await supabase.auth.signInAnonymously()

        if (error) {
            throw error
        }

        if (!data.session?.access_token) {
            throw new Error('No session returned from anonymous sign-in')
        }

        runInAction(() => {
            store.auth.token = data.session!.access_token
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

export const signOut = async (): Promise<void> => {
    await supabase.auth.signOut()
    runInAction(() => {
        store.auth.token = ''
    })
}