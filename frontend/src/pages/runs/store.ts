import type { Run } from '../../stores/types'

export const runs_store = {
    list: [] as Run[],
    loading: false as boolean,
    error: '' as string,
    selectedJobId: '' as string
}

export type RunsStore = typeof runs_store