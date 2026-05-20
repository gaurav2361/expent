/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const __wbg_get_periodbounds_end_ms: (a: number) => bigint;
export const __wbg_get_periodbounds_start_ms: (a: number) => bigint;
export const __wbg_get_savingsprojection_is_attainable: (a: number) => number;
export const __wbg_get_savingsprojection_monthly_contribution: (a: number) => number;
export const __wbg_get_savingsprojection_months_to_goal: (a: number) => number;
export const __wbg_get_spendingvelocity_is_overpacing: (a: number) => number;
export const __wbg_get_spendingvelocity_projected_total: (a: number) => number;
export const __wbg_get_spendingvelocity_target_daily_rate: (a: number) => number;
export const __wbg_periodbounds_free: (a: number, b: number) => void;
export const __wbg_savingsprojection_free: (a: number, b: number) => void;
export const __wbg_set_periodbounds_end_ms: (a: number, b: bigint) => void;
export const __wbg_set_periodbounds_start_ms: (a: number, b: bigint) => void;
export const __wbg_set_savingsprojection_is_attainable: (a: number, b: number) => void;
export const __wbg_set_savingsprojection_monthly_contribution: (a: number, b: number) => void;
export const __wbg_set_savingsprojection_months_to_goal: (a: number, b: number) => void;
export const __wbg_set_spendingvelocity_is_overpacing: (a: number, b: number) => void;
export const __wbg_set_spendingvelocity_projected_total: (a: number, b: number) => void;
export const __wbg_set_spendingvelocity_target_daily_rate: (a: number, b: number) => void;
export const __wbg_spendingvelocity_free: (a: number, b: number) => void;
export const calculate_budget_percentage: (a: number, b: number, c: number, d: number) => [number, number];
export const calculate_spending_velocity: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
export const get_period_bounds: (a: number, b: number) => number;
export const is_transaction_in_period: (a: bigint, b: number, c: number) => number;
export const project_savings_goal: (
  a: number,
  b: number,
  c: number,
  d: number,
  e: number,
  f: number,
  g: number,
  h: number,
) => number;
export const __wbg_get_spendingvelocity_daily_burn_rate: (a: number) => number;
export const __wbg_set_spendingvelocity_daily_burn_rate: (a: number, b: number) => void;
export const __wbindgen_externrefs: WebAssembly.Table;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_start: () => void;
