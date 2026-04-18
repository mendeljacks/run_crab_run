import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { Box, Typography, Paper, Chip, Button, CircularProgress } from '@mui/material'
import ArrowBackIcon from '@mui/icons-material/ArrowBack'
import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { fetchRuns, fetchRunById } from '../runs/helpers'
import { fetchJobs } from '../jobs/helpers'
import { shortId, formatDatetime, formatTimeAgo, formatElapsed, statusColor, statusBgColor } from '../../helpers/format'

export const RunDetailPage = observer(() => {
    const runId = store.ui.runDetailId

    useEffect(() => {
        fetchRunById(runId)
        fetchJobs()
    }, [runId])

    const run = store.runs.list.find(r => r.id === runId)

    if (!run || store.runs.loading) {
        return (
            <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                <CircularProgress />
            </Box>
        )
    }

    const jobName = store.jobs.list.find(j => j.id === run.job_id)?.name ?? shortId(run.job_id)
    const durationMicros = run.finished_at ? run.finished_at - run.started_at : null
    const durationStr = durationMicros != null ? formatElapsed(durationMicros) : '—'

    return (
        <Box>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                    <Button startIcon={<ArrowBackIcon />} onClick={() => { runInAction(() => { store.ui.page = 'runs' }) }} sx={{ textTransform: 'none' }}>
                        Back
                    </Button>
                    <Typography variant="h5" sx={{ fontWeight: 700 }}>
                        Run {shortId(run.id)}
                    </Typography>
                </Box>
                <Chip label={run.status} sx={{ height: 28, fontSize: '0.8rem', fontWeight: 600, bgcolor: statusBgColor(run.status), color: statusColor(run.status) }} />
            </Box>

            <Box sx={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))', gap: 2, mb: 3 }}>
                <DetailCard label="Job" value={jobName} />
                <DetailCard label="Status" value={run.status} valueColor={statusColor(run.status)} />
                <DetailCard label="Duration" value={durationStr} />
                <DetailCard label="Started" value={formatDatetime(run.started_at)} />
                <DetailCard label="Finished" value={run.finished_at ? formatDatetime(run.finished_at) : '—'} />
            </Box>

            {run.terminal_output && (
                <Box sx={{ mt: 3 }}>
                    <Typography variant="subtitle1" sx={{ fontWeight: 600, mb: 1 }}>Terminal Output</Typography>
                    <Paper sx={{ p: 2, borderRadius: 2, border: '1px solid #e2e8f0', bgcolor: '#0f172a', maxHeight: 400, overflow: 'auto' }}>
                        <pre style={{ margin: 0, color: '#e2e8f0', fontFamily: 'monospace', fontSize: '0.85rem', whiteSpace: 'pre-wrap', wordBreak: 'break-word' }}>
                            {run.terminal_output}
                        </pre>
                    </Paper>
                </Box>
            )}
        </Box>
    )
})

const DetailCard = ({ label, value, valueColor }: { label: string; value: string; valueColor?: string }) => (
    <Paper sx={{ p: 2, borderRadius: 2, border: '1px solid #e2e8f0' }}>
        <Typography variant="caption" color="text.secondary" sx={{ mb: 0.5, display: 'block' }}>{label}</Typography>
        <Typography variant="body1" sx={{ fontWeight: 600, color: valueColor ?? '#0f172a' }}>{value}</Typography>
    </Paper>
)