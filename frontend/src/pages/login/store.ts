// Auth store — simple observable state for the auth UI
// Supabase handles token persistence via its own session management
export const auth_store = {
    token: '' as string,
    loading: false as boolean
}

export type AuthStore = typeof auth_store