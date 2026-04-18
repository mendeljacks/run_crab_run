import { Router } from 'express'
import { authMiddleware, type AuthenticatedRequest } from '../auth.js'
import { sql } from '../db.js'

const router: Router = Router()

// GET /api/runs
router.get('/runs', authMiddleware, async (req, res) => {
    const { userId } = req as AuthenticatedRequest
    const { job_id } = req.query as { job_id?: string }

    try {
        const runs = job_id
            ? await sql`
                SELECT id, job_id, terminal_output, status, started_at, finished_at, created_at, updated_at
                FROM runs
                WHERE user_id = ${userId} AND job_id = ${job_id}
                ORDER BY started_at DESC
            `
            : await sql`
                SELECT id, job_id, terminal_output, status, started_at, finished_at, created_at, updated_at
                FROM runs
                WHERE user_id = ${userId}
                ORDER BY started_at DESC
            `
        res.json({ runs })
    } catch (err) {
        console.error('List runs error:', err)
        res.status(500).json({ error: 'Failed to list runs' })
    }
})

// GET /api/runs/:id
router.get('/runs/:id', authMiddleware, async (req, res) => {
    const { userId } = req as AuthenticatedRequest
    const { id } = req.params

    try {
        const [run] = await sql`
            SELECT id, job_id, terminal_output, status, started_at, finished_at, created_at, updated_at
            FROM runs
            WHERE id = ${id} AND user_id = ${userId}
        `
        if (!run) {
            res.status(404).json({ error: 'Run not found' })
            return
        }
        res.json({ run })
    } catch (err) {
        console.error('Get run error:', err)
        res.status(500).json({ error: 'Failed to get run' })
    }
})

// POST /api/runs
router.post('/runs', authMiddleware, async (req, res) => {
    const { userId } = req as AuthenticatedRequest
    const { job_id, status = 'Running', terminal_output = null, started_at, finished_at = null } = req.body

    if (!job_id) {
        res.status(400).json({ error: 'job_id is required' })
        return
    }

    try {
        // Verify job belongs to user
        const [job] = await sql`
            SELECT id FROM jobs WHERE id = ${job_id} AND user_id = ${userId}
        `
        if (!job) {
            res.status(404).json({ error: 'Job not found' })
            return
        }

        const [run] = await sql`
            INSERT INTO runs ${sql({
                user_id: userId,
                job_id,
                status,
                terminal_output,
                started_at: started_at || new Date(),
                finished_at
            })}
            RETURNING id, job_id, terminal_output, status, started_at, finished_at, created_at, updated_at
        `
        res.status(201).json({ run })
    } catch (err) {
        console.error('Create run error:', err)
        res.status(500).json({ error: 'Failed to create run' })
    }
})

// PATCH /api/runs/:id
router.patch('/runs/:id', authMiddleware, async (req, res) => {
    const { userId } = req as AuthenticatedRequest
    const { id } = req.params
    const updates = req.body

    try {
        // Verify ownership
        const [existing] = await sql`
            SELECT id FROM runs WHERE id = ${id} AND user_id = ${userId}
        `
        if (!existing) {
            res.status(404).json({ error: 'Run not found' })
            return
        }

        const setClauses: string[] = []
        const values: any[] = []
        let paramIndex = 1

        if (updates.terminal_output !== undefined) {
            setClauses.push(`terminal_output = $${paramIndex++}`)
            values.push(updates.terminal_output)
        }
        if (updates.status !== undefined) {
            setClauses.push(`status = $${paramIndex++}`)
            values.push(updates.status)
        }
        if (updates.finished_at !== undefined) {
            setClauses.push(`finished_at = $${paramIndex++}`)
            values.push(updates.finished_at)
        }

        if (setClauses.length === 0) {
            res.status(400).json({ error: 'No fields to update' })
            return
        }

        setClauses.push(`updated_at = NOW()`)
        values.push(id)
        values.push(userId)

        const query = `
            UPDATE runs SET ${setClauses.join(', ')}
            WHERE id = $${paramIndex++} AND user_id = $${paramIndex++}
            RETURNING id, job_id, terminal_output, status, started_at, finished_at, created_at, updated_at
        `

        const runs = await sql.unsafe(query, values)
        if (runs.length === 0) {
            res.status(404).json({ error: 'Run not found' })
            return
        }
        res.json({ run: runs[0] })
    } catch (err) {
        console.error('Update run error:', err)
        res.status(500).json({ error: 'Failed to update run' })
    }
})

// DELETE /api/runs/:id
router.delete('/runs/:id', authMiddleware, async (req, res) => {
    const { userId } = req as AuthenticatedRequest
    const { id } = req.params

    try {
        const [deleted] = await sql`
            DELETE FROM runs WHERE id = ${id} AND user_id = ${userId}
            RETURNING id
        `
        if (!deleted) {
            res.status(404).json({ error: 'Run not found' })
            return
        }
        res.json({ deleted: true })
    } catch (err) {
        console.error('Delete run error:', err)
        res.status(500).json({ error: 'Failed to delete run' })
    }
})

export default router