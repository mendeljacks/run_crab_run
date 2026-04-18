import type { Job } from '../../stores/types'

export const ui_store = {
    page: 'dashboard' as string,
    createJobOpen: false as boolean,
    editJobOpen: false as boolean,
    editingJob: null as Job | null,
    deleteJobOpen: false as boolean,
    deletingJobId: '' as string,
    runDetailId: '' as string,
    runsSearch: '' as string,
    runsStatusFilter: '' as string
}

export type UiStore = typeof ui_store