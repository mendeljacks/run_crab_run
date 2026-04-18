import { exec } from 'child_process'
import { sql } from './db.js'

const COMMAND_TIMEOUT_MS = 5 * 60 * 1000 // 5 minutes

interface Run {
    id: string
    job_id: string
    status: string
    terminal_output: string | null
    started_at: string
}

interface Job {
    id: string
    name: string
    command: string
    schedule: string | null
    enabled: boolean
}

/**
 * Pick up runs that are in 'Running' state but haven't been executed yet.
 * We identify these as runs with status='Running' that have no terminal_output
 * (the runner hasn't processed them yet).
 */
export async function processPendingRuns(): Promise<void> {
    const runs = await sql<Run[]>`
        SELECT id, job_id, status, terminal_output, started_at
        FROM runs
        WHERE status = 'Running' AND terminal_output IS NULL
        ORDER BY started_at ASC
    `

    if (runs.length === 0) return

    console.log(`[runner] Found ${runs.length} pending run(s) to process`)

    for (const run of runs) {
        // Mark as being processed by setting a placeholder terminal_output
        // This prevents other runners from picking up the same run
        await sql`
            UPDATE runs SET terminal_output = '__RUNNING__' WHERE id = ${run.id}
        `

        const [job] = await sql<Job[]>`
            SELECT id, name, command, schedule, enabled FROM jobs WHERE id = ${run.job_id}
        `

        if (!job) {
            console.error(`[runner] Job ${run.job_id} not found for run ${run.id}`)
            await sql`
                UPDATE runs SET
                    status = 'Failed',
                    terminal_output = ${'Error: Job not found'},
                    finished_at = NOW(),
                    updated_at = NOW()
                WHERE id = ${run.id}
            `
            continue
        }

        if (!job.enabled) {
            console.log(`[runner] Job "${job.name}" is disabled, marking run ${run.id} as Failed`)
            await sql`
                UPDATE runs SET
                    status = 'Failed',
                    terminal_output = ${'Error: Job is disabled'},
                    finished_at = NOW(),
                    updated_at = NOW()
                WHERE id = ${run.id}
            `
            continue
        }

        console.log(`[runner] Executing run ${run.id.slice(0, 8)} for job "${job.name}": ${job.command}`)
        await executeRun(run.id, job.command)
    }
}

async function executeRun(runId: string, command: string): Promise<void> {
    return new Promise((resolve) => {
        exec(command, { timeout: COMMAND_TIMEOUT_MS, maxBuffer: 1024 * 1024, shell: '/bin/bash' }, (error, stdout, stderr) => {
            const output = (stdout || '') + (stderr || '')
            const status = error ? 'Failed' : 'Succeeded'
            const terminalOutput = output || (error ? error.message : '(no output)')

            sql`
                UPDATE runs SET
                    status = ${status},
                    terminal_output = ${terminalOutput.trim()},
                    finished_at = NOW(),
                    updated_at = NOW()
                WHERE id = ${runId}
            `.then(() => {
                console.log(`[runner] Run ${runId.slice(0, 8)} completed: ${status}`)
                resolve()
            }).catch((err: Error) => {
                console.error(`[runner] Failed to update run ${runId}:`, err)
                resolve()
            })
        })
    })
}

/**
 * Check scheduled (enabled) jobs and create runs for any that are due.
 * A job is considered "due" if it has no completed runs yet, or the most recent
 * run was more than 1 hour ago (simple heuristic for now — proper RRULE parsing
 * can be added later).
 */
export async function scheduleDueJobs(): Promise<void> {
    const jobs = await sql<Job[]>`
        SELECT id, name, command, schedule, enabled FROM jobs
        WHERE enabled = true AND schedule IS NOT NULL
    `

    for (const job of jobs) {
        // Check if there's already a run for this job in the last hour
        const [recent] = await sql`
            SELECT id FROM runs
            WHERE job_id = ${job.id}
            AND started_at > NOW() - INTERVAL '1 hour'
            LIMIT 1
        `

        if (recent) {
            // Already ran recently, skip
            continue
        }

        // Get the user_id from the job
        const [jobRow] = await sql<{ user_id: string }[]>`SELECT user_id FROM jobs WHERE id = ${job.id}`
        if (!jobRow) continue

        console.log(`[runner] Creating scheduled run for job "${job.name}"`)
        await sql`
            INSERT INTO runs (user_id, job_id, status, terminal_output, started_at, finished_at)
            VALUES (${jobRow.user_id}, ${job.id}, 'Running', NULL, NOW(), NULL)
        `
    }
}