/* @ts-self-types="./wasm.d.ts" */

export class PeriodBounds {
  static __wrap(ptr) {
    const obj = Object.create(PeriodBounds.prototype);
    obj.__wbg_ptr = ptr;
    PeriodBoundsFinalization.register(obj, obj.__wbg_ptr, obj);
    return obj;
  }
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    PeriodBoundsFinalization.unregister(this);
    return ptr;
  }
  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_periodbounds_free(ptr, 0);
  }
  /**
   * @returns {bigint}
   */
  get end_ms() {
    const ret = wasm.__wbg_get_periodbounds_end_ms(this.__wbg_ptr);
    return ret;
  }
  /**
   * @returns {bigint}
   */
  get start_ms() {
    const ret = wasm.__wbg_get_periodbounds_start_ms(this.__wbg_ptr);
    return ret;
  }
  /**
   * @param {bigint} arg0
   */
  set end_ms(arg0) {
    wasm.__wbg_set_periodbounds_end_ms(this.__wbg_ptr, arg0);
  }
  /**
   * @param {bigint} arg0
   */
  set start_ms(arg0) {
    wasm.__wbg_set_periodbounds_start_ms(this.__wbg_ptr, arg0);
  }
}
if (Symbol.dispose) PeriodBounds.prototype[Symbol.dispose] = PeriodBounds.prototype.free;

export class SavingsProjection {
  static __wrap(ptr) {
    const obj = Object.create(SavingsProjection.prototype);
    obj.__wbg_ptr = ptr;
    SavingsProjectionFinalization.register(obj, obj.__wbg_ptr, obj);
    return obj;
  }
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    SavingsProjectionFinalization.unregister(this);
    return ptr;
  }
  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_savingsprojection_free(ptr, 0);
  }
  /**
   * @returns {boolean}
   */
  get is_attainable() {
    const ret = wasm.__wbg_get_savingsprojection_is_attainable(this.__wbg_ptr);
    return ret !== 0;
  }
  /**
   * @returns {number}
   */
  get monthly_contribution() {
    const ret = wasm.__wbg_get_savingsprojection_monthly_contribution(this.__wbg_ptr);
    return ret;
  }
  /**
   * @returns {number}
   */
  get months_to_goal() {
    const ret = wasm.__wbg_get_savingsprojection_months_to_goal(this.__wbg_ptr);
    return ret;
  }
  /**
   * @param {boolean} arg0
   */
  set is_attainable(arg0) {
    wasm.__wbg_set_savingsprojection_is_attainable(this.__wbg_ptr, arg0);
  }
  /**
   * @param {number} arg0
   */
  set monthly_contribution(arg0) {
    wasm.__wbg_set_savingsprojection_monthly_contribution(this.__wbg_ptr, arg0);
  }
  /**
   * @param {number} arg0
   */
  set months_to_goal(arg0) {
    wasm.__wbg_set_savingsprojection_months_to_goal(this.__wbg_ptr, arg0);
  }
}
if (Symbol.dispose) SavingsProjection.prototype[Symbol.dispose] = SavingsProjection.prototype.free;

export class SpendingVelocity {
  static __wrap(ptr) {
    const obj = Object.create(SpendingVelocity.prototype);
    obj.__wbg_ptr = ptr;
    SpendingVelocityFinalization.register(obj, obj.__wbg_ptr, obj);
    return obj;
  }
  __destroy_into_raw() {
    const ptr = this.__wbg_ptr;
    this.__wbg_ptr = 0;
    SpendingVelocityFinalization.unregister(this);
    return ptr;
  }
  free() {
    const ptr = this.__destroy_into_raw();
    wasm.__wbg_spendingvelocity_free(ptr, 0);
  }
  /**
   * @returns {number}
   */
  get daily_burn_rate() {
    const ret = wasm.__wbg_get_spendingvelocity_daily_burn_rate(this.__wbg_ptr);
    return ret;
  }
  /**
   * @returns {boolean}
   */
  get is_overpacing() {
    const ret = wasm.__wbg_get_spendingvelocity_is_overpacing(this.__wbg_ptr);
    return ret !== 0;
  }
  /**
   * @returns {number}
   */
  get projected_total() {
    const ret = wasm.__wbg_get_spendingvelocity_projected_total(this.__wbg_ptr);
    return ret;
  }
  /**
   * @returns {number}
   */
  get target_daily_rate() {
    const ret = wasm.__wbg_get_spendingvelocity_target_daily_rate(this.__wbg_ptr);
    return ret;
  }
  /**
   * @param {number} arg0
   */
  set daily_burn_rate(arg0) {
    wasm.__wbg_set_spendingvelocity_daily_burn_rate(this.__wbg_ptr, arg0);
  }
  /**
   * @param {boolean} arg0
   */
  set is_overpacing(arg0) {
    wasm.__wbg_set_spendingvelocity_is_overpacing(this.__wbg_ptr, arg0);
  }
  /**
   * @param {number} arg0
   */
  set projected_total(arg0) {
    wasm.__wbg_set_spendingvelocity_projected_total(this.__wbg_ptr, arg0);
  }
  /**
   * @param {number} arg0
   */
  set target_daily_rate(arg0) {
    wasm.__wbg_set_spendingvelocity_target_daily_rate(this.__wbg_ptr, arg0);
  }
}
if (Symbol.dispose) SpendingVelocity.prototype[Symbol.dispose] = SpendingVelocity.prototype.free;

/**
 * @param {string} spent
 * @param {string} limit
 * @returns {string}
 */
export function calculate_budget_percentage(spent, limit) {
  let deferred3_0;
  let deferred3_1;
  try {
    const ptr0 = passStringToWasm0(spent, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(limit, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.calculate_budget_percentage(ptr0, len0, ptr1, len1);
    deferred3_0 = ret[0];
    deferred3_1 = ret[1];
    return getStringFromWasm0(ret[0], ret[1]);
  } finally {
    wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
  }
}

/**
 * @param {string} spent
 * @param {string} limit
 * @param {string} period
 * @returns {SpendingVelocity | undefined}
 */
export function calculate_spending_velocity(spent, limit, period) {
  const ptr0 = passStringToWasm0(spent, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len0 = WASM_VECTOR_LEN;
  const ptr1 = passStringToWasm0(limit, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len1 = WASM_VECTOR_LEN;
  const ptr2 = passStringToWasm0(period, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len2 = WASM_VECTOR_LEN;
  const ret = wasm.calculate_spending_velocity(ptr0, len0, ptr1, len1, ptr2, len2);
  return ret === 0 ? undefined : SpendingVelocity.__wrap(ret);
}

/**
 * @param {string} period
 * @returns {PeriodBounds | undefined}
 */
export function get_period_bounds(period) {
  const ptr0 = passStringToWasm0(period, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len0 = WASM_VECTOR_LEN;
  const ret = wasm.get_period_bounds(ptr0, len0);
  return ret === 0 ? undefined : PeriodBounds.__wrap(ret);
}

/**
 * @param {bigint} txn_date_ms
 * @param {string} period
 * @returns {boolean}
 */
export function is_transaction_in_period(txn_date_ms, period) {
  const ptr0 = passStringToWasm0(period, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len0 = WASM_VECTOR_LEN;
  const ret = wasm.is_transaction_in_period(txn_date_ms, ptr0, len0);
  return ret !== 0;
}

/**
 * @param {string} current_balance
 * @param {string} target_amount
 * @param {string} monthly_income
 * @param {string} monthly_expenses
 * @returns {SavingsProjection | undefined}
 */
export function project_savings_goal(current_balance, target_amount, monthly_income, monthly_expenses) {
  const ptr0 = passStringToWasm0(current_balance, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len0 = WASM_VECTOR_LEN;
  const ptr1 = passStringToWasm0(target_amount, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len1 = WASM_VECTOR_LEN;
  const ptr2 = passStringToWasm0(monthly_income, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len2 = WASM_VECTOR_LEN;
  const ptr3 = passStringToWasm0(monthly_expenses, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
  const len3 = WASM_VECTOR_LEN;
  const ret = wasm.project_savings_goal(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
  return ret === 0 ? undefined : SavingsProjection.__wrap(ret);
}
function __wbg_get_imports() {
  const import0 = {
    __proto__: null,
    __wbg___wbindgen_throw_9c31b086c2b26051: function (arg0, arg1) {
      throw new Error(getStringFromWasm0(arg0, arg1));
    },
    __wbg_getTime_09f1dd40a44edb30: function (arg0) {
      const ret = arg0.getTime();
      return ret;
    },
    __wbg_new_0_2722fcdb71a888a6: function () {
      const ret = new Date();
      return ret;
    },
    __wbindgen_init_externref_table: function () {
      const table = wasm.__wbindgen_externrefs;
      const offset = table.grow(4);
      table.set(0, undefined);
      table.set(offset + 0, undefined);
      table.set(offset + 1, null);
      table.set(offset + 2, true);
      table.set(offset + 3, false);
    },
  };
  return {
    __proto__: null,
    "./wasm_bg.js": import0,
  };
}

const PeriodBoundsFinalization =
  typeof FinalizationRegistry === "undefined"
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry((ptr) => wasm.__wbg_periodbounds_free(ptr, 1));
const SavingsProjectionFinalization =
  typeof FinalizationRegistry === "undefined"
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry((ptr) => wasm.__wbg_savingsprojection_free(ptr, 1));
const SpendingVelocityFinalization =
  typeof FinalizationRegistry === "undefined"
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry((ptr) => wasm.__wbg_spendingvelocity_free(ptr, 1));

function getStringFromWasm0(ptr, len) {
  return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
  if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
    cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
  }
  return cachedUint8ArrayMemory0;
}

function passStringToWasm0(arg, malloc, realloc) {
  if (realloc === undefined) {
    const buf = cachedTextEncoder.encode(arg);
    const ptr = malloc(buf.length, 1) >>> 0;
    getUint8ArrayMemory0()
      .subarray(ptr, ptr + buf.length)
      .set(buf);
    WASM_VECTOR_LEN = buf.length;
    return ptr;
  }

  let len = arg.length;
  let ptr = malloc(len, 1) >>> 0;

  const mem = getUint8ArrayMemory0();

  let offset = 0;

  for (; offset < len; offset++) {
    const code = arg.charCodeAt(offset);
    if (code > 0x7f) break;
    mem[ptr + offset] = code;
  }
  if (offset !== len) {
    if (offset !== 0) {
      arg = arg.slice(offset);
    }
    ptr = realloc(ptr, len, (len = offset + arg.length * 3), 1) >>> 0;
    const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
    const ret = cachedTextEncoder.encodeInto(arg, view);

    offset += ret.written;
    ptr = realloc(ptr, len, offset, 1) >>> 0;
  }

  WASM_VECTOR_LEN = offset;
  return ptr;
}

let cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
  numBytesDecoded += len;
  if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
    cachedTextDecoder = new TextDecoder("utf-8", { ignoreBOM: true, fatal: true });
    cachedTextDecoder.decode();
    numBytesDecoded = len;
  }
  return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!("encodeInto" in cachedTextEncoder)) {
  cachedTextEncoder.encodeInto = function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
      read: arg.length,
      written: buf.length,
    };
  };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
  wasmInstance = instance;
  wasm = instance.exports;
  wasmModule = module;
  cachedUint8ArrayMemory0 = null;
  wasm.__wbindgen_start();
  return wasm;
}

async function __wbg_load(module, imports) {
  if (typeof Response === "function" && module instanceof Response) {
    if (typeof WebAssembly.instantiateStreaming === "function") {
      try {
        return await WebAssembly.instantiateStreaming(module, imports);
      } catch (e) {
        const validResponse = module.ok && expectedResponseType(module.type);

        if (validResponse && module.headers.get("Content-Type") !== "application/wasm") {
          console.warn(
            "`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n",
            e,
          );
        } else {
          throw e;
        }
      }
    }

    const bytes = await module.arrayBuffer();
    return await WebAssembly.instantiate(bytes, imports);
  } else {
    const instance = await WebAssembly.instantiate(module, imports);

    if (instance instanceof WebAssembly.Instance) {
      return { instance, module };
    } else {
      return instance;
    }
  }

  function expectedResponseType(type) {
    switch (type) {
      case "basic":
      case "cors":
      case "default":
        return true;
    }
    return false;
  }
}

function initSync(module) {
  if (wasm !== undefined) return wasm;

  if (module !== undefined) {
    if (Object.getPrototypeOf(module) === Object.prototype) {
      ({ module } = module);
    } else {
      console.warn("using deprecated parameters for `initSync()`; pass a single object instead");
    }
  }

  const imports = __wbg_get_imports();
  if (!(module instanceof WebAssembly.Module)) {
    module = new WebAssembly.Module(module);
  }
  const instance = new WebAssembly.Instance(module, imports);
  return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
  if (wasm !== undefined) return wasm;

  if (module_or_path !== undefined) {
    if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
      ({ module_or_path } = module_or_path);
    } else {
      console.warn("using deprecated parameters for the initialization function; pass a single object instead");
    }
  }

  if (module_or_path === undefined) {
    module_or_path = new URL("wasm_bg.wasm", import.meta.url);
  }
  const imports = __wbg_get_imports();

  if (
    typeof module_or_path === "string" ||
    (typeof Request === "function" && module_or_path instanceof Request) ||
    (typeof URL === "function" && module_or_path instanceof URL)
  ) {
    module_or_path = fetch(module_or_path);
  }

  const { instance, module } = await __wbg_load(await module_or_path, imports);

  return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
