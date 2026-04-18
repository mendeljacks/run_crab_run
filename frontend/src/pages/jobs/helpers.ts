import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { fetchSql, callReducer, runStatusToStdb, optionToStdb, timestampToStdb, stdbToTimestamp, stdbToOptionTimestamp, stdbToRunStatus, stdbToOption } from '../../helpers/api'
import type { Job } from '../../stores/types'
import { v4 as uuid } from 'uuid'

export const fetchJobs = async (): Promise<void> => {
    runInAction(() => { store.jobs.loading = true })
    try {
        const jobs = await fetchSql('SELECT id, name, command, schedule, enabled, created_at, updated_at FROM jobs', (row, columns) => {
            const col = (name: string) => row[columns.indexOf(name)]
            return {
                id: col('id') as string,
                name: col('name') as string,
                command: col('command') as string,
                schedule: stdbToOption(col('schedule')),
                enabled: col('enabled') as boolean,
                created_at: stdbToTimestamp(col('created_at')),
                updated_at: stdbToTimestamp(col('updated_at'))
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
        await callReducer('insert_job', [
            id,
            name,
            command,
            optionToStdb(schedule),
            enabled
        ])
        await fetchJobs()
    } catch (e) {
        runInAction(() => {
            store.jobs.error = String(e)
        })
    }
}

export const deleteJob = async (id: string): Promise<void> => {
    try {
        await callReducer('delete_job', [id])
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
        await callReducer('update_job', [
            id,
            updates.name ?? null,
            updates.command ?? null,
            // schedule: Option<Option<String>> in SpacetimeDB
            // null = no change, {"some": {"some": "value"}} = set schedule, {"some": {"none": []}} = clear schedule
            updates.schedule !== undefined
                ? (updates.schedule === null ? { some: { none: [] } } : { some: { some: updates.schedule } })
                : null,
            updates.enabled ?? null
        ])
        await fetchJobs()
    } catch (e) {
        runInAction(() => {
            store.jobs.error = String(e)
        })
    }
}

export const triggerRun = async (jobId: string): Promise<void> => {
    const id = uuid()
    const nowMicros = Date.now() * 1000
    runInAction(() => { store.runs.error = '' })
    try {
        await callReducer('insert_run', [
            id,
            jobId,
            { none: [] },  // terminal_output: None
            { running: [] }, // status: Running
            timestampToStdb(nowMicros),  // started_at
            { none: [] }  // finished_at: None
        ])
    } catch (e) {
        runInAction(() => {
            store.runs.error = String(e)
        })
    }
}

export const deleteRun = async (id: string): Promise<void> => {
    try {
        await callReducer('delete_run', [id])
        await fetchRuns()
    } catch (e) {
        runInAction(() => {
            store.runs.error = String(e)
        })
    }
}

import { fetchRuns } from '../runs/helpers'