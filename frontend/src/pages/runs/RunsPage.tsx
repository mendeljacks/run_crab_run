import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { Box, Typography, Paper, CircularProgress, Chip, IconButton, Select, MenuItem, FormControl, InputLabel, TextField, InputAdornment } from '@mui/material'
import RefreshIcon from '@mui/icons-material/Refresh'
import SearchIcon from '@mui/icons-material/Search'
import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { fetchRuns } from '../runs/helpers'
import { fetchJobs } from '../jobs/helpers'
import { shortId, formatTimeAgo, statusColor, statusBgColor } from '../../helpers/format'
import type { Run } from '../../stores/types'
import { useState } from 'react'

export const RunsPage = observer(() => {
    const [statusFilter, setStatusFilter] = useState<string>('')
    const [search, setSearch] = useState('')

    useEffect(() => {
        fetchRuns()
        fetchJobs()
    }, [])

    const runs = store.runs.list
    const jobs = store.jobs.list
    const loading = store.runs.loading
    const jobNameMap = Object.fromEntries(jobs.map(j => [j.id, j.name]))

    const filtered = runs.filter(run => {
        if (statusFilter && run.status !== statusFilter) return false
        if (search) {
            const jobName = jobNameMap[run.job_id] ?? ''
            const q = search.toLowerCase()
            return run.id.toLowerCase().includes(q) || jobName.toLowerCase().includes(q)
        }
        return true
    })

    if (loading && runs.length === 0) {
        return (
            <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                <CircularProgress />
            </Box>
        )
    }

    return (
        <Box>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
                <Typography variant="h5" sx={{ fontWeight: 700 }}>Runs</Typography>
                <IconButton onClick={() => { fetchRuns(); fetchJobs() }} size="small">
                    <RefreshIcon fontSize="small" />
                </IconButton>
            </Box>

            <Box sx={{ display: 'flex', gap: 2, mb: 2 }}>
                <TextField
                    size="small"
                    placeholder="Search by job name or ID…"
                    value={search}
                    onChange={e => setSearch(e.target.value)}
                    slotProps={{
                        input: {
                            startAdornment: (
                                <InputAdornment position="start"><SearchIcon fontSize="small" /></InputAdornment>
                            )
                        }
                    }}
                    sx={{ minWidth: 260 }}
                />
                <FormControl size="small" sx={{ minWidth: 140 }}>
                    <InputLabel>Status</InputLabel>
                    <Select
                        label="Status"
                        value={statusFilter}
                        onChange={e => setStatusFilter(e.target.value)}
                    >
                        <MenuItem value="">All</MenuItem>
                        <MenuItem value="Running">Running</MenuItem>
                        <MenuItem value="Succeeded">Succeeded</MenuItem>
                        <MenuItem value="Failed">Failed</MenuItem>
                    </Select>
                </FormControl>
            </Box>

            <Typography variant="caption" color="text.secondary" sx={{ mb: 1, display: 'block' }}>
                {filtered.length} run{filtered.length !== 1 ? 's' : ''} found
            </Typography>

            {filtered.length === 0 ? (
                <Paper sx={{ p: 4, borderRadius: 2, border: '1px solid #e2e8f0', textAlign: 'center' }}>
                    <Typography color="text.secondary">No runs found.</Typography>
                </Paper>
            ) : (
                <Paper sx={{ borderRadius: 2, border: '1px solid #e2e8f0', overflow: 'hidden' }}>
                    <RunsTable runs={filtered} jobNameMap={jobNameMap} />
                </Paper>
            )}
        </Box>
    )
})

const RunsTable = observer(({ runs, jobNameMap }: { runs: Run[]; jobNameMap: Record<string, string> }) => {
    return (
        <Box component="table" sx={{ width: '100%', borderCollapse: 'collapse' }}>
            <Box component="thead" sx={{ bgcolor: '#f8fafc' }}>
                <tr>
                    {['ID', 'Job', 'Status', 'Started', 'Finished'].map(h => (
                        <Box
                            key={h}
                            component="th"
                            sx={{ p: 1.5, textAlign: 'left', fontSize: '0.75rem', fontWeight: 600, color: '#64748b', borderBottom: '1px solid #e2e8f0' }}
                        >
                            {h}
                        </Box>
                    ))}
                </tr>
            </Box>
            <Box component="tbody">
                {runs.map(run => (
                    <tr
                        key={run.id}
                        style={{ borderBottom: '1px solid #f1f5f9', cursor: 'pointer' }}
                        onClick={() => { runInAction(() => { store.ui.runDetailId = run.id; store.ui.page = 'run-detail' }) }}
                    >
                        <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem', fontFamily: 'monospace' }}>
                            {shortId(run.id)}
                        </Box>
                        <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem' }}>
                            {jobNameMap[run.job_id] ?? shortId(run.job_id)}
                        </Box>
                        <Box component="td" sx={{ p: 1.5 }}>
                            <Chip
                                label={run.status}
                                size="small"
                                sx={{
                                    height: 24,
                                    fontSize: '0.7rem',
                                    fontWeight: 600,
                                    bgcolor: statusBgColor(run.status),
                                    color: statusColor(run.status)
                                }}
                            />
                        </Box>
                        <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem', color: '#64748b' }}>
                            {formatTimeAgo(run.started_at)}
                        </Box>
                        <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem', color: '#64748b' }}>
                            {run.finished_at ? formatTimeAgo(run.finished_at) : (run.status === 'Running' ? '—' : '—')}
                        </Box>
                    </tr>
                ))}
            </Box>
        </Box>
    )
})