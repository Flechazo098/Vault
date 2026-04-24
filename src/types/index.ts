// ── Frontend Type Definitions ───────────────────────────────
// These mirror the Rust backend structures for TypeScript safety.

/// Status of the vault returned from the backend.
export interface VaultStatus {
  initialized: boolean;
  locked: boolean;
  entry_count: number;
}

/// A summary of an entry (no secret included).
export interface EntrySummary {
  id: string;
  name: string;
  entry_type: string;
  description: string;
  category: string;
  created_at: string;
  updated_at: string;
  last_used_at: string | null;
}

/// Input for creating or updating an entry.
export interface EntryInput {
  name: string;
  entry_type: string;
  description: string;
  secret: string;
  category: string;
}

/// A full entry that includes the decrypted secret.
/// Used transiently for copy/view operations.
export interface EntryWithSecret extends EntrySummary {
  secret: string;
}

/// Predefined entry types for the type selector.
export const ENTRY_TYPES = [
  'API Key',
  'Token',
  'Secret',
  'Password',
  'Certificate',
  'Other',
] as const;

/// Predefined categories for the category selector.
export const PRESET_CATEGORIES = [
  'General',
  'Development',
  'Production',
  'Staging',
  'Infrastructure',
  'Database',
  'Cloud',
  'Authentication',
  'CI/CD',
] as const;

/// Result of an import operation — full entry list after import.
export interface ImportResult {
  entries: EntrySummary[];
}
