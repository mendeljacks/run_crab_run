import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { fetchSql, stdbToTimestamp, stdbToOptionTimestamp, stdbToRunStatus, stdbToOption } from '../../helpers/api'
import type { Run, RunStatus } from '../../stores/types'

export const fetchRunById = async (id: string): Promise<void> => {
    runInAction(() => { store.runs.loading = true })
    try {
        const query = `SELECT id, job_id, terminal_output, status, started_at, finished_at, created_at, updated_at FROM runs WHERE id = '${id}'`
        const runs = await fetchSql(query, (row, columns) => {
            const col = (name: string) => row[columns.indexOf(name)]
            return {
                id: col('id') as string,
                job_id: col('job_id') as string,
                terminal_output: stdbToOption(col('terminal_output')),
                status: stdbToRunStatus(col('status')),
                started_at: stdbToTimestamp(col('started_at')),
                finished_at: stdbToOptionTimestamp(col('finished_at')),
                created_at: stdbToTimestamp(col('created_at')),
                updated_at: stdbToTimestamp(col('updated_at'))
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
        const runs = await fetchSql(query, (row, columns) => {
            const col = (name: string) => row[columns.indexOf(name)]
            return {
                id: col('id') as string,
                job_id: col('job_id') as string,
                terminal_output: stdbToOption(col('terminal_output')),
                status: stdbToRunStatus(col('status')),
                started_at: stdbToTimestamp(col('started_at')),
                finished_at: stdbToOptionTimestamp(col('finished_at')),
                created_at: stdbToTimestamp(col('created_at')),
                updated_at: stdbToTimestamp(col('updated_at'))
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