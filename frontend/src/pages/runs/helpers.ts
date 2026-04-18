import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { fetchSql } from '../../helpers/api'
import type { Run, RunStatus } from '../../stores/types'

export const fetchRunById = async (id: string): Promise<void> => {
    runInAction(() => { store.runs.loading = true })
    try {
        const query = `SELECT id, job_id, terminal_output, status, started_at, finished_at, created_at, updated_at FROM runs WHERE id = '${id}'`
        const runs = await fetchSql(query, (row) => {
            return {
                id: row[0] as string,
                job_id: row[1] as string,
                terminal_output: row[2] as string | null,
                status: row[3] as RunStatus,
                started_at: row[4] as number,
                finished_at: row[5] as number | null,
                created_at: row[6] as number,
                updated_at: row[7] as number
            } satisfies Run
        })
        runInAction(() => {
            const existing = store.runs.list.filter(r => r.id !== id)
            store.runs.list = [...existing, ...runs]
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
        const whereClause = jobId ? ` WHERE job_id = '${jobId}'` : ''
        const query = `SELECT id, job_id, terminal_output, status, started_at, finished_at, created_at, updated_at FROM runs${whereClause} ORDER BY started_at DESC`
        const runs = await fetchSql(query, (row) => {
            return {
                id: row[0] as string,
                job_id: row[1] as string,
                terminal_output: row[2] as string | null,
                status: row[3] as RunStatus,
                started_at: row[4] as number,
                finished_at: row[5] as number | null,
                created_at: row[6] as number,
                updated_at: row[7] as number
            } satisfies Run
        })
        runInAction(() => {
            store.runs.list = runs
            store.runs.loading = false
        })
    } catch (e) {
        runInAction(() => {
            store.runs.error = String(e)
            store.runs.loading = false
        })
    }
}