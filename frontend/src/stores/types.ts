export type RunStatus = 'Running' | 'Succeeded' | 'Failed'

export interface Job {
    id: string
    name: string
    command: string
    schedule: string | null
    enabled: boolean
    created_at: number
    updated_at: number
}

export interface Run {
    id: string
    job_id: string
    terminal_output: string | null
    status: RunStatus
    started_at: number
    finished_at: number | null
    created_at: number
    updated_at: number
}

export interface SqlResult {
    schema: {
        elements: Array<{
            name: { some: string }
            algebraic_type: unknown
        }>
    }
    rows: unknown[][]
    total_duration_micros: number
    stats: {
        rows_inserted: number
        rows_deleted: number
        rows_updated: number
    }
}

export interface JobsResponse {
    jobs: Job[]
}

export interface RunsResponse {
    runs: Run[]
    total: number
}