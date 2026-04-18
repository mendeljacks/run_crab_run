# How to Run Locally

## Prerequisites
- [Docker](https://docs.docker.com/get-docker/) (for Supabase local stack)
- Node.js 22+
- pnpm (recommended) or npm

## 1. Start Local Supabase

The Supabase CLI runs Postgres, GoTrue (auth), Studio, and all other Supabase services in Docker containers:

```bash
# Install Supabase CLI (if not installed)
npx supabase init   # already done — only needed once
npx supabase start  # starts all services (~1-2 min first time for Docker pulls)
```

After `supabase start` completes, it prints connection details. You need these:

| Service | URL |
|---------|-----|
| **API / Auth** | `http://127.0.0.1:54321` |
| **Database** | `postgresql://postgres:postgres@127.0.0.1:54322/postgres` |
| **Studio** | `http://127.0.0.1:54323` |
| **JWT Secret** | `super-secret-jwt-token-with-at-least-32-characters-long` |
| **Anon Key** | `eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0` |

> Run `npx supabase status` anytime to see these values.

## 2. Configure Backend

Create `backend/.env` (already gitignored):

```bash
PORT=4000
DATABASE_URL=postgresql://postgres:postgres@127.0.0.1:54322/postgres
JWT_SECRET=super-secret-jwt-token-with-at-least-32-characters-long
SUPABASE_URL=http://127.0.0.1:54321
```

> `SUPABASE_URL` enables JWKS-based JWT verification (ES256) for local dev.
> In production (K8s), only `JWT_SECRET` is needed for HS256 verification.

## 3. Start Backend

```bash
cd backend
npm install
npm run dev
# → Run Crab Run backend listening on port 4000
```

Verify: `curl http://localhost:4000/health` → `{"status":"ok"}`

## 4. Start the Runner

The runner watches for pending runs and executes job commands:

```bash
cd runner
npm install
npm run dev
# → [runner] Polling every 2s
```

When you click "Run" on a job in the UI, it creates a run with `status='Running'`.
The runner picks it up, executes the command, and updates the run with
`status='Succeeded'`/`'Failed'`, `terminal_output`, and `finished_at`.

## 5. Start Frontend

The frontend `.env.development` is already configured for local Supabase:

```bash
cd frontend
npm install
npm run dev
# → Vite dev server on http://localhost:5173
```

The Vite dev server proxies `/api` requests to `http://localhost:4000` (the backend).
The Supabase client connects directly to `http://127.0.0.1:54321` (local Supabase auth).

## 6. Use the App

1. Open http://localhost:5173
2. Click **"Sign In Anonymously"** — Supabase creates an anonymous session
3. Create jobs, trigger runs, etc. — all data stored in local Postgres

## Stopping

```bash
npx supabase stop          # Stop Supabase containers (data preserved)
npx supabase stop --no-backup  # Stop + delete all data
```

## Resetting the Database

If you need a fresh start:

```bash
npx supabase db reset      # Re-runs all migrations
```

## Architecture (Local Dev)

```
┌────────────────────────────────────────────────────────────┐
│  Frontend (Vite :5173)                                    │
│  ├─ @supabase/supabase-js → http://127.0.0.1:54321  (auth)│
│  └─ fetch /api/*          → http://localhost:4000    (data)│
└────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼                               ▼
┌──────────────────────┐     ┌──────────────────────────┐
│ Backend (Express :4000)│    │ Supabase Auth (GoTrue)   │
│ postgres.js → DB      │    │ :54321                   │
│ jose JWKS → Auth      │    │ Signs JWTs (ES256)       │
└──────────┬───────────┘    └──────────────────────────┘
           │
           ▼
┌──────────────────────┐     ┌──────────────────────────┐
│ Postgres (:54322)     │     │ Runner (headless)         │
│ jobs + runs tables    │◄────│ Polls for pending runs    │
│ RLS enabled           │     │ Executes commands          │
└──────────────────────┘     │ Updates status + output    │
                              └──────────────────────────┘
```