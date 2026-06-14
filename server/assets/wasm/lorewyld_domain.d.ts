/* tslint:disable */
/* eslint-disable */

/**
 * Ability modifier for a raw score.
 */
export function ability_modifier(score: number): number;

/**
 * Derive every sheet stat from a JSON-serialized `CharacterSheet`, returned
 * as a JSON string for the caller to parse. Returns `"null"` on malformed
 * input rather than throwing, matching the mobile FFI's defensive contract.
 */
export function derive_stats(sheet_json: string): string;

/**
 * Proficiency bonus for a level (clamped 1..=20).
 */
export function proficiency_bonus(level: number): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly derive_stats: (a: number, b: number) => [number, number];
    readonly ability_modifier: (a: number) => number;
    readonly proficiency_bonus: (a: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
