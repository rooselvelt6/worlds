export class IntoUnderlyingByteSource {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingByteSourceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingbytesource_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get autoAllocateChunkSize() {
        const ret = wasm.intounderlyingbytesource_autoAllocateChunkSize(this.__wbg_ptr);
        return ret >>> 0;
    }
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingbytesource_cancel(ptr);
    }
    /**
     * @param {ReadableByteStreamController} controller
     * @returns {Promise<any>}
     */
    pull(controller) {
        const ret = wasm.intounderlyingbytesource_pull(this.__wbg_ptr, controller);
        return ret;
    }
    /**
     * @param {ReadableByteStreamController} controller
     */
    start(controller) {
        wasm.intounderlyingbytesource_start(this.__wbg_ptr, controller);
    }
    /**
     * @returns {ReadableStreamType}
     */
    get type() {
        const ret = wasm.intounderlyingbytesource_type(this.__wbg_ptr);
        return __wbindgen_enum_ReadableStreamType[ret];
    }
}
if (Symbol.dispose) IntoUnderlyingByteSource.prototype[Symbol.dispose] = IntoUnderlyingByteSource.prototype.free;

export class IntoUnderlyingSink {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingSinkFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsink_free(ptr, 0);
    }
    /**
     * @param {any} reason
     * @returns {Promise<any>}
     */
    abort(reason) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_abort(ptr, reason);
        return ret;
    }
    /**
     * @returns {Promise<any>}
     */
    close() {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.intounderlyingsink_close(ptr);
        return ret;
    }
    /**
     * @param {any} chunk
     * @returns {Promise<any>}
     */
    write(chunk) {
        const ret = wasm.intounderlyingsink_write(this.__wbg_ptr, chunk);
        return ret;
    }
}
if (Symbol.dispose) IntoUnderlyingSink.prototype[Symbol.dispose] = IntoUnderlyingSink.prototype.free;

export class IntoUnderlyingSource {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IntoUnderlyingSourceFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_intounderlyingsource_free(ptr, 0);
    }
    cancel() {
        const ptr = this.__destroy_into_raw();
        wasm.intounderlyingsource_cancel(ptr);
    }
    /**
     * @param {ReadableStreamDefaultController} controller
     * @returns {Promise<any>}
     */
    pull(controller) {
        const ret = wasm.intounderlyingsource_pull(this.__wbg_ptr, controller);
        return ret;
    }
}
if (Symbol.dispose) IntoUnderlyingSource.prototype[Symbol.dispose] = IntoUnderlyingSource.prototype.free;

export function start() {
    wasm.start();
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_debug_string_07cb72cfcc952e2b: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_falsy_f076b393b3ef7644: function(arg0) {
            const ret = !arg0;
            return ret;
        },
        __wbg___wbindgen_is_function_2f0fd7ceb86e64c5: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_066086be3abe9bb3: function(arg0) {
            const ret = arg0 === null;
            return ret;
        },
        __wbg___wbindgen_is_undefined_244a92c34d3b6ec0: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_number_get_dd6d69a6079f26f1: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_965592073e5d848c: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_9c75d47bf9e7731e: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_158e43e869788cdc: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_addEventListener_a95e75babfc4f5a3: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_arc_ebc74f7abf32eace: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.arc(arg1, arg2, arg3, arg4, arg5);
        }, arguments); },
        __wbg_axes_80669e8633f8b14e: function(arg0) {
            const ret = arg0.axes;
            return ret;
        },
        __wbg_beginPath_d31f98e44cba3be0: function(arg0) {
            arg0.beginPath();
        },
        __wbg_body_9a319c5d4ea2d0d8: function(arg0) {
            const ret = arg0.body;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_buffer_9ee17426fe5a5d65: function(arg0) {
            const ret = arg0.buffer;
            return ret;
        },
        __wbg_button_9121eff76035e6f3: function(arg0) {
            const ret = arg0.button;
            return ret;
        },
        __wbg_buttons_c4b0491af6752e80: function(arg0) {
            const ret = arg0.buttons;
            return ret;
        },
        __wbg_byobRequest_178b64c09a0bee03: function(arg0) {
            const ret = arg0.byobRequest;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_byteLength_1f57c71e64ee0180: function(arg0) {
            const ret = arg0.byteLength;
            return ret;
        },
        __wbg_byteOffset_648d0af273024f3d: function(arg0) {
            const ret = arg0.byteOffset;
            return ret;
        },
        __wbg_call_a41d6421b30a32c5: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_call_add9e5a76382e668: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.call(arg1);
            return ret;
        }, arguments); },
        __wbg_cancelAnimationFrame_44f7b2b0c5c39988: function() { return handleError(function (arg0, arg1) {
            arg0.cancelAnimationFrame(arg1);
        }, arguments); },
        __wbg_cancelBubble_b55456d03cb06b55: function(arg0) {
            const ret = arg0.cancelBubble;
            return ret;
        },
        __wbg_clientX_d68312e38d37c06a: function(arg0) {
            const ret = arg0.clientX;
            return ret;
        },
        __wbg_clientY_c414f2d35e1ec005: function(arg0) {
            const ret = arg0.clientY;
            return ret;
        },
        __wbg_clipboard_ed0015a88db5242e: function(arg0) {
            const ret = arg0.clipboard;
            return ret;
        },
        __wbg_cloneNode_c94aa99ab0c25fa5: function() { return handleError(function (arg0) {
            const ret = arg0.cloneNode();
            return ret;
        }, arguments); },
        __wbg_cloneNode_ff15458cb0d2c300: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.cloneNode(arg1 !== 0);
            return ret;
        }, arguments); },
        __wbg_closePath_b438c379d0897f55: function(arg0) {
            arg0.closePath();
        },
        __wbg_close_63e009c5a75f5597: function() { return handleError(function (arg0) {
            arg0.close();
        }, arguments); },
        __wbg_close_c9bdd6a3be7d122d: function(arg0) {
            arg0.close();
        },
        __wbg_close_de471367367aa5cb: function() { return handleError(function (arg0) {
            arg0.close();
        }, arguments); },
        __wbg_composedPath_bb138d201a2e1f3a: function(arg0) {
            const ret = arg0.composedPath();
            return ret;
        },
        __wbg_connect_b0c6d44e9984ca8e: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.connect(arg1);
            return ret;
        }, arguments); },
        __wbg_connected_6561abd475cda6df: function(arg0) {
            const ret = arg0.connected;
            return ret;
        },
        __wbg_content_6ead30b629a1b55d: function(arg0) {
            const ret = arg0.content;
            return ret;
        },
        __wbg_continue_f4956791611d87f7: function() { return handleError(function (arg0) {
            arg0.continue();
        }, arguments); },
        __wbg_copyToChannel_2336494173d6d1be: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.copyToChannel(arg1, arg2);
        }, arguments); },
        __wbg_createBuffer_5e53e4a1f2e73720: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.createBuffer(arg1 >>> 0, arg2 >>> 0, arg3);
            return ret;
        }, arguments); },
        __wbg_createComment_30fa767a9938455e: function(arg0, arg1, arg2) {
            const ret = arg0.createComment(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_createElementNS_edf667dff759d26c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            const ret = arg0.createElementNS(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
            return ret;
        }, arguments); },
        __wbg_createElement_679cad83bb50288c: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createGain_33464d2fccb13fb8: function() { return handleError(function (arg0) {
            const ret = arg0.createGain();
            return ret;
        }, arguments); },
        __wbg_createObjectStore_055630e50b060ddc: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createObjectStore(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createOscillator_c64f08349366ee0d: function() { return handleError(function (arg0) {
            const ret = arg0.createOscillator();
            return ret;
        }, arguments); },
        __wbg_createTextNode_656fb5ad1bda1089: function(arg0, arg1, arg2) {
            const ret = arg0.createTextNode(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_currentTime_6bf7644ca0c23256: function(arg0) {
            const ret = arg0.currentTime;
            return ret;
        },
        __wbg_deleteProperty_9fd68e56d0213328: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.deleteProperty(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_delete_44cfabf3a012f317: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.delete(arg1);
            return ret;
        }, arguments); },
        __wbg_destination_a7fb84721246ff2f: function(arg0) {
            const ret = arg0.destination;
            return ret;
        },
        __wbg_disconnect_8d82e5b39642c4c8: function() { return handleError(function (arg0, arg1) {
            arg0.disconnect(arg1);
        }, arguments); },
        __wbg_document_69bb6a2f7927d532: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_enqueue_6c7cd543c0f3828e: function() { return handleError(function (arg0, arg1) {
            arg0.enqueue(arg1);
        }, arguments); },
        __wbg_error_48655ee7e4756f8b: function(arg0) {
            console.error(arg0);
        },
        __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_fetch_dc020402ef5b5b70: function(arg0, arg1, arg2) {
            const ret = arg0.fetch(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_fillRect_9219f775d7e8e73e: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.fillRect(arg1, arg2, arg3, arg4);
        },
        __wbg_fillText_9fbea3af94326c74: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.fillText(getStringFromWasm0(arg1, arg2), arg3, arg4);
        }, arguments); },
        __wbg_fill_eb2f573270ef9b6d: function(arg0) {
            arg0.fill();
        },
        __wbg_firstElementChild_1d49d1094b14cf60: function(arg0) {
            const ret = arg0.firstElementChild;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_focus_6fb3e144d2c12c7f: function() { return handleError(function (arg0) {
            arg0.focus();
        }, arguments); },
        __wbg_frequency_0f0dc1e8480ee4d1: function(arg0) {
            const ret = arg0.frequency;
            return ret;
        },
        __wbg_gain_c994bc21cdd2e1b9: function(arg0) {
            const ret = arg0.gain;
            return ret;
        },
        __wbg_getBoundingClientRect_e0fb035288f4a416: function(arg0) {
            const ret = arg0.getBoundingClientRect();
            return ret;
        },
        __wbg_getContext_f17252002286474d: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getElementById_22becc83cca95cc2: function(arg0, arg1, arg2) {
            const ret = arg0.getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getGamepads_015e883edab3d776: function() { return handleError(function (arg0) {
            const ret = arg0.getGamepads();
            return ret;
        }, arguments); },
        __wbg_getItem_f68808a9230dd173: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.getItem(getStringFromWasm0(arg2, arg3));
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_get_41476db20fef99a8: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_652f640b3b0b6e3e: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_height_74c12c942761f846: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_height_f036cb27636625f6: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_host_4c1f4b789926d154: function(arg0) {
            const ret = arg0.host;
            return ret;
        },
        __wbg_href_53712054c453ff9f: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.href;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_indexedDB_06cbacc078ae71b2: function() { return handleError(function (arg0) {
            const ret = arg0.indexedDB;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_insertBefore_e97e77a75bb55860: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.insertBefore(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_instanceof_CanvasRenderingContext2d_b433938013de3a1e: function(arg0) {
            let result;
            try {
                result = arg0 instanceof CanvasRenderingContext2D;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Element_515917c379f32ac4: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Element;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GamepadButton_8c744db616c78f92: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GamepadButton;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Gamepad_8432edf7ba4c6cf2: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Gamepad;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlInputElement_d829a3cb28c8ad8f: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLInputElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_IdbCursorWithValue_65f903f099fea99f: function(arg0) {
            let result;
            try {
                result = arg0 instanceof IDBCursorWithValue;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_IdbDatabase_ec41c6a6b2f95dc9: function(arg0) {
            let result;
            try {
                result = arg0 instanceof IDBDatabase;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Response_370b83aa6c17e88a: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Response;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_ShadowRoot_52c7974a7a27fd4c: function(arg0) {
            let result;
            try {
                result = arg0 instanceof ShadowRoot;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_4153c1818a1c0c0b: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isArray_c6c6ef8308995bcf: function(arg0) {
            const ret = Array.isArray(arg0);
            return ret;
        },
        __wbg_key_2e79b9dbd4550ab3: function(arg0, arg1) {
            const ret = arg1.key;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_key_ed37bd2129049b6c: function() { return handleError(function (arg0) {
            const ret = arg0.key;
            return ret;
        }, arguments); },
        __wbg_language_2f20c76888b8bc2e: function(arg0, arg1) {
            const ret = arg1.language;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_left_ed21748ed5f587d7: function(arg0) {
            const ret = arg0.left;
            return ret;
        },
        __wbg_length_0a6ce016dc1460b0: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_ba3c032602efe310: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_lineTo_fe5522fbbf79a59d: function(arg0, arg1, arg2) {
            arg0.lineTo(arg1, arg2);
        },
        __wbg_linearRampToValueAtTime_9ed061559bf44548: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.linearRampToValueAtTime(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_localStorage_11b5275c3ad2bab7: function() { return handleError(function (arg0) {
            const ret = arg0.localStorage;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_location_0f18c0567ac29e07: function(arg0) {
            const ret = arg0.location;
            return ret;
        },
        __wbg_log_72d22df918dcc232: function(arg0) {
            console.log(arg0);
        },
        __wbg_moveTo_89e84c82679f8ac9: function(arg0, arg1, arg2) {
            arg0.moveTo(arg1, arg2);
        },
        __wbg_movementX_234cea13fe25dae4: function(arg0) {
            const ret = arg0.movementX;
            return ret;
        },
        __wbg_movementY_3a54512f6f23708b: function(arg0) {
            const ret = arg0.movementY;
            return ret;
        },
        __wbg_navigator_f3468c6dc9006b7c: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_3baa8d9866155c79: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_66c1d98fddc2e0c4: function() { return handleError(function (arg0) {
            const ret = new ConvolverNode(arg0);
            return ret;
        }, arguments); },
        __wbg_new_a6b46eaf9085fbeb: function() { return handleError(function () {
            const ret = new lAudioContext();
            return ret;
        }, arguments); },
        __wbg_new_c9ea13ea803a692e: function(arg0, arg1) {
            const ret = new Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_eb8acd9352be84ba: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen__convert__closures_____invoke__h9885bac28915fc8d(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return ret;
            } finally {
                state0.a = 0;
            }
        },
        __wbg_new_from_slice_0f99167330d1143b: function(arg0, arg1) {
            const ret = new Float32Array(getArrayF32FromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_from_slice_823acd363b3844cf: function(arg0, arg1) {
            const ret = new Uint32Array(getArrayU32FromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_typed_1137602701dc87d4: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen__convert__closures_____invoke__h625988fa5e0206e9(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return ret;
            } finally {
                state0.a = 0;
            }
        },
        __wbg_new_with_byte_offset_and_length_643e5e9e2fb6b1ad: function(arg0, arg1, arg2) {
            const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_new_with_length_d360e1480e55002f: function(arg0) {
            const ret = new Float32Array(arg0 >>> 0);
            return ret;
        },
        __wbg_now_4f457f10f864aec5: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_now_b205f8c23840112e: function(arg0) {
            const ret = arg0.now();
            return ret;
        },
        __wbg_objectStore_20dd598b399f89ea: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.objectStore(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_openCursor_ebc2623e3193a5e5: function() { return handleError(function (arg0) {
            const ret = arg0.openCursor();
            return ret;
        }, arguments); },
        __wbg_open_5a9e43bc5e42b3c5: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.open(getStringFromWasm0(arg1, arg2), arg3 >>> 0);
            return ret;
        }, arguments); },
        __wbg_parentNode_c5865dc42e23bdcd: function(arg0) {
            const ret = arg0.parentNode;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_performance_8e9fec534a95f99f: function(arg0) {
            const ret = arg0.performance;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_pointerLockElement_13e9e143ab751cad: function(arg0) {
            const ret = arg0.pointerLockElement;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_pressed_f3474d2085f7d3f1: function(arg0) {
            const ret = arg0.pressed;
            return ret;
        },
        __wbg_preventDefault_2c34c219d9b04b86: function(arg0) {
            arg0.preventDefault();
        },
        __wbg_put_3964c453c1f17a2b: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.put(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_queueMicrotask_40ac6ffc2848ba77: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_queueMicrotask_74d092439f6494c1: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_random_fc287e2ecb3e2805: function() {
            const ret = Math.random();
            return ret;
        },
        __wbg_register_09d5b5de43664dad: function(arg0, arg1, arg2) {
            const ret = arg0.register(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_removeEventListener_2ce4c0697d2b692c: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.removeEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_removeItem_a5faee82be5c6ed1: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.removeItem(getStringFromWasm0(arg1, arg2));
        }, arguments); },
        __wbg_remove_6e8ac6d05597c920: function(arg0) {
            arg0.remove();
        },
        __wbg_remove_cd0727e0f0c757f2: function(arg0) {
            arg0.remove();
        },
        __wbg_requestAnimationFrame_d187174d7b146805: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestAnimationFrame(arg1);
            return ret;
        }, arguments); },
        __wbg_requestFullscreen_f1ea4024677ac57a: function() { return handleError(function (arg0) {
            arg0.requestFullscreen();
        }, arguments); },
        __wbg_requestPointerLock_7dbfa94574f241c1: function(arg0) {
            arg0.requestPointerLock();
        },
        __wbg_resolve_9feb5d906ca62419: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_respond_e7e53102735b2ae2: function() { return handleError(function (arg0, arg1) {
            arg0.respond(arg1 >>> 0);
        }, arguments); },
        __wbg_result_2cd4f12832e66239: function() { return handleError(function (arg0) {
            const ret = arg0.result;
            return ret;
        }, arguments); },
        __wbg_run_c9143d3225a408b9: function(arg0, arg1, arg2) {
            try {
                var state0 = {a: arg1, b: arg2};
                var cb0 = () => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen__convert__closures_____invoke__h267ac317aad8c6b8(a, state0.b, );
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = arg0.run(cb0);
                return ret;
            } finally {
                state0.a = 0;
            }
        },
        __wbg_sampleRate_b7f221c5b3d93248: function(arg0) {
            const ret = arg0.sampleRate;
            return ret;
        },
        __wbg_search_2052c2bcc785d890: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.search;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_serviceWorker_79ea5e4ddb36fe0e: function(arg0) {
            const ret = arg0.serviceWorker;
            return ret;
        },
        __wbg_setAttribute_50dcf32d70e1628c: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setInterval_9bc467b74ad0a322: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.setInterval(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_5337f8ac82364a3f: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(arg0, arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_b0d9dc239ecdb765: function(arg0, arg1, arg2) {
            arg0.set(getArrayU8FromWasm0(arg1, arg2));
        },
        __wbg_set_buffer_cd6b31a0a4bb28a4: function(arg0, arg1) {
            arg0.buffer = arg1;
        },
        __wbg_set_fillStyle_a3656c7c5d4ad803: function(arg0, arg1, arg2) {
            arg0.fillStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_font_5b1b8c76449f5864: function(arg0, arg1, arg2) {
            arg0.font = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_innerHTML_faa6730a8fd54513: function(arg0, arg1, arg2) {
            arg0.innerHTML = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_lineWidth_da5d8942373f2ea0: function(arg0, arg1) {
            arg0.lineWidth = arg1;
        },
        __wbg_set_nodeValue_a07ce0a80ebf7431: function(arg0, arg1, arg2) {
            arg0.nodeValue = arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_normalize_2a1164224a92dfd7: function(arg0, arg1) {
            arg0.normalize = arg1 !== 0;
        },
        __wbg_set_onerror_4035c0925a7c14c0: function(arg0, arg1) {
            arg0.onerror = arg1;
        },
        __wbg_set_onsuccess_a2847f90c494c640: function(arg0, arg1) {
            arg0.onsuccess = arg1;
        },
        __wbg_set_onupgradeneeded_c5f3a86916133da4: function(arg0, arg1) {
            arg0.onupgradeneeded = arg1;
        },
        __wbg_set_strokeStyle_cee0bcfd92da6363: function(arg0, arg1, arg2) {
            arg0.strokeStyle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_tabIndex_2ae06d048487bc20: function(arg0, arg1) {
            arg0.tabIndex = arg1;
        },
        __wbg_set_textAlign_2293f6bbd3877cb0: function(arg0, arg1, arg2) {
            arg0.textAlign = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_type_1425359fd8e9769c: function(arg0, arg1) {
            arg0.type = __wbindgen_enum_OscillatorType[arg1];
        },
        __wbg_set_value_78631e9dc5b69626: function(arg0, arg1) {
            arg0.value = arg1;
        },
        __wbg_shiftKey_8896b6760df23dca: function(arg0) {
            const ret = arg0.shiftKey;
            return ret;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_start_bd3a632033118000: function() { return handleError(function (arg0) {
            arg0.start();
        }, arguments); },
        __wbg_static_accessor_CREATE_TASK_9f4644da9615d6a2: function() {
            const ret = typeof console === 'undefined' ? null : console?.createTask;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_1c7f1bd6c6941fdb: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_e039bc914f83e74e: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_8bf8c48c28420ad5: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_6aeee9b51652ee0f: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_stopPropagation_8b2f1c5aac391c21: function(arg0) {
            arg0.stopPropagation();
        },
        __wbg_stop_59ba924a741d57eb: function() { return handleError(function (arg0, arg1) {
            arg0.stop(arg1);
        }, arguments); },
        __wbg_stroke_38f034c148fd63eb: function(arg0) {
            arg0.stroke();
        },
        __wbg_tagName_1392ecc13f557e7b: function(arg0, arg1) {
            const ret = arg1.tagName;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_target_88ed73b611ebed5d: function(arg0) {
            const ret = arg0.target;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_text_de416916b5c06490: function() { return handleError(function (arg0) {
            const ret = arg0.text();
            return ret;
        }, arguments); },
        __wbg_then_20a157d939b514f5: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_then_5ef9b762bc91555c: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_threeBridgeCreateParticles_c5316aa666e6e9cf: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            threeBridgeCreateParticles(getStringFromWasm0(arg0, arg1), arg2 >>> 0, arg3, arg4, arg5, arg6);
        },
        __wbg_threeBridgeInit_25008efedbacab1a: function(arg0) {
            threeBridgeInit(arg0);
        },
        __wbg_threeBridgeRemoveGrass_d75bef418c26eb24: function(arg0, arg1) {
            threeBridgeRemoveGrass(getStringFromWasm0(arg0, arg1));
        },
        __wbg_threeBridgeRemoveMesh_48c7608ee9c51508: function(arg0, arg1) {
            threeBridgeRemoveMesh(getStringFromWasm0(arg0, arg1));
        },
        __wbg_threeBridgeRenderFrame_c9b6c80240de2300: function() {
            threeBridgeRenderFrame();
        },
        __wbg_threeBridgeSetBiome_5587ad70f02e2b00: function(arg0) {
            threeBridgeSetBiome(arg0);
        },
        __wbg_threeBridgeSetCamera_56d1c2c887ec9a86: function(arg0, arg1, arg2, arg3, arg4) {
            threeBridgeSetCamera(arg0, arg1, arg2, arg3, arg4);
        },
        __wbg_threeBridgeSetFade_61e533422dc7aeac: function(arg0) {
            threeBridgeSetFade(arg0);
        },
        __wbg_threeBridgeSetFog_2b5b40568cb663db: function(arg0, arg1, arg2, arg3) {
            threeBridgeSetFog(arg0, arg1, arg2, arg3);
        },
        __wbg_threeBridgeSetMeshFrustumCulled_0c5d80eddf1ee0b9: function(arg0, arg1, arg2) {
            threeBridgeSetMeshFrustumCulled(getStringFromWasm0(arg0, arg1), arg2 !== 0);
        },
        __wbg_threeBridgeSetMeshPosition_0ff072b7061b7242: function(arg0, arg1, arg2, arg3, arg4) {
            threeBridgeSetMeshPosition(getStringFromWasm0(arg0, arg1), arg2, arg3, arg4);
        },
        __wbg_threeBridgeSetMeshRotation_a481e033c81513f0: function(arg0, arg1, arg2, arg3, arg4) {
            threeBridgeSetMeshRotation(getStringFromWasm0(arg0, arg1), arg2, arg3, arg4);
        },
        __wbg_threeBridgeSetMeshVisible_6ce85b3c29ecfa02: function(arg0, arg1, arg2) {
            threeBridgeSetMeshVisible(getStringFromWasm0(arg0, arg1), arg2 !== 0);
        },
        __wbg_threeBridgeSetNightParams_95d990a3b9ed0d77: function(arg0, arg1, arg2, arg3) {
            threeBridgeSetNightParams(arg0, arg1, arg2, arg3);
        },
        __wbg_threeBridgeSetParticlesOpacity_ad6bd1956fee66b6: function(arg0, arg1, arg2) {
            threeBridgeSetParticlesOpacity(getStringFromWasm0(arg0, arg1), arg2);
        },
        __wbg_threeBridgeSetSunLight_a87b089ce5765519: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            threeBridgeSetSunLight(arg0, arg1, arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_threeBridgeSetSunPosition_1873a85031923a6f: function(arg0, arg1, arg2, arg3) {
            threeBridgeSetSunPosition(arg0, arg1, arg2, arg3);
        },
        __wbg_threeBridgeSetUnderwater_673b05ad70578cd8: function(arg0) {
            threeBridgeSetUnderwater(arg0 !== 0);
        },
        __wbg_threeBridgeSetWind_b6d45f9f679a085d: function(arg0, arg1) {
            threeBridgeSetWind(arg0, arg1);
        },
        __wbg_threeBridgeUpdateMeshPositions_3b8f9995b2778bed: function(arg0, arg1, arg2) {
            threeBridgeUpdateMeshPositions(getStringFromWasm0(arg0, arg1), arg2);
        },
        __wbg_threeBridgeUpdateParticles_2b3434cd61de7726: function(arg0, arg1, arg2) {
            threeBridgeUpdateParticles(getStringFromWasm0(arg0, arg1), arg2);
        },
        __wbg_threeBridgeUpdateRemotePlayer_d63ecf73e9244e42: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
            threeBridgeUpdateRemotePlayer(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), arg4, arg5, arg6, arg7, arg8);
        },
        __wbg_threeBridgeUpdateWaterMesh_1d3397640c2b992d: function(arg0, arg1, arg2, arg3, arg4) {
            threeBridgeUpdateWaterMesh(getStringFromWasm0(arg0, arg1), arg2, arg3, arg4);
        },
        __wbg_threeBridgeUploadGrass_74689fb5a1886ce3: function(arg0, arg1, arg2, arg3, arg4) {
            threeBridgeUploadGrass(getStringFromWasm0(arg0, arg1), arg2, arg3 >>> 0, arg4);
        },
        __wbg_threeBridgeUploadMesh_c1e55152b2cff27e: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            threeBridgeUploadMesh(getStringFromWasm0(arg0, arg1), arg2, arg3, arg4, arg5, arg6);
        },
        __wbg_threeBridgeUploadPortalMesh_87cbe8c19f47049d: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            threeBridgeUploadPortalMesh(getStringFromWasm0(arg0, arg1), arg2, arg3, arg4, arg5, arg6 >>> 0, arg7);
        },
        __wbg_threeBridgeUploadSkyMesh_9383db5af831f36a: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            threeBridgeUploadSkyMesh(getStringFromWasm0(arg0, arg1), arg2, arg3, arg4, arg5);
        },
        __wbg_threeBridgeUploadWaterMesh_77376828005b41b7: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            threeBridgeUploadWaterMesh(getStringFromWasm0(arg0, arg1), arg2, arg3, arg4, arg5);
        },
        __wbg_threeBridgeWorkerGenChunk_70bb35414ae0c700: function(arg0, arg1, arg2, arg3, arg4) {
            const ret = threeBridgeWorkerGenChunk(getStringFromWasm0(arg0, arg1), arg2, arg3, arg4 >>> 0);
            return ret;
        },
        __wbg_threeBridgeWorkerGetReady_8c158b7c16796db8: function(arg0) {
            const ret = threeBridgeWorkerGetReady();
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_threeBridgeWorkerInit_e9f82549dda10b58: function() {
            threeBridgeWorkerInit();
        },
        __wbg_threeBridgeWorkerPending_83bfb09eccb23bc5: function() {
            const ret = threeBridgeWorkerPending();
            return ret;
        },
        __wbg_threeBridgeWorkerSetSeed_73b870d5a7bf8f5e: function(arg0) {
            threeBridgeWorkerSetSeed(arg0 >>> 0);
        },
        __wbg_threeBridgeWsConnect_b734de64c0f22369: function(arg0, arg1, arg2, arg3) {
            threeBridgeWsConnect(getStringFromWasm0(arg0, arg1), arg2 >>> 0, arg3);
        },
        __wbg_threeBridgeWsDisconnect_c89ab895d3ff9acb: function() {
            threeBridgeWsDisconnect();
        },
        __wbg_threeBridgeWsSendChat_da53219eaf24b30b: function(arg0, arg1) {
            threeBridgeWsSendChat(getStringFromWasm0(arg0, arg1));
        },
        __wbg_threeBridgeWsSendPos_315247bf6941aff3: function(arg0, arg1, arg2, arg3, arg4) {
            threeBridgeWsSendPos(arg0, arg1, arg2, arg3, arg4);
        },
        __wbg_top_48ee6b46ac920115: function(arg0) {
            const ret = arg0.top;
            return ret;
        },
        __wbg_transaction_0f9063b8d28894ba: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.transaction(getStringFromWasm0(arg1, arg2), __wbindgen_enum_IdbTransactionMode[arg3]);
            return ret;
        }, arguments); },
        __wbg_userAgent_08b9a244999ff008: function() { return handleError(function (arg0, arg1) {
            const ret = arg1.userAgent;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_value_27e4840f678952c0: function() { return handleError(function (arg0) {
            const ret = arg0.value;
            return ret;
        }, arguments); },
        __wbg_value_5aded02f5e9f705d: function(arg0) {
            const ret = arg0.value;
            return ret;
        },
        __wbg_value_81b19d1762b11a96: function(arg0, arg1) {
            const ret = arg1.value;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_view_16bd97d49793e1a9: function(arg0) {
            const ret = arg0.view;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_warn_1f9b94806da61fbb: function(arg0) {
            console.warn(arg0);
        },
        __wbg_warn_80a2693a4aeddbf5: function(arg0, arg1, arg2) {
            console.warn(arg0, arg1, arg2);
        },
        __wbg_width_73079be53f70e8ba: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_width_745cdbb52ce771fd: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_writeText_8da2a080a8f02fcd: function(arg0, arg1, arg2) {
            const ret = arg0.writeText(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 1840, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h99e5e92ab90136f4);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 1954, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h8fffdad006f999cb);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 1803, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hba0f936819cdfcbb);
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 1839, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h3b8febfe60ee5d3f);
            return ret;
        },
        __wbindgen_cast_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("Event")], shim_idx: 967, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hc326e61b6998ff17);
            return ret;
        },
        __wbindgen_cast_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("KeyboardEvent")], shim_idx: 966, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h7911ea61948fbcf1);
            return ret;
        },
        __wbindgen_cast_0000000000000007: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 968, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hed6abad60d56c37b);
            return ret;
        },
        __wbindgen_cast_0000000000000008: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [String], shim_idx: 969, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h012062ebd6e2388d);
            return ret;
        },
        __wbindgen_cast_0000000000000009: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 1802, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h13726a534d84580c);
            return ret;
        },
        __wbindgen_cast_000000000000000a: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 1817, ret: Unit, inner_ret: Some(Unit) }, mutable: false }) -> Externref`.
            const ret = makeClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hb9dc5cd41c6131b0);
            return ret;
        },
        __wbindgen_cast_000000000000000b: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 1838, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h7c3a30fa75f40756);
            return ret;
        },
        __wbindgen_cast_000000000000000c: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(F32)) -> NamedExternref("Float32Array")`.
            const ret = getArrayF32FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_000000000000000d: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
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
        "./worlds-app_bg.js": import0,
    };
}

const lAudioContext = (typeof AudioContext !== 'undefined' ? AudioContext : (typeof webkitAudioContext !== 'undefined' ? webkitAudioContext : undefined));
function wasm_bindgen__convert__closures_____invoke__h13726a534d84580c(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__h13726a534d84580c(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__hb9dc5cd41c6131b0(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__hb9dc5cd41c6131b0(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__h7c3a30fa75f40756(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__h7c3a30fa75f40756(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__h267ac317aad8c6b8(arg0, arg1) {
    const ret = wasm.wasm_bindgen__convert__closures_____invoke__h267ac317aad8c6b8(arg0, arg1);
    return ret !== 0;
}

function wasm_bindgen__convert__closures_____invoke__h99e5e92ab90136f4(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h99e5e92ab90136f4(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__hba0f936819cdfcbb(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__hba0f936819cdfcbb(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h3b8febfe60ee5d3f(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h3b8febfe60ee5d3f(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__hc326e61b6998ff17(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__hc326e61b6998ff17(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h7911ea61948fbcf1(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__h7911ea61948fbcf1(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__hed6abad60d56c37b(arg0, arg1, arg2) {
    wasm.wasm_bindgen__convert__closures_____invoke__hed6abad60d56c37b(arg0, arg1, arg2);
}

function wasm_bindgen__convert__closures_____invoke__h8fffdad006f999cb(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen__convert__closures_____invoke__h8fffdad006f999cb(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen__convert__closures_____invoke__h9885bac28915fc8d(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures_____invoke__h9885bac28915fc8d(arg0, arg1, arg2, arg3);
}

function wasm_bindgen__convert__closures_____invoke__h625988fa5e0206e9(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures_____invoke__h625988fa5e0206e9(arg0, arg1, arg2, arg3);
}

function wasm_bindgen__convert__closures_____invoke__h012062ebd6e2388d(arg0, arg1, arg2) {
    const ptr0 = passStringToWasm0(arg2, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.wasm_bindgen__convert__closures_____invoke__h012062ebd6e2388d(arg0, arg1, ptr0, len0);
}


const __wbindgen_enum_IdbTransactionMode = ["readonly", "readwrite", "versionchange", "readwriteflush", "cleanup"];


const __wbindgen_enum_OscillatorType = ["sine", "square", "sawtooth", "triangle", "custom"];


const __wbindgen_enum_ReadableStreamType = ["bytes"];
const IntoUnderlyingByteSourceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingbytesource_free(ptr, 1));
const IntoUnderlyingSinkFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsink_free(ptr, 1));
const IntoUnderlyingSourceFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_intounderlyingsource_free(ptr, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayF32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let cachedFloat32ArrayMemory0 = null;
function getFloat32ArrayMemory0() {
    if (cachedFloat32ArrayMemory0 === null || cachedFloat32ArrayMemory0.byteLength === 0) {
        cachedFloat32ArrayMemory0 = new Float32Array(wasm.memory.buffer);
    }
    return cachedFloat32ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        try {
            return f(state.a, state.b, ...args);
        } finally {
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedFloat32ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
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
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
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
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('worlds-app_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
