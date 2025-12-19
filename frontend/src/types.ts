// TypeScript types matching the Rust backend types
// Keep these in sync with src/lib.rs

export interface RouteConfig {
    id: number;
    path: string;
    method: string;
    description?: string | null;
}

export interface RequestLog {
    timestamp: string;
    method: string;
    path: string;
    status: number;
}

