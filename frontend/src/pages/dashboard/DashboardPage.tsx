import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { Box, Typography, Paper, CircularProgress, Chip } from '@mui/material'
import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { fetchJobs } from '../jobs/helpers'
import { fetchRuns } from '../runs/helpers'
import { statusColor, statusBgColor, formatTimeAgo, shortId } from '../../helpers/format'
import type { Run, RunStatus } from '../../stores/types'

export const DashboardPage = observer(() => {
    useEffect(() => {
        fetchJobs()
        fetchRuns()
    }, [])

    const jobs = store.jobs.list
    const runs = store.runs.list
    const loading = store.jobs.loading || store.runs.loading

    const runningCount = runs.filter(r => r.status === 'Running').length
    const succeededCount = runs.filter(r => r.status === 'Succeeded').length
    const failedCount = runs.filter(r => r.status === 'Failed').length

    if (loading) {
        return (
            <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                <CircularProgress />
            </Box>
        )
    }

    return (
        <Box>
            <Typography variant="h5" sx={{ fontWeight: 700, mb: 3 }}>Dashboard</Typography>

            <Box sx={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', gap: 2, mb: 4 }}>
                <StatCard label="Jobs" value={jobs.length} />
                <StatCard label="Running" value={runningCount} color="#0ea5e9" />
                <StatCard label="Succeeded" value={succeededCount} color="#10b981" />
                <StatCard label="Failed" value={failedCount} color="#ef4444" />
            </Box>

            <Typography variant="h6" sx={{ fontWeight: 600, mb: 2 }}>Jobs</Typography>
            {jobs.length === 0 ? (
                <Paper sx={{ p: 3, borderRadius: 2, border: '1px solid #e2e8f0', textAlign: 'center' }}>
                    <Typography color="text.secondary">No jobs yet. Create one!</Typography>
                </Paper>
            ) : (
                <Box sx={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: 2, mb: 4 }}>
                    {jobs.map(job => (
                        <JobCard key={job.id} job={job} runs={runs.filter(r => r.job_id === job.id)} />
                    ))}
                </Box>
            )}

            <Typography variant="h6" sx={{ fontWeight: 600, mb: 2 }}>Recent Runs</Typography>
            {runs.length === 0 ? (
                <Paper sx={{ p: 3, borderRadius: 2, border: '1px solid #e2e8f0', textAlign: 'center' }}>
                    <Typography color="text.secondary">No runs yet.</Typography>
                </Paper>
            ) : (
                <Paper sx={{ borderRadius: 2, border: '1px solid #e2e8f0', overflow: 'hidden' }}>
                    <RecentRunsTable runs={runs.slice(0, 10)} jobs={jobs} />
                </Paper>
            )}
        </Box>
    )
})

const StatCard = ({ label, value, color }: { label: string; value: number; color?: string }) => (
    <Paper sx={{ p: 2, borderRadius: 2, border: '1px solid #e2e8f0' }}>
        <Typography variant="h4" sx={{ fontWeight: 700, color: color ?? '#0f172a' }}>{value}</Typography>
        <Typography variant="caption" color="text.secondary">{label}</Typography>
    </Paper>
)

const JobCard = observer(({ job, runs }: { job: { id: string; name: string; command: string; schedule: string | null; enabled: boolean }; runs: Run[] }) => {
    const runningCount = runs.filter(r => r.status === 'Running').length
    return (
        <Paper sx={{ p: 2, borderRadius: 2, border: '1px solid #e2e8f0' }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1 }}>
                <Typography variant="subtitle1" sx={{ fontWeight: 600 }}>{job.name}</Typography>
                <Chip
                    label={job.enabled ? 'enabled' : 'disabled'}
                    size="small"
                    sx={{ height: 24, fontSize: '0.7rem', fontWeight: 600, bgcolor: job.enabled ? '#d1fae5' : '#f1f5f9', color: job.enabled ? '#059669' : '#64748b' }}
                />
            </Box>
            <Typography variant="body2" sx={{ fontFamily: 'monospace', color: '#475569', mb: 1 }}>
                {job.command}
            </Typography>
            <Typography variant="caption" color="text.secondary">
                {job.schedule ?? '⚡ Manual'} • {runningCount > 0 ? `${runningCount} running, ` : ''}{runs.length} total
            </Typography>
        </Paper>
    )
})

const RecentRunsTable = observer(({ runs, jobs }: { runs: Run[]; jobs: { id: string; name: string }[] }) => {
    const jobNameMap = Object.fromEntries(jobs.map(j => [j.id, j.name]))

    return (
        <Box component="table" sx={{ width: '100%', borderCollapse: 'collapse' }}>
            <Box component="thead" sx={{ bgcolor: '#f8fafc' }}>
                <tr>
                    {['ID', 'Job', 'Status', 'Started'].map(h => (
                        <Box key={h} component="th" sx={{ p: 1.5, textAlign: 'left', fontSize: '0.75rem', fontWeight: 600, color: '#64748b', borderBottom: '1px solid #e2e8f0' }}>
                            {h}
                        </Box>
                    ))}
                </tr>
            </Box>
            <Box component="tbody">
                {runs.map(run => (
                    <tr key={run.id} style={{ borderBottom: '1px solid #f1f5f9', cursor: 'pointer' }} onClick={() => { runInAction(() => { store.ui.runDetailId = run.id; store.ui.page = 'run-detail' }) }}>
                        <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem', fontFamily: 'monospace' }}>{shortId(run.id)}</Box>
                        <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem' }}>{jobNameMap[run.job_id] ?? shortId(run.job_id)}</Box>
                        <Box component="td" sx={{ p: 1.5 }}>
                            <Chip label={run.status} size="small" sx={{ height: 24, fontSize: '0.7rem', fontWeight: 600, bgcolor: statusBgColor(run.status), color: statusColor(run.status) }} />
                        </Box>
                        <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem', color: '#64748b' }}>{formatTimeAgo(run.started_at)}</Box>
                    </tr>
                ))}
            </Box>
        </Box>
    )
})