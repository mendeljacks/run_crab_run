import type { Job } from '../../stores/types'

export const jobs_store = {
    list: [] as Job[],
    loading: false as boolean,
    error: '' as string
}

export type JobsStore = typeof jobs_store