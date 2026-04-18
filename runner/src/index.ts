import { processPendingRuns, scheduleDueJobs } from './runner.js'

const POLL_INTERVAL_MS = 2000 // 2 seconds

console.log('[runner] Run Crab Run runner starting...')

async function main(): Promise<void> {
    // Initial run
    await processPendingRuns()
    await scheduleDueJobs()

    // Poll loop
    setInterval(async () => {
        try {
            await processPendingRuns()
        } catch (err) {
            console.error('[runner] Error processing pending runs:', err)
        }

        try {
            await scheduleDueJobs()
        } catch (err) {
            console.error('[runner] Error scheduling due jobs:', err)
        }
    }, POLL_INTERVAL_MS)

    console.log(`[runner] Polling every ${POLL_INTERVAL_MS / 1000}s`)
}

main().catch((err) => {
    console.error('[runner] Fatal error:', err)
    process.exit(1)
})