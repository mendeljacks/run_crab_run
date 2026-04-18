import { execSync } from 'child_process'
import { readFileSync } from 'fs'
import pc from 'picocolors'

// Load .env into process.env so build args are available
const envFile = readFileSync('.env', 'utf-8')
for (const line of envFile.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue
    const eq = trimmed.indexOf('=')
    if (eq === -1) continue
    const key = trimmed.slice(0, eq).trim()
    const val = trimmed.slice(eq + 1).trim()
    if (!(key in process.env)) process.env[key] = val
}

const run = (cmd: string, cwd?: string) => {
    console.log(pc.cyan(`> ${cmd}`))
    execSync(cmd, { stdio: 'inherit', cwd: cwd || process.cwd() })
}

const ROOT = process.cwd().replace('/infrastructure', '')
const REGISTRY = 'registry.no3rd.com'
const NAMESPACE = 'run-crab-run'
const TAG = new Date().toISOString().replace(/[-:T]/g, '').slice(0, 14) // e.g. 20260319120530

const step = (name: string, fn: () => void) => {
    console.log(pc.bold(pc.blue(`\n=== ${name} ===\n`)))
    fn()
    console.log(pc.green(`Done: ${name}`))
}

export const deploy = async () => {
    try {
        step('Create namespace', () => {
            try {
                run(`kubectl create namespace ${NAMESPACE}`)
            } catch {
                console.log(pc.yellow('Namespace already exists'))
            }
        })

        step('Create K8s secrets from .env', () => {
            try {
                run(`kubectl delete secret rcr-env -n ${NAMESPACE}`)
            } catch {}
            run(`kubectl create secret generic rcr-env --from-env-file=.env -n ${NAMESPACE}`)
        })

        step('Run database migrations', () => {
            console.log(pc.yellow('Run migrations manually via kubectl exec or Studio SQL editor'))
            console.log(pc.white('  Migration files are in infrastructure/migrations/'))
        })

        step('Install frontend dependencies', () => {
            run('pnpm install --frozen-lockfile', `${ROOT}/frontend`)
        })

        step('Build and push frontend Docker image', () => {
            const img = `${REGISTRY}/run-crab-run-frontend`
            run(
                `docker build -f frontend/Dockerfile ` +
                    `--build-arg VITE_SUPABASE_URL=https://run-crab-run.example.com/supabase ` +
                    `--build-arg VITE_SUPABASE_ANON_KEY=${process.env.ANON_KEY || 'missing'} ` +
                    `-t ${img}:${TAG} -t ${img}:latest .`,
                ROOT
            )
            run(`docker push ${img}:${TAG}`)
            run(`docker push ${img}:latest`)
        })

        step('Install backend dependencies', () => {
            run('pnpm install --frozen-lockfile', `${ROOT}/backend`)
        })

        step('Build and push backend Docker image', () => {
            const img = `${REGISTRY}/run-crab-run-backend`
            run(`docker build -f backend/Dockerfile ` + `-t ${img}:${TAG} -t ${img}:latest .`, ROOT)
            run(`docker push ${img}:${TAG}`)
            run(`docker push ${img}:latest`)
        })

        step('Deploy Supabase', () => {
            run(`kubectl apply -f k8s_supabase.yaml`)
        })

        step('Deploy Backend', () => {
            run(`kubectl apply -f k8s_backend.yaml`)
            run(
                `kubectl set image deployment/backend backend=${REGISTRY}/run-crab-run-backend:${TAG} -n ${NAMESPACE}`
            )
        })

        step('Deploy Frontend', () => {
            run(`kubectl apply -f k8s_frontend.yaml`)
            run(
                `kubectl set image deployment/frontend frontend=${REGISTRY}/run-crab-run-frontend:${TAG} -n ${NAMESPACE}`
            )
        })

        step('Deploy Ingress', () => {
            run(`kubectl apply -f k8s_ingress.yaml`)
        })

        console.log(pc.bold(pc.green('\n=== Deployment complete! ===')))
        console.log(pc.white(`Frontend:      https://run-crab-run.example.com`))
        console.log(pc.white(`Backend API:   https://run-crab-run.example.com/api`))
        console.log(pc.white(`Supabase Auth: https://run-crab-run.example.com/supabase/auth/v1`))
        console.log(pc.white(`Supabase Studio: https://studio.run-crab-run.example.com`))
    } catch (err) {
        console.error(pc.red('Deployment failed:'), err)
        process.exit(1)
    }
}

deploy()