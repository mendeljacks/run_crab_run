import { jwtVerify, createRemoteJWKSet } from 'jose'
import type { Request, Response, NextFunction } from 'express'

const JWT_SECRET = process.env.JWT_SECRET
const SUPABASE_URL = process.env.SUPABASE_URL

if (!JWT_SECRET && !SUPABASE_URL) {
    throw new Error('Either JWT_SECRET or SUPABASE_URL environment variable is required')
}

export interface AuthenticatedRequest extends Request {
    userId: string
}

// Create a JWKS resolver if SUPABASE_URL is provided (for local dev with Supabase CLI)
// This handles ES256 tokens signed by GoTrue
let jwks: ReturnType<typeof createRemoteJWKSet> | null = null
if (SUPABASE_URL) {
    const jwksUrl = new URL('/auth/v1/.well-known/jwks.json', SUPABASE_URL)
    jwks = createRemoteJWKSet(jwksUrl)
}

/**
 * Middleware that validates the Supabase JWT from the Authorization header
 * and extracts the user ID.
 *
 * Supports two verification modes:
 * 1. JWKS (recommended for Supabase CLI local dev): Set SUPABASE_URL to your
 *    local Supabase API URL (e.g. http://127.0.0.1:54321). Tokens are verified
 *    against the JWKS endpoint (handles ES256 keys).
 * 2. HS256 secret (for self-hosted K8s with shared JWT_SECRET): Set JWT_SECRET
 *    to the same value used by GoTrue. Tokens are verified with HMAC.
 *
 * Both can be set — JWKS attempt runs first, falls back to HS256.
 */
export const authMiddleware = async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    const authHeader = req.headers.authorization
    if (!authHeader?.startsWith('Bearer ')) {
        res.status(401).json({ error: 'Missing authorization header' })
        return
    }

    const token = authHeader.slice(7)

    let userId: string | undefined

    // Try JWKS verification first (handles ES256 from Supabase CLI)
    if (jwks) {
        try {
            const { payload } = await jwtVerify(token, jwks, {
                algorithms: ['ES256', 'RS256', 'HS256']
            })
            if (payload.sub) {
                userId = payload.sub
            }
        } catch {
            // JWKS verification failed, try HS256 fallback if available
        }
    }

    // Fallback: HS256 verification with shared secret (for self-hosted K8s)
    if (!userId && JWT_SECRET) {
        try {
            const { payload } = await jwtVerify(token, new TextEncoder().encode(JWT_SECRET), {
                algorithms: ['HS256']
            })
            if (payload.sub) {
                userId = payload.sub
            }
        } catch {
            // Both verification methods failed
        }
    }

    if (!userId) {
        res.status(401).json({ error: 'Invalid or expired token' })
        return
    }

    ;(req as AuthenticatedRequest).userId = userId
    next()
}