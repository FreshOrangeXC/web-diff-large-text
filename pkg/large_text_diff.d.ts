/* tslint:disable */
/* eslint-disable */

/**
 * 对两段多行文本做行级 diff，返回 JSON 字符串。
 *
 * # 参数
 * - `before`    原始文本（整块，含换行符）
 * - `after`     修改后文本
 * - `algorithm` `"histogram"`（默认，推荐）或 `"myers"`
 *
 * # 返回值
 * JSON 数组，每个元素：
 * ```json
 * { "type": "equal"|"delete"|"insert",
 *   "before_start": u32, "before_end": u32,
 *   "after_start":  u32, "after_end":  u32 }
 * ```
 * 行号从 0 开始，区间为左闭右开 `[start, end)`。
 */
export function diff_lines(before: string, after: string, algorithm: string): string;

export function init_panic_hook(): void;

export function wasm_version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly diff_lines: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly init_panic_hook: () => void;
    readonly wasm_version: (a: number) => void;
    readonly __wbindgen_export: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export2: (a: number, b: number) => number;
    readonly __wbindgen_export3: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
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
