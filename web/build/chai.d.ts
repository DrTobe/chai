/* tslint:disable */
/* eslint-disable */
/**
* @returns {string}
*/
export function new_game(): string;
/**
* @param {string} json_game
* @param {number} field
* @returns {string}
*/
export function get_legal_moves_for_single_piece(json_game: string, field: number): string;
/**
* @param {string} json_game
* @returns {string}
*/
export function get_minimax_move(json_game: string): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly new_game: (a: number) => void;
  readonly get_legal_moves_for_single_piece: (a: number, b: number, c: number, d: number) => void;
  readonly get_minimax_move: (a: number, b: number, c: number) => void;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
        