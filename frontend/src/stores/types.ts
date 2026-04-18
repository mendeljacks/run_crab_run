export type RunStatus = 'Running' | 'Succeeded' | 'Failed'

export interface Job {
    id: string
    name: string
    command: string
    schedule: string | null
    enabled: boolean
    created_at: string
    updated_at: string
}

export interface Run {
    id: string
    job_id: string
    terminal_output: string | null
    status: RunStatus
    started_at: string
    finished_at: string | null
    created_at: string
    updated_at: string
}