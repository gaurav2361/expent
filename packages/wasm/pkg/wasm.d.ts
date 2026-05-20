/* tslint:disable */
/* eslint-disable */

export class PeriodBounds {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    end_ms: bigint;
    start_ms: bigint;
}

export class SavingsProjection {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    is_attainable: boolean;
    monthly_contribution: number;
    months_to_goal: number;
}

export class SpendingVelocity {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    daily_burn_rate: number;
    is_overpacing: boolean;
    projected_total: number;
    target_daily_rate: number;
}

export function calculate_budget_percentage(spent: string, limit: string): string;

export function calculate_spending_velocity(spent: string, limit: string, period: string): SpendingVelocity | undefined;

export function get_period_bounds(period: string): PeriodBounds | undefined;

export function is_transaction_in_period(txn_date_ms: bigint, period: string): boolean;

export function project_savings_goal(current_balance: string, target_amount: string, monthly_income: string, monthly_expenses: string): SavingsProjection | undefined;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_periodbounds_end_ms: (a: number) => bigint;
    readonly __wbg_get_periodbounds_start_ms: (a: number) => bigint;
    readonly __wbg_get_savingsprojection_is_attainable: (a: number) => number;
    readonly __wbg_get_savingsprojection_monthly_contribution: (a: number) => number;
    readonly __wbg_get_savingsprojection_months_to_goal: (a: number) => number;
    readonly __wbg_get_spendingvelocity_is_overpacing: (a: number) => number;
    readonly __wbg_get_spendingvelocity_projected_total: (a: number) => number;
    readonly __wbg_get_spendingvelocity_target_daily_rate: (a: number) => number;
    readonly __wbg_periodbounds_free: (a: number, b: number) => void;
    readonly __wbg_savingsprojection_free: (a: number, b: number) => void;
    readonly __wbg_set_periodbounds_end_ms: (a: number, b: bigint) => void;
    readonly __wbg_set_periodbounds_start_ms: (a: number, b: bigint) => void;
    readonly __wbg_set_savingsprojection_is_attainable: (a: number, b: number) => void;
    readonly __wbg_set_savingsprojection_monthly_contribution: (a: number, b: number) => void;
    readonly __wbg_set_savingsprojection_months_to_goal: (a: number, b: number) => void;
    readonly __wbg_set_spendingvelocity_is_overpacing: (a: number, b: number) => void;
    readonly __wbg_set_spendingvelocity_projected_total: (a: number, b: number) => void;
    readonly __wbg_set_spendingvelocity_target_daily_rate: (a: number, b: number) => void;
    readonly __wbg_spendingvelocity_free: (a: number, b: number) => void;
    readonly calculate_budget_percentage: (a: number, b: number, c: number, d: number) => [number, number];
    readonly calculate_spending_velocity: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly get_period_bounds: (a: number, b: number) => number;
    readonly is_transaction_in_period: (a: bigint, b: number, c: number) => number;
    readonly project_savings_goal: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
    readonly __wbg_get_spendingvelocity_daily_burn_rate: (a: number) => number;
    readonly __wbg_set_spendingvelocity_daily_burn_rate: (a: number, b: number) => void;
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
