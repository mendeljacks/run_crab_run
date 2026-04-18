import { observable, toJS } from 'mobx'
import { jobs_store } from '../pages/jobs/store'
import { runs_store } from '../pages/runs/store'
import { ui_store } from '../pages/ui/store'

// Auth is managed via Supabase — we just keep a token reference here
// for the MobX observability layer
const auth_store = {
    token: '' as string,
    loading: false as boolean
}

export type AuthStore = typeof auth_store

export const store = observable({
    auth: auth_store,
    jobs: jobs_store,
    runs: runs_store,
    ui: ui_store
})

export type StoreType = typeof store

interface WindowWithStore {
    store: StoreType
    toJS: typeof toJS
}

const win = window as unknown as WindowWithStore
win.store = store
win.toJS = toJS

if (import.meta.hot) {
    if (import.meta.hot.data.store) {
        const prev = import.meta.hot.data.store as StoreType
        Object.assign(store.auth, prev.auth)
        Object.assign(store.jobs, prev.jobs)
        Object.assign(store.runs, prev.runs)
        Object.assign(store.ui, prev.ui)
    }
    import.meta.hot.dispose((data) => {
        data.store = store
    })
}