import { observer } from 'mobx-react-lite'
import { Box, Button, Typography, Paper, CircularProgress } from '@mui/material'
import { store } from '../../stores/root_store'
import { requestNewToken } from './helpers'

export const LoginPage = observer(() => {
    return (
        <Box sx={{
            minHeight: '100vh',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            background: 'linear-gradient(135deg, #0f172a 0%, #1e293b 50%, #0f172a 100%)'
        }}>
            <Paper sx={{ p: 6, borderRadius: 4, maxWidth: 420, width: '100%', textAlign: 'center' }}>
                <Typography variant="h4" sx={{ fontWeight: 700, color: '#0f172a', mb: 0.5 }}>
                    🦀 Run Crab Run
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mb: 4 }}>
                    Job runner dashboard powered by SpacetimeDB
                </Typography>

                {store.auth.loading ? (
                    <CircularProgress />
                ) : (
                    <Button
                        variant="contained"
                        size="large"
                        onClick={() => requestNewToken()}
                        sx={{ textTransform: 'none', borderRadius: 2, px: 4 }}
                    >
                        Connect to SpacetimeDB
                    </Button>
                )}

                {store.auth.token === '' && !store.auth.loading && (
                    <Typography variant="caption" color="error" sx={{ mt: 2, display: 'block' }}>
                        Not connected. Click above to authenticate.
                    </Typography>
                )}
            </Paper>
        </Box>
    )
})