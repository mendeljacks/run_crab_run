import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { apiGet, apiPost, apiPatch, apiDelete } from '../../helpers/api'
import type { Job } from '../../stores/types'

export const fetchJobs = async (): Promise<void> => {
    runInAction(() => { store.jobs.loading = true })
    try {
        const data = await apiGet<{ jobs: Job[] }>('/jobs')
        runInAction(() => {
            store.jobs.list = data.jobs
            store.jobs.loading = false
        })
    } catch (e) {
        runInAction(() => {
            store.jobs.error = String(e)
            store.jobs.loading = false
        })
    }
}

export const createJob = async (name: string, command: string, schedule: string | null, enabled: boolean): Promise<void> => {
    runInAction(() => { store.jobs.error = '' })
    try {
        await apiPost('/jobs', { name, command, schedule, enabled })
        await fetchJobs()
    } catch (e) {
        runInAction(() => {
            store.jobs.error = String(e)
        })
    }
}

export const deleteJob = async (id: string): Promise<void> => {
    try {
        await apiDelete(`/jobs/${id}`)
        await fetchJobs()
    } catch (e) {
        runInAction(() => {
            store.jobs.error = String(e)
        })
    }
}

export const updateJob = async (id: string, updates: { name?: string; command?: string; schedule?: string | null; enabled?: boolean }): Promise<void> => {
    runInAction(() => { store.jobs.error = '' })
    try {
        await apiPatch(`/jobs/${id}`, updates)
        await fetchJobs()
    } catch (e) {
        runInAction(() => {
            store.jobs.error = String(e)
        })
    }
}

export const triggerRun = async (jobId: string): Promise<void> => {
    runInAction(() => { store.runs.error = '' })
    try {
        await apiPost('/runs', { job_id: jobId, status: 'Running' })
    } catch (e) {
        runInAction(() => {
            store.runs.error = String(e)
        })
    }
}

export const deleteRun = async (id: string): Promise<void> => {
    try {
        await apiDelete(`/runs/${id}`)
        await fetchRuns()
    } catch (e) {
        runInAction(() => {
            store.runs.error = String(e)
        })
    }
}

import { fetchRuns } from '../runs/helpers'