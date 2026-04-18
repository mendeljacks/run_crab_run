import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'
import { CssBaseline, Box, AppBar, Toolbar, Typography, Button, CircularProgress } from '@mui/material'
import { runInAction } from 'mobx'
import { store } from './stores/root_store'
import { authenticate } from './pages/login/helpers'
import { LoginPage } from './pages/login/LoginPage'
import { DashboardPage } from './pages/dashboard/DashboardPage'
import { JobsPage } from './pages/jobs/JobsPage'
import { RunsPage } from './pages/runs/RunsPage'
import { RunDetailPage } from './pages/runs/RunDetailPage'

const NAV_ITEMS = [
    { key: 'dashboard', label: 'Dashboard' },
    { key: 'jobs', label: 'Jobs' },
    { key: 'runs', label: 'Runs' }
] as const

const navigateTo = (page: string) => {
    runInAction(() => { store.ui.page = page })
}

const App = observer(() => {
    useEffect(() => {
        authenticate()
    }, [])

    if (store.auth.loading) {
        return (
            <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', minHeight: '100vh' }}>
                <CircularProgress />
            </Box>
        )
    }

    if (!store.auth.token) {
        return (
            <>
                <CssBaseline />
                <LoginPage />
            </>
        )
    }

    return (
        <>
            <CssBaseline />
            <AppBar position="static" elevation={0} sx={{ bgcolor: '#0f172a' }}>
                <Toolbar>
                    <Typography
                        variant="h6"
                        sx={{ fontWeight: 700, mr: 4, cursor: 'pointer' }}
                        onClick={() => navigateTo('dashboard')}
                    >
                        🦀 Run Crab Run
                    </Typography>
                    {NAV_ITEMS.map(item => (
                        <Button
                            key={item.key}
                            onClick={() => navigateTo(item.key)}
                            sx={{
                                textTransform: 'none',
                                color: store.ui.page === item.key ? '#fff' : '#94a3b8',
                                fontWeight: store.ui.page === item.key ? 600 : 400,
                                borderBottom: store.ui.page === item.key ? '2px solid #0ea5e9' : '2px solid transparent',
                                borderRadius: 0,
                                px: 2
                            }}
                        >
                            {item.label}
                        </Button>
                    ))}
                </Toolbar>
            </AppBar>
            <Box sx={{ p: 3 }}>
                {store.ui.page === 'dashboard' && <DashboardPage />}
                {store.ui.page === 'jobs' && <JobsPage />}
                {store.ui.page === 'runs' && <RunsPage />}
                {store.ui.page === 'run-detail' && <RunDetailPage />}
            </Box>
        </>
    )
})

export default App