import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { apiGet } from '../../helpers/api'
import type { Run } from '../../stores/types'

export const fetchRunById = async (id: string): Promise<void> => {
    runInAction(() => { store.runs.loading = true })
    try {
        const data = await apiGet<{ run: Run }>(`/runs/${id}`)
        runInAction(() => {
            const existing = store.runs.list.filter(r => r.id !== id)
            store.runs.list = [...existing, data.run]
            store.runs.loading = false
        })
    } catch (e) {
        runInAction(() => {
            store.runs.error = String(e)
            store.runs.loading = false
        })
    }
}

export const fetchRuns = async (jobId?: string): Promise<void> => {
    runInAction(() => { store.runs.loading = true })
    try {
        const path = jobId ? `/runs?job_id=${jobId}` : '/runs'
        const data = await apiGet<{ runs: Run[] }>(path)
        runInAction(() => {
            store.runs.list = data.runs
            store.runs.loading = false
        })
    } catch (e) {
        runInAction(() => {
            store.runs.error = String(e)
            store.runs.loading = false
        })
    }
}