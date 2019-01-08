/* tslint:disable */
import * as wasm from './wasmtest_bg';
import { Node } from './yoga';
import { LayOut } from './yoga';

const heap = new Array(32);

heap.fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    if (typeof(heap_next) !== 'number') throw new Error('corrupt heap');

    heap[idx] = obj;
    return idx;
}

export function __wbg_new_9292473d8fb57b99() {
    return addHeapObject(new Node());
}

function getObject(idx) { return heap[idx]; }

export function __wbg_setWidth_38f64bb13cb0c183(arg0, arg1) {
    getObject(arg0).setWidth(arg1);
}

export function __wbg_setHeight_442d1185eb0cea5b(arg0, arg1) {
    getObject(arg0).setHeight(arg1);
}

export function __wbg_getComputedLayout_5ece483c209332f8(arg0) {
    return addHeapObject(getObject(arg0).getComputedLayout());
}

export function __wbg_calculateLayout_a75db4e6bf6d78e4(arg0, arg1, arg2, arg3) {
    getObject(arg0).calculateLayout(arg1, arg2, arg3);
}

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

export function __wbg_insertChild_c1e8c9bd8e89b276(arg0, arg1, arg2) {
    getObject(arg0).insertChild(takeObject(arg1), arg2);
}

export function __wbg_getChild_adc2b94ae0b83776(arg0, arg1) {
    return addHeapObject(getObject(arg0).getChild(arg1));
}

export function __wbg_getLeft_be4e3d53a496d828(arg0) {
    return getObject(arg0).getLeft();
}

let cachedTextDecoder = new TextDecoder('utf-8');

let cachegetUint8Memory = null;
function getUint8Memory() {
    if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory;
}

function getStringFromWasm(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

export function __wbg_alert_33cbf07b9a94ee59(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    alert(varg0);
}
/**
* @returns {void}
*/
export function greet() {
    return wasm.greet();
}

export function __wbindgen_object_drop_ref(i) { dropObject(i); }

