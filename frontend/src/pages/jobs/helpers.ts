import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { fetchSql, callReducer } from '../../helpers/api'
import type { Job } from '../../stores/types'
import { v4 as uuid } from 'uuid'

export const fetchJobs = async (): Promise<void> => {
    runInAction(() => { store.jobs.loading = true })
    try {
        const jobs = await fetchSql('SELECT id, name, command, schedule, enabled, created_at, updated_at FROM jobs', (row) => {
            return {
                id: row[0] as string,
                name: row[1] as string,
                command: row[2] as string,
                schedule: row[3] as string | null,
                enabled: row[4] as boolean,
                created_at: row[5] as number,
                updated_at: row[6] as number
            } satisfies Job
        })
        runInAction(() => {
            store.jobs.list = jobs
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
    const id = uuid()
    runInAction(() => { store.jobs.error = '' })
    try {
        await callReducer('insert_job', [id, name, command, schedule || null, enabled])
        await fetchJobs()
    } catch (e) {
        runInAction(() => {
            store.jobs.error = String(e)
        })
    }
}

export const insertRun = async (jobId: string, status: 'Running' | 'Succeeded' | 'Failed', terminalOutput: string | null, startedAt: number, finishedAt: number | null): Promise<void> => {
    const id = uuid()
    runInAction(() => { store.runs.error = '' })
    try {
        await callReducer('insert_run', [id, jobId, terminalOutput, status, startedAt, finishedAt])
    } catch (e) {
        runInAction(() => {
            store.runs.error = String(e)
        })
    }
}