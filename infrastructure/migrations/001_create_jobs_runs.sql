-- Migration: Create jobs and runs tables for Run Crab Run
-- Run against the Supabase Postgres database

CREATE TABLE public.jobs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    command     TEXT NOT NULL,
    schedule    TEXT,
    enabled     BOOLEAN NOT NULL DEFAULT true,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE public.runs (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    job_id           UUID NOT NULL REFERENCES public.jobs(id) ON DELETE CASCADE,
    terminal_output  TEXT,
    status           TEXT NOT NULL DEFAULT 'Running' CHECK (status IN ('Running', 'Succeeded', 'Failed')),
    started_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at      TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes
CREATE INDEX idx_jobs_user ON public.jobs(user_id, created_at DESC);
CREATE INDEX idx_runs_user ON public.runs(user_id, started_at DESC);
CREATE INDEX idx_runs_job ON public.runs(job_id, started_at DESC);
CREATE INDEX idx_runs_status ON public.runs(user_id, status);

-- Row Level Security
ALTER TABLE public.jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.runs ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can only see their own jobs" ON public.jobs FOR SELECT USING (auth.uid() = user_id);
CREATE POLICY "Users can only insert their own jobs" ON public.jobs FOR INSERT WITH CHECK (auth.uid() = user_id);
CREATE POLICY "Users can only update their own jobs" ON public.jobs FOR UPDATE USING (auth.uid() = user_id);
CREATE POLICY "Users can only delete their own jobs" ON public.jobs FOR DELETE USING (auth.uid() = user_id);

CREATE POLICY "Users can only see their own runs" ON public.runs FOR SELECT USING (auth.uid() = user_id);
CREATE POLICY "Users can only insert their own runs" ON public.runs FOR INSERT WITH CHECK (auth.uid() = user_id);
CREATE POLICY "Users can only update their own runs" ON public.runs FOR UPDATE USING (auth.uid() = user_id);
CREATE POLICY "Users can only delete their own runs" ON public.runs FOR DELETE USING (auth.uid() = user_id);