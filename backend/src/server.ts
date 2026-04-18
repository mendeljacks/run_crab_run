import express from 'express'
import cors from 'cors'
import jobsRoutes from './routes/jobs.js'
import runsRoutes from './routes/runs.js'

const app = express()
const PORT = parseInt(process.env.PORT || '4000')

app.use(cors())
app.use(express.json())

app.get('/health', (_req, res) => {
    res.json({ status: 'ok' })
})

app.use('/api', jobsRoutes)
app.use('/api', runsRoutes)

app.listen(PORT, () => {
    console.log(`Run Crab Run backend listening on port ${PORT}`)
})