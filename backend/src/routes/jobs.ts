import { Router } from 'express'
import { authMiddleware, type AuthenticatedRequest } from '../auth.js'
import { sql } from '../db.js'

const router: Router = Router()

// GET /api/jobs
router.get('/jobs', authMiddleware, async (req, res) => {
    const { userId } = req as AuthenticatedRequest
    try {
        const jobs = await sql`
            SELECT id, name, command, schedule, enabled, created_at, updated_at
            FROM jobs
            WHERE user_id = ${userId}
            ORDER BY created_at DESC
        `
        res.json({ jobs })
    } catch (err) {
        console.error('List jobs error:', err)
        res.status(500).json({ error: 'Failed to list jobs' })
    }
})

// POST /api/jobs
router.post('/jobs', authMiddleware, async (req, res) => {
    const { userId } = req as AuthenticatedRequest
    const { name, command, schedule = null, enabled = true } = req.body

    if (!name || !command) {
        res.status(400).json({ error: 'name and command are required' })
        return
    }

    try {
        const [job] = await sql`
            INSERT INTO jobs ${sql({ user_id: userId, name, command, schedule, enabled })}
            RETURNING id, name, command, schedule, enabled, created_at, updated_at
        `
        res.status(201).json({ job })
    } catch (err) {
        console.error('Create job error:', err)
        res.status(500).json({ error: 'Failed to create job' })
    }
})

// PATCH /api/jobs/:id
router.patch('/jobs/:id', authMiddleware, async (req, res) => {
    const { userId } = req as AuthenticatedRequest
    const { id } = req.params
    const updates = req.body

    try {
        // Verify ownership
        const [existing] = await sql`
            SELECT id FROM jobs WHERE id = ${id} AND user_id = ${userId}
        `
        if (!existing) {
            res.status(404).json({ error: 'Job not found' })
            return
        }

        const setClauses: string[] = []
        const values: any[] = []
        let paramIndex = 1

        if (updates.name !== undefined) {
            setClauses.push(`name = $${paramIndex++}`)
            values.push(updates.name)
        }
        if (updates.command !== undefined) {
            setClauses.push(`command = $${paramIndex++}`)
            values.push(updates.command)
        }
        if (updates.schedule !== undefined) {
            setClauses.push(`schedule = $${paramIndex++}`)
            values.push(updates.schedule) // null clears the schedule
        }
        if (updates.enabled !== undefined) {
            setClauses.push(`enabled = $${paramIndex++}`)
            values.push(updates.enabled)
        }

        if (setClauses.length === 0) {
            res.status(400).json({ error: 'No fields to update' })
            return
        }

        setClauses.push(`updated_at = NOW()`)
        values.push(id)
        values.push(userId)

        // Use unparameterized query with sanitized inputs since postgres.js tagged templates don't easily support dynamic SET clauses
        const query = `
            UPDATE jobs SET ${setClauses.join(', ')}
            WHERE id = $${paramIndex++} AND user_id = $${paramIndex++}
            RETURNING id, name, command, schedule, enabled, created_at, updated_at
        `

        const jobs = await sql.unsafe(query, values)
        if (jobs.length === 0) {
            res.status(404).json({ error: 'Job not found' })
            return
        }
        res.json({ job: jobs[0] })
    } catch (err) {
        console.error('Update job error:', err)
        res.status(500).json({ error: 'Failed to update job' })
    }
})

// DELETE /api/jobs/:id
router.delete('/jobs/:id', authMiddleware, async (req, res) => {
    const { userId } = req as AuthenticatedRequest
    const { id } = req.params

    try {
        // CASCADE will delete associated runs
        const [deleted] = await sql`
            DELETE FROM jobs WHERE id = ${id} AND user_id = ${userId}
            RETURNING id
        `
        if (!deleted) {
            res.status(404).json({ error: 'Job not found' })
            return
        }
        res.json({ deleted: true })
    } catch (err) {
        console.error('Delete job error:', err)
        res.status(500).json({ error: 'Failed to delete job' })
    }
})

export default router