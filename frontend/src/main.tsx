import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { ThemeProvider, createTheme } from '@mui/material'
import App from './App'

const theme = createTheme({
    palette: {
        primary: {
            main: '#0f172a',
            light: '#1e293b',
            dark: '#020617',
            contrastText: '#f8fafc'
        },
        secondary: {
            main: '#64748b',
            light: '#94a3b8',
            dark: '#475569',
            contrastText: '#f8fafc'
        },
        background: {
            default: '#f1f5f9',
            paper: '#ffffff'
        }
    },
    typography: {
        fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif'
    },
    components: {
        MuiButton: {
            defaultProps: {
                disableElevation: true
            }
        },
        MuiPaper: {
            defaultProps: {
                elevation: 0
            }
        }
    }
})

createRoot(document.getElementById('root')!).render(
    <StrictMode>
        <ThemeProvider theme={theme}>
            <App />
        </ThemeProvider>
    </StrictMode>
)