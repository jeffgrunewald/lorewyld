/* tslint:disable */
/* eslint-disable */

/**
 * Ability modifier for a raw score.
 */
export function ability_modifier(score: number): number;

/**
 * The skeleton record for a category, as a JSON string used to seed a new
 * authoring form. `"null"` for an unknown category.
 */
export function default_record(category: string): string;

/**
 * Derive every sheet stat from a JSON-serialized `CharacterSheet`, returned
 * as a JSON string for the caller to parse. Returns `"null"` on malformed
 * input rather than throwing, matching the mobile FFI's defensive contract.
 */
export function derive_stats(sheet_json: string): string;

/**
 * The authoring [`FieldSchema`](lorewyld_domain::FieldSchema) for a content
 * category, as a JSON string the form builder renders from. `"null"` for an
 * unknown/unauthorable category.
 */
export function field_schema(category: string): string;

/**
 * The fixed new-player guidance questionnaire, as a JSON string the quiz
 * UI renders from.
 */
export function guidance_questionnaire(): string;

/**
 * Rank candidate records against quiz answers. `answers_json` is a JSON
 * array of `{question, value}`; `candidates_json` is a JSON object of
 * `{classes, species, backgrounds}` record arrays (summaries suffice).
 * Returns a JSON `RecommendationSet`, or `"null"` on malformed input.
 */
export function guidance_recommend(answers_json: string, candidates_json: string): string;

/**
 * Proficiency bonus for a level (clamped 1..=20).
 */
export function proficiency_bonus(level: number): number;

/**
 * Validate authoring input (a JSON object of field values) against the
 * category's schema. Returns a JSON array of `{field, message}` errors —
 * `"[]"` means valid. The web form runs this pre-submit; the server runs
 * the same `lorewyld_domain::validate_record` authoritatively.
 */
export function validate_record(category: string, input_json: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly default_record: (a: number, b: number) => [number, number];
    readonly derive_stats: (a: number, b: number) => [number, number];
    readonly field_schema: (a: number, b: number) => [number, number];
    readonly guidance_questionnaire: () => [number, number];
    readonly guidance_recommend: (a: number, b: number, c: number, d: number) => [number, number];
    readonly validate_record: (a: number, b: number, c: number, d: number) => [number, number];
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
