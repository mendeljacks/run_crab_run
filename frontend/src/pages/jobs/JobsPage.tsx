import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'
import { Box, Typography, Paper, CircularProgress, Button, Chip, Dialog, DialogTitle, DialogContent, DialogActions, TextField, Switch, FormControlLabel, IconButton, Alert, Tooltip } from '@mui/material'
import AddIcon from '@mui/icons-material/Add'
import RefreshIcon from '@mui/icons-material/Refresh'
import PlayArrowIcon from '@mui/icons-material/PlayArrow'
import EditIcon from '@mui/icons-material/Edit'
import DeleteIcon from '@mui/icons-material/Delete'
import ListAltIcon from '@mui/icons-material/ListAlt'
import { runInAction } from 'mobx'
import { store } from '../../stores/root_store'
import { fetchJobs, createJob, updateJob, deleteJob, triggerRun } from './helpers'
import { fetchRuns } from '../runs/helpers'
import { formatSchedule } from '../../helpers/format'
import type { Job } from '../../stores/types'

export const JobsPage = observer(() => {
    useEffect(() => {
        fetchJobs()
        fetchRuns()
    }, [])

    const jobs = store.jobs.list
    const runs = store.runs.list
    const loading = store.jobs.loading && jobs.length === 0

    if (loading) {
        return (
            <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                <CircularProgress />
            </Box>
        )
    }

    return (
        <Box>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
                <Typography variant="h5" sx={{ fontWeight: 700 }}>Jobs</Typography>
                <Box sx={{ display: 'flex', gap: 1 }}>
                    <IconButton onClick={() => { fetchJobs(); fetchRuns() }} size="small">
                        <RefreshIcon fontSize="small" />
                    </IconButton>
                    <Button variant="contained" startIcon={<AddIcon />} onClick={() => { runInAction(() => { store.ui.createJobOpen = true }) }} sx={{ textTransform: 'none', borderRadius: 2 }}>
                        New Job
                    </Button>
                </Box>
            </Box>

            {store.jobs.error && <Alert severity="error" sx={{ mb: 2 }}>{store.jobs.error}</Alert>}

            {jobs.length === 0 ? (
                <Paper sx={{ p: 4, borderRadius: 2, border: '1px solid #e2e8f0', textAlign: 'center' }}>
                    <Typography color="text.secondary">No jobs yet. Create one!</Typography>
                </Paper>
            ) : (
                <Paper sx={{ borderRadius: 2, border: '1px solid #e2e8f0', overflow: 'hidden' }}>
                    <JobsTable jobs={jobs} runs={runs} />
                </Paper>
            )}

            <CreateJobDialog />

            <EditJobDialog />

            <DeleteJobConfirmDialog />
        </Box>
    )
})

const JobsTable = observer(({ jobs, runs }: { jobs: Job[]; runs: { id: string; job_id: string; status: string }[] }) => {
    return (
        <Box component="table" sx={{ width: '100%', borderCollapse: 'collapse' }}>
            <Box component="thead" sx={{ bgcolor: '#f8fafc' }}>
                <tr>
                    {['Name', 'Command', 'Schedule', 'Status', 'Runs', 'Actions'].map(h => (
                        <Box key={h} component="th" sx={{ p: 1.5, textAlign: 'left', fontSize: '0.75rem', fontWeight: 600, color: '#64748b', borderBottom: '1px solid #e2e8f0' }}>
                            {h}
                        </Box>
                    ))}
                </tr>
            </Box>
            <Box component="tbody">
                {jobs.map(job => {
                    const jobRuns = runs.filter(r => r.job_id === job.id)
                    const runningCount = jobRuns.filter(r => r.status === 'Running').length
                    return (
                        <tr key={job.id} style={{ borderBottom: '1px solid #f1f5f9' }}>
                            <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem', fontWeight: 600 }}>{job.name}</Box>
                            <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem', fontFamily: 'monospace', color: '#475569', maxWidth: 200, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                                {job.command}
                            </Box>
                            <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem', color: '#64748b' }}>{formatSchedule(job.schedule)}</Box>
                            <Box component="td" sx={{ p: 1.5 }}>
                                <Chip label={job.enabled ? 'enabled' : 'disabled'} size="small" sx={{ height: 24, fontSize: '0.7rem', fontWeight: 600, bgcolor: job.enabled ? '#d1fae5' : '#f1f5f9', color: job.enabled ? '#059669' : '#64748b' }} />
                            </Box>
                            <Box component="td" sx={{ p: 1.5, fontSize: '0.85rem', color: '#64748b' }}>
                                {runningCount > 0 && <Chip label={`${runningCount} running`} size="small" sx={{ mr: 0.5, height: 24, fontSize: '0.7rem', fontWeight: 600, bgcolor: '#e0f2fe', color: '#0284c7' }} />}
                                {jobRuns.length > 0 ? <span>{jobRuns.length} total</span> : '—'}
                            </Box>
                            <Box component="td" sx={{ p: 1.5 }}>
                                <Box sx={{ display: 'flex', gap: 0.5 }}>
                                    <Tooltip title="Run now">
                                        <IconButton size="small" color="primary" onClick={async () => { await triggerRun(job.id); fetchRuns() }}>
                                            <PlayArrowIcon fontSize="small" />
                                        </IconButton>
                                    </Tooltip>
                                    <Tooltip title="View runs">
                                        <IconButton size="small" onClick={() => {
                                            runInAction(() => {
                                                store.ui.runsSearch = job.name
                                                store.ui.page = 'runs'
                                            })
                                            fetchRuns()
                                        }}>
                                            <ListAltIcon fontSize="small" />
                                        </IconButton>
                                    </Tooltip>
                                    <Tooltip title="Edit">
                                        <IconButton size="small" onClick={() => {
                                            runInAction(() => {
                                                store.ui.editingJob = job
                                                store.ui.editJobOpen = true
                                            })
                                        }}>
                                            <EditIcon fontSize="small" />
                                        </IconButton>
                                    </Tooltip>
                                    <Tooltip title="Delete">
                                        <IconButton size="small" color="error" onClick={() => {
                                            runInAction(() => {
                                                store.ui.deletingJobId = job.id
                                                store.ui.deleteJobOpen = true
                                            })
                                        }}>
                                            <DeleteIcon fontSize="small" />
                                        </IconButton>
                                    </Tooltip>
                                </Box>
                            </Box>
                        </tr>
                    )
                })}
            </Box>
        </Box>
    )
})

const CreateJobDialog = observer(() => {
    const [name, setName] = useState('')
    const [command, setCommand] = useState('')
    const [schedule, setSchedule] = useState('')
    const [enabled, setEnabled] = useState(true)

    const open = store.ui.createJobOpen
    const submitting = store.jobs.loading

    const handleSubmit = async () => {
        if (!name || !command) return
        runInAction(() => { store.ui.createJobOpen = false })
        await createJob(name, command, schedule || null, enabled)
        setName('')
        setCommand('')
        setSchedule('')
        setEnabled(true)
    }

    const handleClose = () => {
        runInAction(() => { store.ui.createJobOpen = false })
        setName('')
        setCommand('')
        setSchedule('')
        setEnabled(true)
    }

    return (
        <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
            <DialogTitle sx={{ fontWeight: 600 }}>Create Job</DialogTitle>
            <DialogContent sx={{ display: 'grid', gap: 2, pt: '8px !important' }}>
                <TextField label="Name" value={name} onChange={e => setName(e.target.value)} size="small" fullWidth required placeholder="e.g., deploy-prod" />
                <TextField label="Command" value={command} onChange={e => setCommand(e.target.value)} size="small" fullWidth required placeholder="e.g., ./deploy.sh" helperText="Shell command to execute" />
                <TextField label="Schedule (RRULE)" value={schedule} onChange={e => setSchedule(e.target.value)} size="small" fullWidth placeholder="FREQ=DAILY;BYHOUR=9" helperText="Leave empty for manual-only" />
                <FormControlLabel control={<Switch checked={enabled} onChange={e => setEnabled(e.target.checked)} />} label="Enabled" />
            </DialogContent>
            <DialogActions sx={{ p: 2 }}>
                <Button onClick={handleClose} sx={{ textTransform: 'none' }}>Cancel</Button>
                <Button variant="contained" onClick={handleSubmit} disabled={!name || !command || submitting} sx={{ textTransform: 'none', borderRadius: 2 }}>
                    Create
                </Button>
            </DialogActions>
        </Dialog>
    )
})

const EditJobDialog = observer(() => {
    const job = store.ui.editingJob as Job | null
    const [name, setName] = useState('')
    const [command, setCommand] = useState('')
    const [schedule, setSchedule] = useState('')
    const [enabled, setEnabled] = useState(true)

    // Sync form when editingJob changes
    useEffect(() => {
        if (job) {
            setName(job.name)
            setCommand(job.command)
            setSchedule(job.schedule ?? '')
            setEnabled(job.enabled)
        }
    }, [job])

    const open = store.ui.editJobOpen

    const handleSubmit = async () => {
        if (!job) return
        const updates: { name?: string; command?: string; schedule?: string | null; enabled?: boolean } = {}
        if (name !== job.name) updates.name = name
        if (command !== job.command) updates.command = command
        if (schedule !== (job.schedule ?? '')) updates.schedule = schedule || null
        if (enabled !== job.enabled) updates.enabled = enabled

        runInAction(() => { store.ui.editJobOpen = false })
        if (Object.keys(updates).length > 0) {
            await updateJob(job.id, updates)
        }
    }

    const handleClose = () => {
        runInAction(() => { store.ui.editJobOpen = false })
    }

    return (
        <Dialog open={open} onClose={handleClose} maxWidth="sm" fullWidth>
            <DialogTitle sx={{ fontWeight: 600 }}>Edit Job</DialogTitle>
            <DialogContent sx={{ display: 'grid', gap: 2, pt: '8px !important' }}>
                <TextField label="Name" value={name} onChange={e => setName(e.target.value)} size="small" fullWidth />
                <TextField label="Command" value={command} onChange={e => setCommand(e.target.value)} size="small" fullWidth />
                <TextField label="Schedule (RRULE)" value={schedule} onChange={e => setSchedule(e.target.value)} size="small" fullWidth placeholder="Leave empty for manual-only" helperText="Clear to remove schedule" />
                <FormControlLabel control={<Switch checked={enabled} onChange={e => setEnabled(e.target.checked)} />} label="Enabled" />
            </DialogContent>
            <DialogActions sx={{ p: 2 }}>
                <Button onClick={handleClose} sx={{ textTransform: 'none' }}>Cancel</Button>
                <Button variant="contained" onClick={handleSubmit} disabled={!name || !command} sx={{ textTransform: 'none', borderRadius: 2 }}>
                    Save
                </Button>
            </DialogActions>
        </Dialog>
    )
})

const DeleteJobConfirmDialog = observer(() => {
    const open = store.ui.deleteJobOpen
    const jobId = store.ui.deletingJobId
    const jobName = store.jobs.list.find(j => j.id === jobId)?.name ?? ''

    const handleDelete = async () => {
        if (!jobId) return
        runInAction(() => { store.ui.deleteJobOpen = false })
        await deleteJob(jobId)
    }

    const handleClose = () => {
        runInAction(() => { store.ui.deleteJobOpen = false })
    }

    return (
        <Dialog open={open} onClose={handleClose} maxWidth="xs" fullWidth>
            <DialogTitle sx={{ fontWeight: 600 }}>Delete Job?</DialogTitle>
            <DialogContent>
                <Typography>Are you sure you want to delete <strong>{jobName}</strong>? All associated runs will also be deleted.</Typography>
            </DialogContent>
            <DialogActions sx={{ p: 2 }}>
                <Button onClick={handleClose} sx={{ textTransform: 'none' }}>Cancel</Button>
                <Button variant="contained" color="error" onClick={handleDelete} sx={{ textTransform: 'none', borderRadius: 2 }}>
                    Delete
                </Button>
            </DialogActions>
        </Dialog>
    )
})