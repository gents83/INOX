/**
 * Chroma subsampling format
 * @enum {0 | 1 | 2 | 3}
 */
export const ChromaSampling = Object.freeze({
    /**
     * Both vertically and horizontally subsampled.
     */
    Cs420: 0, "0": "Cs420",
    /**
     * Horizontally subsampled.
     */
    Cs422: 1, "1": "Cs422",
    /**
     * Not subsampled.
     */
    Cs444: 2, "2": "Cs444",
    /**
     * Monochrome.
     */
    Cs400: 3, "3": "Cs400",
});

/**
 * @param {string} s
 */
export function send_event_from_string(s) {
    const ptr0 = passStringToWasm0(s, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.send_event_from_string(ptr0, len0);
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Window_afcc911b2f9c92e2: function(arg0) {
            const ret = arg0.Window;
            return ret;
        },
        __wbg_WorkerGlobalScope_5d19ebc889ff397e: function(arg0) {
            const ret = arg0.WorkerGlobalScope;
            return ret;
        },
        __wbg___wbindgen_boolean_get_7a12af2b3f899c5a: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_0e68cf47c9cbd9b0: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_function_fcda5e3902d732fe: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_5160b3e381865372: function(arg0) {
            const ret = arg0 === null;
            return ret;
        },
        __wbg___wbindgen_is_string_c4f7cb494a2a21f1: function(arg0) {
            const ret = typeof(arg0) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_8c687d0b90d5b524: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_number_get_1dc732b810cb937c: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_92ab86bb19cbc12f: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_5d9e815e6fdf150f: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_997e73d32238e655: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_activeTexture_79f1f4662ee9ed93: function(arg0, arg1) {
            arg0.activeTexture(arg1 >>> 0);
        },
        __wbg_activeTexture_e6dac82eb23d9409: function(arg0, arg1) {
            arg0.activeTexture(arg1 >>> 0);
        },
        __wbg_addEventListener_9534e2dc12044ee4: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
        }, arguments); },
        __wbg_attachShader_74ecf95efc73ee58: function(arg0, arg1, arg2) {
            arg0.attachShader(arg1, arg2);
        },
        __wbg_attachShader_bfb57af27e3c4a25: function(arg0, arg1, arg2) {
            arg0.attachShader(arg1, arg2);
        },
        __wbg_beginComputePass_431a159006c13c7c: function(arg0, arg1) {
            const ret = arg0.beginComputePass(arg1);
            return ret;
        },
        __wbg_beginQuery_8c123fa7e7aec2a4: function(arg0, arg1, arg2) {
            arg0.beginQuery(arg1 >>> 0, arg2);
        },
        __wbg_beginRenderPass_aa22c432e793359a: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.beginRenderPass(arg1);
            return ret;
        }, arguments); },
        __wbg_bindAttribLocation_601c8f14be6021f1: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.bindAttribLocation(arg1, arg2 >>> 0, getStringFromWasm0(arg3, arg4));
        },
        __wbg_bindAttribLocation_f97028966e63a9a0: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.bindAttribLocation(arg1, arg2 >>> 0, getStringFromWasm0(arg3, arg4));
        },
        __wbg_bindBufferRange_34928ebaef37fa71: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.bindBufferRange(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
        },
        __wbg_bindBuffer_24fab2b14c509d6f: function(arg0, arg1, arg2) {
            arg0.bindBuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindBuffer_f1f33e087b91687c: function(arg0, arg1, arg2) {
            arg0.bindBuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindFramebuffer_6c6a9caf742c2cdb: function(arg0, arg1, arg2) {
            arg0.bindFramebuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindFramebuffer_dc00cdd13fca80a8: function(arg0, arg1, arg2) {
            arg0.bindFramebuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindRenderbuffer_27f21bc7608d07e3: function(arg0, arg1, arg2) {
            arg0.bindRenderbuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindRenderbuffer_a5af5373faa4fb14: function(arg0, arg1, arg2) {
            arg0.bindRenderbuffer(arg1 >>> 0, arg2);
        },
        __wbg_bindSampler_06a184f24b4f473a: function(arg0, arg1, arg2) {
            arg0.bindSampler(arg1 >>> 0, arg2);
        },
        __wbg_bindTexture_7858cc136e95ec24: function(arg0, arg1, arg2) {
            arg0.bindTexture(arg1 >>> 0, arg2);
        },
        __wbg_bindTexture_e8a625bf5d240b0f: function(arg0, arg1, arg2) {
            arg0.bindTexture(arg1 >>> 0, arg2);
        },
        __wbg_bindVertexArrayOES_c947f00135eda668: function(arg0, arg1) {
            arg0.bindVertexArrayOES(arg1);
        },
        __wbg_bindVertexArray_4687bb1e8b327fc9: function(arg0, arg1) {
            arg0.bindVertexArray(arg1);
        },
        __wbg_blendColor_25f3208af2a1a86f: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.blendColor(arg1, arg2, arg3, arg4);
        },
        __wbg_blendColor_7b3873c65a61b8a7: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.blendColor(arg1, arg2, arg3, arg4);
        },
        __wbg_blendEquationSeparate_7255a352fb614ccb: function(arg0, arg1, arg2) {
            arg0.blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_blendEquationSeparate_a63d5c1a833c1137: function(arg0, arg1, arg2) {
            arg0.blendEquationSeparate(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_blendEquation_1636f14afa7d37d0: function(arg0, arg1) {
            arg0.blendEquation(arg1 >>> 0);
        },
        __wbg_blendEquation_9a5ca6936a9438b3: function(arg0, arg1) {
            arg0.blendEquation(arg1 >>> 0);
        },
        __wbg_blendFuncSeparate_1afc51f9c96486b2: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_blendFuncSeparate_222d39ec0d6bb147: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.blendFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_blendFunc_041c4d7dc50ac8c9: function(arg0, arg1, arg2) {
            arg0.blendFunc(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_blendFunc_ad7d043180d4f86a: function(arg0, arg1, arg2) {
            arg0.blendFunc(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_blitFramebuffer_beb217dd67c23446: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
            arg0.blitFramebuffer(arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0);
        },
        __wbg_bufferData_5d0d376bd14d58f3: function(arg0, arg1, arg2, arg3) {
            arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
        },
        __wbg_bufferData_9ce5cb209386d279: function(arg0, arg1, arg2, arg3) {
            arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
        },
        __wbg_bufferData_b58d3ee720ef8b13: function(arg0, arg1, arg2, arg3) {
            arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
        },
        __wbg_bufferData_c074912da5a8e8c2: function(arg0, arg1, arg2, arg3) {
            arg0.bufferData(arg1 >>> 0, arg2, arg3 >>> 0);
        },
        __wbg_bufferSubData_1210904a130da787: function(arg0, arg1, arg2, arg3) {
            arg0.bufferSubData(arg1 >>> 0, arg2, arg3);
        },
        __wbg_bufferSubData_bb3556faa77ce63a: function(arg0, arg1, arg2, arg3) {
            arg0.bufferSubData(arg1 >>> 0, arg2, arg3);
        },
        __wbg_button_f6aa74c65b739064: function(arg0) {
            const ret = arg0.button;
            return ret;
        },
        __wbg_clearBufferfv_5c3233abb46232dd: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.clearBufferfv(arg1 >>> 0, arg2, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_clearBufferiv_a6017e170f127402: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.clearBufferiv(arg1 >>> 0, arg2, getArrayI32FromWasm0(arg3, arg4));
        },
        __wbg_clearBufferuiv_a7ff44fb5a19e886: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.clearBufferuiv(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4));
        },
        __wbg_clearDepth_a4e9a96e0b87fadb: function(arg0, arg1) {
            arg0.clearDepth(arg1);
        },
        __wbg_clearDepth_ea9081c357d405be: function(arg0, arg1) {
            arg0.clearDepth(arg1);
        },
        __wbg_clearStencil_8c03daa5a936cf97: function(arg0, arg1) {
            arg0.clearStencil(arg1);
        },
        __wbg_clearStencil_bd3508b2e4a055c8: function(arg0, arg1) {
            arg0.clearStencil(arg1);
        },
        __wbg_clear_55f5d5ab26fe80b4: function(arg0, arg1) {
            arg0.clear(arg1 >>> 0);
        },
        __wbg_clear_faad852996d5b5ca: function(arg0, arg1) {
            arg0.clear(arg1 >>> 0);
        },
        __wbg_clientWaitSync_1e222800321920a4: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.clientWaitSync(arg1, arg2 >>> 0, arg3 >>> 0);
            return ret;
        },
        __wbg_code_de6c620a1a241f59: function(arg0, arg1) {
            const ret = arg1.code;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_colorMask_dc6ca7d28d90ca79: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
        },
        __wbg_colorMask_ee5503a19adab893: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.colorMask(arg1 !== 0, arg2 !== 0, arg3 !== 0, arg4 !== 0);
        },
        __wbg_compileShader_cabeb4aa8cb42914: function(arg0, arg1) {
            arg0.compileShader(arg1);
        },
        __wbg_compileShader_dada15a718167c7f: function(arg0, arg1) {
            arg0.compileShader(arg1);
        },
        __wbg_compressedTexSubImage2D_08a73ef1b07eadd4: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
            arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8);
        },
        __wbg_compressedTexSubImage2D_26483f478d06e5b6: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
            arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8);
        },
        __wbg_compressedTexSubImage2D_fd0d06f029b00475: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.compressedTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8, arg9);
        },
        __wbg_compressedTexSubImage3D_916c6fa47a2c6ca4: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
            arg0.compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10);
        },
        __wbg_compressedTexSubImage3D_ca08254972ab7956: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.compressedTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10, arg11);
        },
        __wbg_configure_0e4789c0f6b35c8e: function() { return handleError(function (arg0, arg1) {
            arg0.configure(arg1);
        }, arguments); },
        __wbg_constructor_207e5664c25e1af9: function(arg0) {
            const ret = arg0.constructor;
            return ret;
        },
        __wbg_copyBufferSubData_4649cb7afad601b3: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.copyBufferSubData(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
        },
        __wbg_copyBufferToTexture_29b9d26780296819: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            arg0.copyBufferToTexture(arg1, arg2, arg3);
        }, arguments); },
        __wbg_copyTexSubImage2D_b5dc78e744bb31d3: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
            arg0.copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
        },
        __wbg_copyTexSubImage2D_f1d2043ea68078f1: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8) {
            arg0.copyTexSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8);
        },
        __wbg_copyTexSubImage3D_a6674efc4225de77: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.copyTexSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9);
        },
        __wbg_createBindGroupLayout_49a7e2b3d076afcf: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.createBindGroupLayout(arg1);
            return ret;
        }, arguments); },
        __wbg_createBindGroup_655c6e6c0258530e: function(arg0, arg1) {
            const ret = arg0.createBindGroup(arg1);
            return ret;
        },
        __wbg_createBuffer_0726dd2ab09ea1d2: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.createBuffer(arg1);
            return ret;
        }, arguments); },
        __wbg_createBuffer_77a9a8580e5123cc: function(arg0) {
            const ret = arg0.createBuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createBuffer_b4dd77c6c40a2e87: function(arg0) {
            const ret = arg0.createBuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createCommandEncoder_ec1f40f0cb4d09df: function(arg0, arg1) {
            const ret = arg0.createCommandEncoder(arg1);
            return ret;
        },
        __wbg_createComputePipeline_2545c47f08715810: function(arg0, arg1) {
            const ret = arg0.createComputePipeline(arg1);
            return ret;
        },
        __wbg_createFramebuffer_aa440c781a6c7da2: function(arg0) {
            const ret = arg0.createFramebuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createFramebuffer_ac4ca0fc2cfc1bd1: function(arg0) {
            const ret = arg0.createFramebuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createPipelineLayout_2c8cd4528b06c108: function(arg0, arg1) {
            const ret = arg0.createPipelineLayout(arg1);
            return ret;
        },
        __wbg_createProgram_89a2bb21dab7e222: function(arg0) {
            const ret = arg0.createProgram();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createProgram_e43ace4085d1086d: function(arg0) {
            const ret = arg0.createProgram();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createQuery_731629e292b35522: function(arg0) {
            const ret = arg0.createQuery();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createRenderPipeline_cf98d4d699bfb03c: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.createRenderPipeline(arg1);
            return ret;
        }, arguments); },
        __wbg_createRenderbuffer_3605e09d07f5a422: function(arg0) {
            const ret = arg0.createRenderbuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createRenderbuffer_42eeb481e4c92ff2: function(arg0) {
            const ret = arg0.createRenderbuffer();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createSampler_c8ffb3c8d565f704: function(arg0, arg1) {
            const ret = arg0.createSampler(arg1);
            return ret;
        },
        __wbg_createSampler_ebfb34feb26878f1: function(arg0) {
            const ret = arg0.createSampler();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createShaderModule_2e44fc7677c6288b: function(arg0, arg1) {
            const ret = arg0.createShaderModule(arg1);
            return ret;
        },
        __wbg_createShader_4d944e14daceede8: function(arg0, arg1) {
            const ret = arg0.createShader(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createShader_70a84327b207bda4: function(arg0, arg1) {
            const ret = arg0.createShader(arg1 >>> 0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createTexture_1bac74c999b8a48e: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.createTexture(arg1);
            return ret;
        }, arguments); },
        __wbg_createTexture_a409a8c7972e99b3: function(arg0) {
            const ret = arg0.createTexture();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createTexture_ecccbb9df64e8c81: function(arg0) {
            const ret = arg0.createTexture();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createVertexArrayOES_083ec6fc6262613d: function(arg0) {
            const ret = arg0.createVertexArrayOES();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createVertexArray_9411e94d2ee6864e: function(arg0) {
            const ret = arg0.createVertexArray();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_createView_ceaf2f5881adbd34: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.createView(arg1);
            return ret;
        }, arguments); },
        __wbg_cullFace_3e14866a0c370004: function(arg0, arg1) {
            arg0.cullFace(arg1 >>> 0);
        },
        __wbg_cullFace_b64f84debd112644: function(arg0, arg1) {
            arg0.cullFace(arg1 >>> 0);
        },
        __wbg_deleteBuffer_54e287dbecb17d6e: function(arg0, arg1) {
            arg0.deleteBuffer(arg1);
        },
        __wbg_deleteBuffer_c9dc22cdbb93ffbc: function(arg0, arg1) {
            arg0.deleteBuffer(arg1);
        },
        __wbg_deleteFramebuffer_6c3b62109b65e8de: function(arg0, arg1) {
            arg0.deleteFramebuffer(arg1);
        },
        __wbg_deleteFramebuffer_977fae88c7df1c6f: function(arg0, arg1) {
            arg0.deleteFramebuffer(arg1);
        },
        __wbg_deleteProgram_16fe8b000141443c: function(arg0, arg1) {
            arg0.deleteProgram(arg1);
        },
        __wbg_deleteProgram_c990794adb74a326: function(arg0, arg1) {
            arg0.deleteProgram(arg1);
        },
        __wbg_deleteQuery_670219eac170c9fb: function(arg0, arg1) {
            arg0.deleteQuery(arg1);
        },
        __wbg_deleteRenderbuffer_1066bd086097d9c0: function(arg0, arg1) {
            arg0.deleteRenderbuffer(arg1);
        },
        __wbg_deleteRenderbuffer_be22a49b59807a79: function(arg0, arg1) {
            arg0.deleteRenderbuffer(arg1);
        },
        __wbg_deleteSampler_a7f5bf9c8096d80a: function(arg0, arg1) {
            arg0.deleteSampler(arg1);
        },
        __wbg_deleteShader_7169f1760153b284: function(arg0, arg1) {
            arg0.deleteShader(arg1);
        },
        __wbg_deleteShader_9b34ccd212c66de7: function(arg0, arg1) {
            arg0.deleteShader(arg1);
        },
        __wbg_deleteSync_5f19ce0564df8ade: function(arg0, arg1) {
            arg0.deleteSync(arg1);
        },
        __wbg_deleteTexture_4cc3a009d8de3629: function(arg0, arg1) {
            arg0.deleteTexture(arg1);
        },
        __wbg_deleteTexture_5f176c5fb10209e7: function(arg0, arg1) {
            arg0.deleteTexture(arg1);
        },
        __wbg_deleteVertexArrayOES_8ddbff0c3c4bd01b: function(arg0, arg1) {
            arg0.deleteVertexArrayOES(arg1);
        },
        __wbg_deleteVertexArray_a810728c9f325e4c: function(arg0, arg1) {
            arg0.deleteVertexArray(arg1);
        },
        __wbg_depthFunc_e3cb63d0ea760f4f: function(arg0, arg1) {
            arg0.depthFunc(arg1 >>> 0);
        },
        __wbg_depthFunc_fc1a3d2fde41ec92: function(arg0, arg1) {
            arg0.depthFunc(arg1 >>> 0);
        },
        __wbg_depthMask_77fed42fc2117471: function(arg0, arg1) {
            arg0.depthMask(arg1 !== 0);
        },
        __wbg_depthMask_e85b064c1ee5147d: function(arg0, arg1) {
            arg0.depthMask(arg1 !== 0);
        },
        __wbg_depthRange_2b6920405cfb1e3d: function(arg0, arg1, arg2) {
            arg0.depthRange(arg1, arg2);
        },
        __wbg_depthRange_6e341dd878a6597d: function(arg0, arg1, arg2) {
            arg0.depthRange(arg1, arg2);
        },
        __wbg_description_02485704e69b1e7f: function(arg0, arg1) {
            const ret = arg1.description;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_destroy_fe937f756bf8df37: function(arg0) {
            arg0.destroy();
        },
        __wbg_devicePixelRatio_da557bfb3b99fb02: function(arg0) {
            const ret = arg0.devicePixelRatio;
            return ret;
        },
        __wbg_disableVertexAttribArray_45e2fd394cde8686: function(arg0, arg1) {
            arg0.disableVertexAttribArray(arg1 >>> 0);
        },
        __wbg_disableVertexAttribArray_b985896436c82562: function(arg0, arg1) {
            arg0.disableVertexAttribArray(arg1 >>> 0);
        },
        __wbg_disable_6907c683adc38e04: function(arg0, arg1) {
            arg0.disable(arg1 >>> 0);
        },
        __wbg_disable_71ea0d3d0529f100: function(arg0, arg1) {
            arg0.disable(arg1 >>> 0);
        },
        __wbg_dispatchWorkgroups_afb2344298c62227: function(arg0, arg1, arg2, arg3) {
            arg0.dispatchWorkgroups(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0);
        },
        __wbg_document_c7f486c52d63d24e: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_drawArraysInstancedANGLE_1161faf698564d6d: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.drawArraysInstancedANGLE(arg1 >>> 0, arg2, arg3, arg4);
        },
        __wbg_drawArraysInstanced_0ef1fbcdba111c38: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.drawArraysInstanced(arg1 >>> 0, arg2, arg3, arg4);
        },
        __wbg_drawArrays_983f352c9e94d800: function(arg0, arg1, arg2, arg3) {
            arg0.drawArrays(arg1 >>> 0, arg2, arg3);
        },
        __wbg_drawArrays_bc085b67da45ec03: function(arg0, arg1, arg2, arg3) {
            arg0.drawArrays(arg1 >>> 0, arg2, arg3);
        },
        __wbg_drawBuffersWEBGL_6cfb1b5e064e9ed2: function(arg0, arg1) {
            arg0.drawBuffersWEBGL(arg1);
        },
        __wbg_drawBuffers_9b22ea9d8f6a15f0: function(arg0, arg1) {
            arg0.drawBuffers(arg1);
        },
        __wbg_drawElementsInstancedANGLE_da080f7c11a85ead: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawElementsInstancedANGLE(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        },
        __wbg_drawElementsInstanced_ea7d6b71beea79a4: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawElementsInstanced(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        },
        __wbg_drawIndexedIndirect_15e7ca7c20c668f6: function(arg0, arg1, arg2) {
            arg0.drawIndexedIndirect(arg1, arg2);
        },
        __wbg_drawIndexed_d31913e79d58fbac: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.drawIndexed(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5 >>> 0);
        },
        __wbg_draw_6877f98847e1e36c: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.draw(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_enableVertexAttribArray_ad4d8ff487ea8fe1: function(arg0, arg1) {
            arg0.enableVertexAttribArray(arg1 >>> 0);
        },
        __wbg_enableVertexAttribArray_db5eef27f6b32c4a: function(arg0, arg1) {
            arg0.enableVertexAttribArray(arg1 >>> 0);
        },
        __wbg_enable_9fb8e4de75a17155: function(arg0, arg1) {
            arg0.enable(arg1 >>> 0);
        },
        __wbg_enable_9fca6962aca10f48: function(arg0, arg1) {
            arg0.enable(arg1 >>> 0);
        },
        __wbg_endQuery_4eea0267f103b625: function(arg0, arg1) {
            arg0.endQuery(arg1 >>> 0);
        },
        __wbg_end_c36889de8ddef882: function(arg0) {
            arg0.end();
        },
        __wbg_end_f99ebed53d4e198a: function(arg0) {
            arg0.end();
        },
        __wbg_error_622f41c5c32a13db: function(arg0) {
            const ret = arg0.error;
            return ret;
        },
        __wbg_error_6c67c10ae1b96486: function(arg0, arg1) {
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
        __wbg_features_d4d2020319592cbe: function(arg0) {
            const ret = arg0.features;
            return ret;
        },
        __wbg_fenceSync_13999b65854e9b9b: function(arg0, arg1, arg2) {
            const ret = arg0.fenceSync(arg1 >>> 0, arg2 >>> 0);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_finish_0ce019eb398085de: function(arg0) {
            arg0.finish();
        },
        __wbg_finish_4d91de5e927dd13f: function(arg0, arg1) {
            const ret = arg0.finish(arg1);
            return ret;
        },
        __wbg_finish_646a6b6ddc91f563: function(arg0) {
            arg0.finish();
        },
        __wbg_finish_6e06b68ab68cd9f6: function(arg0) {
            const ret = arg0.finish();
            return ret;
        },
        __wbg_flush_824eea12383b8cca: function(arg0) {
            arg0.flush();
        },
        __wbg_flush_c4a6f752e79bfc53: function(arg0) {
            arg0.flush();
        },
        __wbg_framebufferRenderbuffer_aa5275e6a52dc6bd: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4);
        },
        __wbg_framebufferRenderbuffer_f5e60406b8ee912e: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.framebufferRenderbuffer(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4);
        },
        __wbg_framebufferTexture2D_7a85448ff0ab4b2c: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5);
        },
        __wbg_framebufferTexture2D_ee6a0d85fd7083aa: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.framebufferTexture2D(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4, arg5);
        },
        __wbg_framebufferTextureLayer_7c877ec2a0f9220b: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.framebufferTextureLayer(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5);
        },
        __wbg_framebufferTextureMultiviewOVR_64b070ac25c07833: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.framebufferTextureMultiviewOVR(arg1 >>> 0, arg2 >>> 0, arg3, arg4, arg5, arg6);
        },
        __wbg_frontFace_77f09a7664c8cd54: function(arg0, arg1) {
            arg0.frontFace(arg1 >>> 0);
        },
        __wbg_frontFace_b3888c65efdb32fe: function(arg0, arg1) {
            arg0.frontFace(arg1 >>> 0);
        },
        __wbg_getBoundingClientRect_299f40f8641457e6: function(arg0) {
            const ret = arg0.getBoundingClientRect();
            return ret;
        },
        __wbg_getBufferSubData_2c901ce3bd14301e: function(arg0, arg1, arg2, arg3) {
            arg0.getBufferSubData(arg1 >>> 0, arg2, arg3);
        },
        __wbg_getContext_2c75bba1a1c9739b: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2), arg3);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getContext_5ff6bd600503b094: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getContext_db25e9527f2ccc08: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2), arg3);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getContext_e0c05ffee530bdcf: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getContext(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getCurrentTexture_20714d1bd9051cab: function() { return handleError(function (arg0) {
            const ret = arg0.getCurrentTexture();
            return ret;
        }, arguments); },
        __wbg_getElementById_ccc92d66acf76819: function(arg0, arg1, arg2) {
            const ret = arg0.getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getExtension_112db5d69fdfc10f: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getExtension(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getIndexedParameter_d4437eb4ad4a0bfa: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getIndexedParameter(arg1 >>> 0, arg2 >>> 0);
            return ret;
        }, arguments); },
        __wbg_getMappedRange_d0bf3141224111b6: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.getMappedRange(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_getParameter_031885c3618c2d0c: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.getParameter(arg1 >>> 0);
            return ret;
        }, arguments); },
        __wbg_getParameter_7f269432cd46eb73: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.getParameter(arg1 >>> 0);
            return ret;
        }, arguments); },
        __wbg_getProgramInfoLog_0e335b65673afba8: function(arg0, arg1, arg2) {
            const ret = arg1.getProgramInfoLog(arg2);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getProgramInfoLog_82aa11c9d7e8d96e: function(arg0, arg1, arg2) {
            const ret = arg1.getProgramInfoLog(arg2);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getProgramParameter_1a93fd2f4dcc0622: function(arg0, arg1, arg2) {
            const ret = arg0.getProgramParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getProgramParameter_2a5860ab824690ec: function(arg0, arg1, arg2) {
            const ret = arg0.getProgramParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getQueryParameter_145128cfab5de56d: function(arg0, arg1, arg2) {
            const ret = arg0.getQueryParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getRandomValues_a678b7300e8ed57f: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_getShaderInfoLog_416aea280f6fdf84: function(arg0, arg1, arg2) {
            const ret = arg1.getShaderInfoLog(arg2);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getShaderInfoLog_e66ab6619bcd208d: function(arg0, arg1, arg2) {
            const ret = arg1.getShaderInfoLog(arg2);
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_getShaderParameter_b19fb6adcff77066: function(arg0, arg1, arg2) {
            const ret = arg0.getShaderParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getShaderParameter_d542b1412563ae42: function(arg0, arg1, arg2) {
            const ret = arg0.getShaderParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getSupportedExtensions_67cde11dd49cad1a: function(arg0) {
            const ret = arg0.getSupportedExtensions();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getSupportedProfiles_abd53cd63178e1e0: function(arg0) {
            const ret = arg0.getSupportedProfiles();
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getSyncParameter_f4eeca8570f06af5: function(arg0, arg1, arg2) {
            const ret = arg0.getSyncParameter(arg1, arg2 >>> 0);
            return ret;
        },
        __wbg_getUniformBlockIndex_db57fe2d6ed6af2a: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.getUniformBlockIndex(arg1, getStringFromWasm0(arg2, arg3));
            return ret;
        },
        __wbg_getUniformLocation_03b9677a8f868582: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.getUniformLocation(arg1, getStringFromWasm0(arg2, arg3));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getUniformLocation_20bad5ce4a0aaae4: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.getUniformLocation(arg1, getStringFromWasm0(arg2, arg3));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_get_95e4d462165c92ae: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_get_unchecked_363572bdd397d473: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_gpu_2ccc250735d24a2a: function(arg0) {
            const ret = arg0.gpu;
            return ret;
        },
        __wbg_has_0c97053e877f47cc: function(arg0, arg1, arg2) {
            const ret = arg0.has(getStringFromWasm0(arg1, arg2));
            return ret;
        },
        __wbg_height_24586a23e4789d28: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_height_c15ee46a1345e9af: function(arg0) {
            const ret = arg0.height;
            return ret;
        },
        __wbg_includes_c0f578d8314b8dbf: function(arg0, arg1, arg2) {
            const ret = arg0.includes(arg1, arg2);
            return ret;
        },
        __wbg_info_cf0d9a286850cd24: function(arg0) {
            const ret = arg0.info;
            return ret;
        },
        __wbg_innerHeight_f0945dfd6f2d3a87: function() { return handleError(function (arg0) {
            const ret = arg0.innerHeight;
            return ret;
        }, arguments); },
        __wbg_innerWidth_1553ed0783afe5b4: function() { return handleError(function (arg0) {
            const ret = arg0.innerWidth;
            return ret;
        }, arguments); },
        __wbg_instanceof_GpuInternalError_5528115882ef5df4: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUInternalError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuOutOfMemoryError_6429c750997f1c8d: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUOutOfMemoryError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_GpuValidationError_75fa3611f065f4df: function(arg0) {
            let result;
            try {
                result = arg0 instanceof GPUValidationError;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlCanvasElement_4d7e131643d814c1: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLCanvasElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_WebGl2RenderingContext_015d221b904a3c7b: function(arg0) {
            let result;
            try {
                result = arg0 instanceof WebGL2RenderingContext;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_a3b8566f0a9c5d1a: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_invalidateFramebuffer_4538559f02a556ae: function() { return handleError(function (arg0, arg1, arg2) {
            arg0.invalidateFramebuffer(arg1 >>> 0, arg2);
        }, arguments); },
        __wbg_isFallbackAdapter_8ccb967428491dcb: function(arg0) {
            const ret = arg0.isFallbackAdapter;
            return ret;
        },
        __wbg_is_61443cc073056436: function(arg0, arg1) {
            const ret = Object.is(arg0, arg1);
            return ret;
        },
        __wbg_label_7ed42f25f841996b: function(arg0, arg1) {
            const ret = arg1.label;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_length_31bdaf014f5fbde2: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_4e1adc0d42e23620: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_limits_20c6f56636df7d38: function(arg0) {
            const ret = arg0.limits;
            return ret;
        },
        __wbg_linkProgram_794fd290a759542a: function(arg0, arg1) {
            arg0.linkProgram(arg1);
        },
        __wbg_linkProgram_e408ee5f93686f1f: function(arg0, arg1) {
            arg0.linkProgram(arg1);
        },
        __wbg_log_b69f24713c9b6859: function(arg0, arg1) {
            console.log(getStringFromWasm0(arg0, arg1));
        },
        __wbg_mapAsync_52b01fa9e8f765fd: function(arg0, arg1, arg2, arg3) {
            const ret = arg0.mapAsync(arg1 >>> 0, arg2, arg3);
            return ret;
        },
        __wbg_maxBindGroupsPlusVertexBuffers_33e5006b23e20478: function(arg0) {
            const ret = arg0.maxBindGroupsPlusVertexBuffers;
            return ret;
        },
        __wbg_maxBindGroups_f6d26f3a67826666: function(arg0) {
            const ret = arg0.maxBindGroups;
            return ret;
        },
        __wbg_maxBindingsPerBindGroup_edab2e8dabbf6060: function(arg0) {
            const ret = arg0.maxBindingsPerBindGroup;
            return ret;
        },
        __wbg_maxBufferSize_bbc69284c14aa7da: function(arg0) {
            const ret = arg0.maxBufferSize;
            return ret;
        },
        __wbg_maxColorAttachmentBytesPerSample_63ebe4f81de2f34c: function(arg0) {
            const ret = arg0.maxColorAttachmentBytesPerSample;
            return ret;
        },
        __wbg_maxColorAttachments_aed8c38beabf3a5c: function(arg0) {
            const ret = arg0.maxColorAttachments;
            return ret;
        },
        __wbg_maxComputeInvocationsPerWorkgroup_2d964564c37f1c65: function(arg0) {
            const ret = arg0.maxComputeInvocationsPerWorkgroup;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeX_a3e3206570da184f: function(arg0) {
            const ret = arg0.maxComputeWorkgroupSizeX;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeY_dffa4a62244b7563: function(arg0) {
            const ret = arg0.maxComputeWorkgroupSizeY;
            return ret;
        },
        __wbg_maxComputeWorkgroupSizeZ_976ebcb760f6d07d: function(arg0) {
            const ret = arg0.maxComputeWorkgroupSizeZ;
            return ret;
        },
        __wbg_maxComputeWorkgroupStorageSize_2e8dbece6e532e2a: function(arg0) {
            const ret = arg0.maxComputeWorkgroupStorageSize;
            return ret;
        },
        __wbg_maxComputeWorkgroupsPerDimension_bb7d36b4d20c80f4: function(arg0) {
            const ret = arg0.maxComputeWorkgroupsPerDimension;
            return ret;
        },
        __wbg_maxDynamicStorageBuffersPerPipelineLayout_1ca859cb96a414e0: function(arg0) {
            const ret = arg0.maxDynamicStorageBuffersPerPipelineLayout;
            return ret;
        },
        __wbg_maxDynamicUniformBuffersPerPipelineLayout_e968f2c8cd8f4d46: function(arg0) {
            const ret = arg0.maxDynamicUniformBuffersPerPipelineLayout;
            return ret;
        },
        __wbg_maxInterStageShaderVariables_138ac882c4d6a3d3: function(arg0) {
            const ret = arg0.maxInterStageShaderVariables;
            return ret;
        },
        __wbg_maxSampledTexturesPerShaderStage_bb3e6b2698321fa6: function(arg0) {
            const ret = arg0.maxSampledTexturesPerShaderStage;
            return ret;
        },
        __wbg_maxSamplersPerShaderStage_98c00a1829fa414b: function(arg0) {
            const ret = arg0.maxSamplersPerShaderStage;
            return ret;
        },
        __wbg_maxStorageBufferBindingSize_e500e31f479e669e: function(arg0) {
            const ret = arg0.maxStorageBufferBindingSize;
            return ret;
        },
        __wbg_maxStorageBuffersInFragmentStage_2a7e0663edaa7055: function(arg0) {
            const ret = arg0.maxStorageBuffersInFragmentStage;
            return ret;
        },
        __wbg_maxStorageBuffersInVertexStage_781f0e12842424b7: function(arg0) {
            const ret = arg0.maxStorageBuffersInVertexStage;
            return ret;
        },
        __wbg_maxStorageBuffersPerShaderStage_eb663f6d7521b6a7: function(arg0) {
            const ret = arg0.maxStorageBuffersPerShaderStage;
            return ret;
        },
        __wbg_maxStorageTexturesInFragmentStage_5aac711e4cb616b0: function(arg0) {
            const ret = arg0.maxStorageTexturesInFragmentStage;
            return ret;
        },
        __wbg_maxStorageTexturesInVertexStage_5c835176c23c81bd: function(arg0) {
            const ret = arg0.maxStorageTexturesInVertexStage;
            return ret;
        },
        __wbg_maxStorageTexturesPerShaderStage_bb3ad93b53e618c0: function(arg0) {
            const ret = arg0.maxStorageTexturesPerShaderStage;
            return ret;
        },
        __wbg_maxTextureArrayLayers_2a56d05fb261c99a: function(arg0) {
            const ret = arg0.maxTextureArrayLayers;
            return ret;
        },
        __wbg_maxTextureDimension1D_84590c1d4770d319: function(arg0) {
            const ret = arg0.maxTextureDimension1D;
            return ret;
        },
        __wbg_maxTextureDimension2D_7f2b5c8b2727e3fc: function(arg0) {
            const ret = arg0.maxTextureDimension2D;
            return ret;
        },
        __wbg_maxTextureDimension3D_7f3babddf55c32a6: function(arg0) {
            const ret = arg0.maxTextureDimension3D;
            return ret;
        },
        __wbg_maxUniformBufferBindingSize_d80a09e23c0b284c: function(arg0) {
            const ret = arg0.maxUniformBufferBindingSize;
            return ret;
        },
        __wbg_maxUniformBuffersPerShaderStage_0b8b2de676fa740e: function(arg0) {
            const ret = arg0.maxUniformBuffersPerShaderStage;
            return ret;
        },
        __wbg_maxVertexAttributes_a693dd921316649b: function(arg0) {
            const ret = arg0.maxVertexAttributes;
            return ret;
        },
        __wbg_maxVertexBufferArrayStride_f256d91f281076cb: function(arg0) {
            const ret = arg0.maxVertexBufferArrayStride;
            return ret;
        },
        __wbg_maxVertexBuffers_70ab564b25d5ac20: function(arg0) {
            const ret = arg0.maxVertexBuffers;
            return ret;
        },
        __wbg_message_4ada57a3710f1502: function(arg0, arg1) {
            const ret = arg1.message;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_minStorageBufferOffsetAlignment_3248ed00dcdbf79f: function(arg0) {
            const ret = arg0.minStorageBufferOffsetAlignment;
            return ret;
        },
        __wbg_minUniformBufferOffsetAlignment_3b9fa4caae03e903: function(arg0) {
            const ret = arg0.minUniformBufferOffsetAlignment;
            return ret;
        },
        __wbg_name_b4e1ee96e711fbdf: function(arg0) {
            const ret = arg0.name;
            return ret;
        },
        __wbg_navigator_d217ca64c4bbff48: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_navigator_d25c0f071226f233: function(arg0) {
            const ret = arg0.navigator;
            return ret;
        },
        __wbg_new_03132361a3def9f8: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_12e5d807044fbfe3: function() { return handleError(function (arg0, arg1) {
            const ret = new OffscreenCanvas(arg0 >>> 0, arg1 >>> 0);
            return ret;
        }, arguments); },
        __wbg_new_bebc3f4757acf305: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_ffa92086ea89f79c: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_typed_7d4574ab4b8446c8: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_with_byte_offset_and_length_492c969e8b5da8a4: function(arg0, arg1, arg2) {
            const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_now_d1fb6650485d7f3e: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_of_1a3f47170341cefe: function(arg0) {
            const ret = Array.of(arg0);
            return ret;
        },
        __wbg_offsetX_e641d924bec09a05: function(arg0) {
            const ret = arg0.offsetX;
            return ret;
        },
        __wbg_offsetY_0c531091d9e0aa90: function(arg0) {
            const ret = arg0.offsetY;
            return ret;
        },
        __wbg_onSubmittedWorkDone_270d6b5a45520e79: function(arg0) {
            const ret = arg0.onSubmittedWorkDone();
            return ret;
        },
        __wbg_pixelStorei_1bc7980b82e1ba4b: function(arg0, arg1, arg2) {
            arg0.pixelStorei(arg1 >>> 0, arg2);
        },
        __wbg_pixelStorei_d540c25aafa93789: function(arg0, arg1, arg2) {
            arg0.pixelStorei(arg1 >>> 0, arg2);
        },
        __wbg_polygonOffset_a97069c9fe385bff: function(arg0, arg1, arg2) {
            arg0.polygonOffset(arg1, arg2);
        },
        __wbg_polygonOffset_f99887a0cc3d7df4: function(arg0, arg1, arg2) {
            arg0.polygonOffset(arg1, arg2);
        },
        __wbg_prototypesetcall_ae9f5e7459250748: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_push_bfdf956ba476f65b: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_queryCounterEXT_c9fb8ad39c35fdf6: function(arg0, arg1, arg2) {
            arg0.queryCounterEXT(arg1, arg2 >>> 0);
        },
        __wbg_querySelectorAll_fdbf93e11103921d: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.querySelectorAll(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_querySelector_5b1d9ebad373dc9d: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_queueMicrotask_85c90f6987555d65: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_queueMicrotask_f6a1fa10b81d1fc0: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_queue_adce34608fd0c893: function(arg0) {
            const ret = arg0.queue;
            return ret;
        },
        __wbg_readBuffer_1679a4b6dcd78e30: function(arg0, arg1) {
            arg0.readBuffer(arg1 >>> 0);
        },
        __wbg_readPixels_5d75a0ec1e2aab33: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
        }, arguments); },
        __wbg_readPixels_c60bfe6126f65619: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
        }, arguments); },
        __wbg_readPixels_dcde619ca94b6296: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7) {
            arg0.readPixels(arg1, arg2, arg3, arg4, arg5 >>> 0, arg6 >>> 0, arg7);
        }, arguments); },
        __wbg_renderbufferStorageMultisample_5d9012ce50f133e3: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.renderbufferStorageMultisample(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        },
        __wbg_renderbufferStorage_2b50da85624a8b73: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
        },
        __wbg_renderbufferStorage_2eb3d4343d1a1c6e: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.renderbufferStorage(arg1 >>> 0, arg2 >>> 0, arg3, arg4);
        },
        __wbg_requestAdapter_9112bb147d47f963: function(arg0, arg1) {
            const ret = arg0.requestAdapter(arg1);
            return ret;
        },
        __wbg_requestAnimationFrame_4988887658fcee2e: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.requestAnimationFrame(arg1);
            return ret;
        }, arguments); },
        __wbg_requestDevice_ab46d0519ea1cc34: function(arg0, arg1) {
            const ret = arg0.requestDevice(arg1);
            return ret;
        },
        __wbg_resolve_35ec7e0c6af4c82c: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_samplerParameterf_34605394359af7ae: function(arg0, arg1, arg2, arg3) {
            arg0.samplerParameterf(arg1, arg2 >>> 0, arg3);
        },
        __wbg_samplerParameteri_ff1f558c8f43a220: function(arg0, arg1, arg2, arg3) {
            arg0.samplerParameteri(arg1, arg2 >>> 0, arg3);
        },
        __wbg_scissor_c53448a6fa939bfb: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.scissor(arg1, arg2, arg3, arg4);
        },
        __wbg_scissor_d896348c98d0c9ac: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.scissor(arg1, arg2, arg3, arg4);
        },
        __wbg_setAttribute_8c84e9351986b1f6: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.setAttribute(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_setBindGroup_268fd1714fff0ef5: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
        }, arguments); },
        __wbg_setBindGroup_3e4ce136bc833ea1: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.setBindGroup(arg1 >>> 0, arg2, getArrayU32FromWasm0(arg3, arg4), arg5, arg6 >>> 0);
        }, arguments); },
        __wbg_setBindGroup_c22a1b95c0b17f37: function(arg0, arg1, arg2) {
            arg0.setBindGroup(arg1 >>> 0, arg2);
        },
        __wbg_setBindGroup_f0de6cb2c7dbfc2c: function(arg0, arg1, arg2) {
            arg0.setBindGroup(arg1 >>> 0, arg2);
        },
        __wbg_setIndexBuffer_2531a9103450445e: function(arg0, arg1, arg2, arg3) {
            arg0.setIndexBuffer(arg1, __wbindgen_enum_GpuIndexFormat[arg2], arg3);
        },
        __wbg_setIndexBuffer_7f3cf667b4d71566: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.setIndexBuffer(arg1, __wbindgen_enum_GpuIndexFormat[arg2], arg3, arg4);
        },
        __wbg_setPipeline_c41bf46790f27f9e: function(arg0, arg1) {
            arg0.setPipeline(arg1);
        },
        __wbg_setPipeline_d73f019e98c76d2d: function(arg0, arg1) {
            arg0.setPipeline(arg1);
        },
        __wbg_setVertexBuffer_1e448859663dd400: function(arg0, arg1, arg2, arg3) {
            arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3);
        },
        __wbg_setVertexBuffer_7cf533d694e747f3: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.setVertexBuffer(arg1 >>> 0, arg2, arg3, arg4);
        },
        __wbg_set_9cfc0f17d60ff0af: function(arg0, arg1, arg2) {
            arg0.set(arg1, arg2 >>> 0);
        },
        __wbg_set_a377297433dfea63: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(arg0, arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_a_88262a42340d0b1c: function(arg0, arg1) {
            arg0.a = arg1;
        },
        __wbg_set_access_9a5092f05dc45fad: function(arg0, arg1) {
            arg0.access = __wbindgen_enum_GpuStorageTextureAccess[arg1];
        },
        __wbg_set_address_mode_u_9e2695575a219e33: function(arg0, arg1) {
            arg0.addressModeU = __wbindgen_enum_GpuAddressMode[arg1];
        },
        __wbg_set_address_mode_v_f479b2e6cccbcac4: function(arg0, arg1) {
            arg0.addressModeV = __wbindgen_enum_GpuAddressMode[arg1];
        },
        __wbg_set_address_mode_w_46273e153230180d: function(arg0, arg1) {
            arg0.addressModeW = __wbindgen_enum_GpuAddressMode[arg1];
        },
        __wbg_set_alpha_bfd2df62e7bc581b: function(arg0, arg1) {
            arg0.alpha = arg1;
        },
        __wbg_set_alpha_mode_df805952892caa9c: function(arg0, arg1) {
            arg0.alphaMode = __wbindgen_enum_GpuCanvasAlphaMode[arg1];
        },
        __wbg_set_alpha_to_coverage_enabled_8b5dc2b0a225b3b2: function(arg0, arg1) {
            arg0.alphaToCoverageEnabled = arg1 !== 0;
        },
        __wbg_set_array_layer_count_7312f0f31af94e7c: function(arg0, arg1) {
            arg0.arrayLayerCount = arg1 >>> 0;
        },
        __wbg_set_array_stride_f64_27ffaf4fffd74e61: function(arg0, arg1) {
            arg0.arrayStride = arg1;
        },
        __wbg_set_aspect_0d453bca3d012f02: function(arg0, arg1) {
            arg0.aspect = __wbindgen_enum_GpuTextureAspect[arg1];
        },
        __wbg_set_aspect_4962514fe99e68e6: function(arg0, arg1) {
            arg0.aspect = __wbindgen_enum_GpuTextureAspect[arg1];
        },
        __wbg_set_attributes_7537844a7e6dafdc: function(arg0, arg1, arg2) {
            arg0.attributes = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_b_c47befe0af3261eb: function(arg0, arg1) {
            arg0.b = arg1;
        },
        __wbg_set_base_array_layer_f176bb9f1b37b342: function(arg0, arg1) {
            arg0.baseArrayLayer = arg1 >>> 0;
        },
        __wbg_set_base_mip_level_1df145d9f8db32a9: function(arg0, arg1) {
            arg0.baseMipLevel = arg1 >>> 0;
        },
        __wbg_set_beginning_of_pass_write_index_0d4fa06109208ad7: function(arg0, arg1) {
            arg0.beginningOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_beginning_of_pass_write_index_e9f5d016947893bd: function(arg0, arg1) {
            arg0.beginningOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_bind_group_layouts_285e2b2733fc3309: function(arg0, arg1, arg2) {
            arg0.bindGroupLayouts = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_binding_155b0440b4307793: function(arg0, arg1) {
            arg0.binding = arg1 >>> 0;
        },
        __wbg_set_binding_f74df3510792aba1: function(arg0, arg1) {
            arg0.binding = arg1 >>> 0;
        },
        __wbg_set_blend_7493c2066c3e9970: function(arg0, arg1) {
            arg0.blend = arg1;
        },
        __wbg_set_buffer_9c01e3b6d6765ea2: function(arg0, arg1) {
            arg0.buffer = arg1;
        },
        __wbg_set_buffer_c3410572051920ba: function(arg0, arg1) {
            arg0.buffer = arg1;
        },
        __wbg_set_buffer_ef7f75306cf663ed: function(arg0, arg1) {
            arg0.buffer = arg1;
        },
        __wbg_set_buffers_faa74fdd65a24354: function(arg0, arg1, arg2) {
            arg0.buffers = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_bytes_per_row_c54ca96953f35774: function(arg0, arg1) {
            arg0.bytesPerRow = arg1 >>> 0;
        },
        __wbg_set_clear_value_gpu_color_dict_6211425789c76e59: function(arg0, arg1) {
            arg0.clearValue = arg1;
        },
        __wbg_set_code_b4f37f81f45b5b25: function(arg0, arg1, arg2) {
            arg0.code = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_color_83aa977526e88cbb: function(arg0, arg1) {
            arg0.color = arg1;
        },
        __wbg_set_color_attachments_1a79f09cc3078ca3: function(arg0, arg1, arg2) {
            arg0.colorAttachments = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_compare_cd9b62cdb92eb580: function(arg0, arg1) {
            arg0.compare = __wbindgen_enum_GpuCompareFunction[arg1];
        },
        __wbg_set_compare_f36b34abfaa08ccb: function(arg0, arg1) {
            arg0.compare = __wbindgen_enum_GpuCompareFunction[arg1];
        },
        __wbg_set_compute_d0c2d276b6d4b18d: function(arg0, arg1) {
            arg0.compute = arg1;
        },
        __wbg_set_count_069a4eac409bac55: function(arg0, arg1) {
            arg0.count = arg1 >>> 0;
        },
        __wbg_set_cull_mode_fc649853947a3d0c: function(arg0, arg1) {
            arg0.cullMode = __wbindgen_enum_GpuCullMode[arg1];
        },
        __wbg_set_default_queue_bb91dd10fcb87ff5: function(arg0, arg1) {
            arg0.defaultQueue = arg1;
        },
        __wbg_set_depth_bias_clamp_1c0d695df7f092e5: function(arg0, arg1) {
            arg0.depthBiasClamp = arg1;
        },
        __wbg_set_depth_bias_d7cd16096242a657: function(arg0, arg1) {
            arg0.depthBias = arg1;
        },
        __wbg_set_depth_bias_slope_scale_c4e52ec743ef55ba: function(arg0, arg1) {
            arg0.depthBiasSlopeScale = arg1;
        },
        __wbg_set_depth_clear_value_beda3ec5b1a5c43a: function(arg0, arg1) {
            arg0.depthClearValue = arg1;
        },
        __wbg_set_depth_compare_0c8631eb2eae98e3: function(arg0, arg1) {
            arg0.depthCompare = __wbindgen_enum_GpuCompareFunction[arg1];
        },
        __wbg_set_depth_fail_op_668155ae33d3c06f: function(arg0, arg1) {
            arg0.depthFailOp = __wbindgen_enum_GpuStencilOperation[arg1];
        },
        __wbg_set_depth_load_op_511c513eab4e56a9: function(arg0, arg1) {
            arg0.depthLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
        },
        __wbg_set_depth_or_array_layers_89371305ed0bd962: function(arg0, arg1) {
            arg0.depthOrArrayLayers = arg1 >>> 0;
        },
        __wbg_set_depth_read_only_7f41a74741c144ec: function(arg0, arg1) {
            arg0.depthReadOnly = arg1 !== 0;
        },
        __wbg_set_depth_stencil_97506c7bea4f53da: function(arg0, arg1) {
            arg0.depthStencil = arg1;
        },
        __wbg_set_depth_stencil_attachment_73b79e8b4e948222: function(arg0, arg1) {
            arg0.depthStencilAttachment = arg1;
        },
        __wbg_set_depth_store_op_c89f33b39b43361c: function(arg0, arg1) {
            arg0.depthStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
        },
        __wbg_set_depth_write_enabled_ce89750042940350: function(arg0, arg1) {
            arg0.depthWriteEnabled = arg1 !== 0;
        },
        __wbg_set_device_e275d1d4f3c9eb74: function(arg0, arg1) {
            arg0.device = arg1;
        },
        __wbg_set_dimension_868eee80f4b90011: function(arg0, arg1) {
            arg0.dimension = __wbindgen_enum_GpuTextureDimension[arg1];
        },
        __wbg_set_dimension_e325282e613ca0a4: function(arg0, arg1) {
            arg0.dimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
        },
        __wbg_set_dst_factor_ec7407f19be1aff9: function(arg0, arg1) {
            arg0.dstFactor = __wbindgen_enum_GpuBlendFactor[arg1];
        },
        __wbg_set_end_of_pass_write_index_0d546e46b86ea069: function(arg0, arg1) {
            arg0.endOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_end_of_pass_write_index_49a6ddbb2b888bfa: function(arg0, arg1) {
            arg0.endOfPassWriteIndex = arg1 >>> 0;
        },
        __wbg_set_entries_86a29dd6291c95e7: function(arg0, arg1, arg2) {
            arg0.entries = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_entries_a12aca1e458b0456: function(arg0, arg1, arg2) {
            arg0.entries = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_entry_point_207540f042015ce5: function(arg0, arg1, arg2) {
            arg0.entryPoint = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_entry_point_5f26aacbe4c545eb: function(arg0, arg1, arg2) {
            arg0.entryPoint = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_entry_point_e87e79251dd3144f: function(arg0, arg1, arg2) {
            arg0.entryPoint = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_external_texture_386483d8dd82ab56: function(arg0, arg1) {
            arg0.externalTexture = arg1;
        },
        __wbg_set_fail_op_92f716dbc88b6973: function(arg0, arg1) {
            arg0.failOp = __wbindgen_enum_GpuStencilOperation[arg1];
        },
        __wbg_set_format_1fcaa7d60546b490: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_2c1414a817c213f8: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_533f9ffa7eef563d: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_5d2f25cc93654ecc: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuVertexFormat[arg1];
        },
        __wbg_set_format_5ff53724ed6cedf2: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_815efd4dc4817bbb: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_format_e52bdcca880d2c8e: function(arg0, arg1) {
            arg0.format = __wbindgen_enum_GpuTextureFormat[arg1];
        },
        __wbg_set_fragment_8b780f00a0b0e6f3: function(arg0, arg1) {
            arg0.fragment = arg1;
        },
        __wbg_set_front_face_28ffdf524eedce5b: function(arg0, arg1) {
            arg0.frontFace = __wbindgen_enum_GpuFrontFace[arg1];
        },
        __wbg_set_g_5983abfc46e0cf4e: function(arg0, arg1) {
            arg0.g = arg1;
        },
        __wbg_set_has_dynamic_offset_62bc230bdb7c54d0: function(arg0, arg1) {
            arg0.hasDynamicOffset = arg1 !== 0;
        },
        __wbg_set_height_1202ad0eab43cbe0: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_height_14335c4047cf9c1b: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_height_698fb3b255bc1348: function(arg0, arg1) {
            arg0.height = arg1 >>> 0;
        },
        __wbg_set_label_05ad8ec46a1a7f41: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_08d9be3e4719c226: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_17eb9fe3a02f62b0: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_48e6b787d256f621: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_547d0d4aec39fbe9: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_5ee7427342869829: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_60ad96c811e0d109: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_6db0393d3fdc90a5: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_72bb4f41ef0cb893: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_79387decda299036: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_9556af8b5cda3c9d: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_d010f237b26f2c55: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_e16e2dbe51349c7f: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_e3944e54881b8c50: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_e922700240417ab5: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_label_ef44793ddf4455c5: function(arg0, arg1, arg2) {
            arg0.label = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_layout_41021cfd2a2f62df: function(arg0, arg1) {
            arg0.layout = arg1;
        },
        __wbg_set_layout_50ab727f44b38f26: function(arg0, arg1) {
            arg0.layout = arg1;
        },
        __wbg_set_layout_913d53c17194c989: function(arg0, arg1) {
            arg0.layout = arg1;
        },
        __wbg_set_layout_gpu_auto_layout_mode_0a5a185b3d52b726: function(arg0, arg1) {
            arg0.layout = __wbindgen_enum_GpuAutoLayoutMode[arg1];
        },
        __wbg_set_layout_gpu_auto_layout_mode_aeba193938b47882: function(arg0, arg1) {
            arg0.layout = __wbindgen_enum_GpuAutoLayoutMode[arg1];
        },
        __wbg_set_load_op_99661da6c4eab9b0: function(arg0, arg1) {
            arg0.loadOp = __wbindgen_enum_GpuLoadOp[arg1];
        },
        __wbg_set_lod_max_clamp_dd2d9f9f052f4f44: function(arg0, arg1) {
            arg0.lodMaxClamp = arg1;
        },
        __wbg_set_lod_min_clamp_6d20c97916baeb93: function(arg0, arg1) {
            arg0.lodMinClamp = arg1;
        },
        __wbg_set_mag_filter_b5adebc99cb938e1: function(arg0, arg1) {
            arg0.magFilter = __wbindgen_enum_GpuFilterMode[arg1];
        },
        __wbg_set_mapped_at_creation_81b586dc90a50347: function(arg0, arg1) {
            arg0.mappedAtCreation = arg1 !== 0;
        },
        __wbg_set_mask_70a8a59ce09e5997: function(arg0, arg1) {
            arg0.mask = arg1 >>> 0;
        },
        __wbg_set_max_anisotropy_2beada0e2db62c45: function(arg0, arg1) {
            arg0.maxAnisotropy = arg1;
        },
        __wbg_set_min_binding_size_f64_5005a6904cdf43da: function(arg0, arg1) {
            arg0.minBindingSize = arg1;
        },
        __wbg_set_min_filter_c72f17375e135f0a: function(arg0, arg1) {
            arg0.minFilter = __wbindgen_enum_GpuFilterMode[arg1];
        },
        __wbg_set_mip_level_count_534caaa7e68e68b8: function(arg0, arg1) {
            arg0.mipLevelCount = arg1 >>> 0;
        },
        __wbg_set_mip_level_count_776c8c218b65bc08: function(arg0, arg1) {
            arg0.mipLevelCount = arg1 >>> 0;
        },
        __wbg_set_mip_level_f7ac79e8c54f59ad: function(arg0, arg1) {
            arg0.mipLevel = arg1 >>> 0;
        },
        __wbg_set_mipmap_filter_5bf66195a3639700: function(arg0, arg1) {
            arg0.mipmapFilter = __wbindgen_enum_GpuMipmapFilterMode[arg1];
        },
        __wbg_set_mode_9990b3393ba469ae: function(arg0, arg1) {
            arg0.mode = __wbindgen_enum_GpuCanvasToneMappingMode[arg1];
        },
        __wbg_set_module_5db9b76ee2dd2a59: function(arg0, arg1) {
            arg0.module = arg1;
        },
        __wbg_set_module_d0e2098713606cae: function(arg0, arg1) {
            arg0.module = arg1;
        },
        __wbg_set_module_f02e076ca7e7daf8: function(arg0, arg1) {
            arg0.module = arg1;
        },
        __wbg_set_multisample_37ddafe88b5cd466: function(arg0, arg1) {
            arg0.multisample = arg1;
        },
        __wbg_set_multisampled_7913fd7183272840: function(arg0, arg1) {
            arg0.multisampled = arg1 !== 0;
        },
        __wbg_set_offset_f64_28c24dc15000932e: function(arg0, arg1) {
            arg0.offset = arg1;
        },
        __wbg_set_offset_f64_89f0ce01a689839e: function(arg0, arg1) {
            arg0.offset = arg1;
        },
        __wbg_set_offset_f64_b562d1367e34ef93: function(arg0, arg1) {
            arg0.offset = arg1;
        },
        __wbg_set_onuncapturederror_c8a77eb8695205a0: function(arg0, arg1) {
            arg0.onuncapturederror = arg1;
        },
        __wbg_set_operation_62ce44e1728c4047: function(arg0, arg1) {
            arg0.operation = __wbindgen_enum_GpuBlendOperation[arg1];
        },
        __wbg_set_origin_gpu_origin_3d_dict_631c04520718091f: function(arg0, arg1) {
            arg0.origin = arg1;
        },
        __wbg_set_pass_op_cf02fa088d6352a7: function(arg0, arg1) {
            arg0.passOp = __wbindgen_enum_GpuStencilOperation[arg1];
        },
        __wbg_set_power_preference_8fdca0b7af640d49: function(arg0, arg1) {
            arg0.powerPreference = __wbindgen_enum_GpuPowerPreference[arg1];
        },
        __wbg_set_primitive_43c23761a55b4088: function(arg0, arg1) {
            arg0.primitive = arg1;
        },
        __wbg_set_query_set_41de86d2401aee04: function(arg0, arg1) {
            arg0.querySet = arg1;
        },
        __wbg_set_query_set_4889dd944d5ec0fd: function(arg0, arg1) {
            arg0.querySet = arg1;
        },
        __wbg_set_r_c6f4c68f4804d655: function(arg0, arg1) {
            arg0.r = arg1;
        },
        __wbg_set_required_features_1baf274a8669db60: function(arg0, arg1, arg2) {
            arg0.requiredFeatures = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_required_limits_871ed33c68613dcb: function(arg0, arg1) {
            arg0.requiredLimits = arg1;
        },
        __wbg_set_resolve_target_gpu_texture_view_b19a4f2debf79b96: function(arg0, arg1) {
            arg0.resolveTarget = arg1;
        },
        __wbg_set_resource_5ae7b5e67924f234: function(arg0, arg1) {
            arg0.resource = arg1;
        },
        __wbg_set_resource_gpu_buffer_binding_e5dbca063e7cb67b: function(arg0, arg1) {
            arg0.resource = arg1;
        },
        __wbg_set_resource_gpu_external_texture_6db3b033e81e42e5: function(arg0, arg1) {
            arg0.resource = arg1;
        },
        __wbg_set_resource_gpu_texture_view_eb46c355d51ad7e5: function(arg0, arg1) {
            arg0.resource = arg1;
        },
        __wbg_set_rows_per_image_5011f97318ee71af: function(arg0, arg1) {
            arg0.rowsPerImage = arg1 >>> 0;
        },
        __wbg_set_sample_count_eb86a8b18545b54f: function(arg0, arg1) {
            arg0.sampleCount = arg1 >>> 0;
        },
        __wbg_set_sample_type_c32e1dfff94e63eb: function(arg0, arg1) {
            arg0.sampleType = __wbindgen_enum_GpuTextureSampleType[arg1];
        },
        __wbg_set_sampler_c0e1258543a33bce: function(arg0, arg1) {
            arg0.sampler = arg1;
        },
        __wbg_set_shader_location_7e1832a74f912217: function(arg0, arg1) {
            arg0.shaderLocation = arg1 >>> 0;
        },
        __wbg_set_size_f64_6bcd40704bf4cfdc: function(arg0, arg1) {
            arg0.size = arg1;
        },
        __wbg_set_size_f64_8b8f6bba5d678162: function(arg0, arg1) {
            arg0.size = arg1;
        },
        __wbg_set_size_gpu_extent_3d_dict_7e42e1c98fa36434: function(arg0, arg1) {
            arg0.size = arg1;
        },
        __wbg_set_src_factor_9bfe84af9b7b5cac: function(arg0, arg1) {
            arg0.srcFactor = __wbindgen_enum_GpuBlendFactor[arg1];
        },
        __wbg_set_stencil_back_85b22f1db5b1940a: function(arg0, arg1) {
            arg0.stencilBack = arg1;
        },
        __wbg_set_stencil_clear_value_42be608809151e2a: function(arg0, arg1) {
            arg0.stencilClearValue = arg1 >>> 0;
        },
        __wbg_set_stencil_front_525526164a798a44: function(arg0, arg1) {
            arg0.stencilFront = arg1;
        },
        __wbg_set_stencil_load_op_31838c036993098a: function(arg0, arg1) {
            arg0.stencilLoadOp = __wbindgen_enum_GpuLoadOp[arg1];
        },
        __wbg_set_stencil_read_mask_5cc26495e8b3ae82: function(arg0, arg1) {
            arg0.stencilReadMask = arg1 >>> 0;
        },
        __wbg_set_stencil_read_only_bf1d0c1897e25c62: function(arg0, arg1) {
            arg0.stencilReadOnly = arg1 !== 0;
        },
        __wbg_set_stencil_store_op_e6be1cbc3a8fc210: function(arg0, arg1) {
            arg0.stencilStoreOp = __wbindgen_enum_GpuStoreOp[arg1];
        },
        __wbg_set_stencil_write_mask_d9cb40ec4b4bee5b: function(arg0, arg1) {
            arg0.stencilWriteMask = arg1 >>> 0;
        },
        __wbg_set_step_mode_a97bb24714da41a9: function(arg0, arg1) {
            arg0.stepMode = __wbindgen_enum_GpuVertexStepMode[arg1];
        },
        __wbg_set_storage_texture_939a097db4b18bd4: function(arg0, arg1) {
            arg0.storageTexture = arg1;
        },
        __wbg_set_store_op_b5fdf672436f13f3: function(arg0, arg1) {
            arg0.storeOp = __wbindgen_enum_GpuStoreOp[arg1];
        },
        __wbg_set_strip_index_format_9f787be6c5fc9e87: function(arg0, arg1) {
            arg0.stripIndexFormat = __wbindgen_enum_GpuIndexFormat[arg1];
        },
        __wbg_set_swizzle_a58dec0c3a585545: function(arg0, arg1, arg2) {
            arg0.swizzle = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_targets_41d06f521a7427e5: function(arg0, arg1, arg2) {
            arg0.targets = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_texture_1f64653a5d2d7b4d: function(arg0, arg1) {
            arg0.texture = arg1;
        },
        __wbg_set_texture_9dcedde1bb31eda6: function(arg0, arg1) {
            arg0.texture = arg1;
        },
        __wbg_set_timestamp_writes_59c2d19ed8aecd97: function(arg0, arg1) {
            arg0.timestampWrites = arg1;
        },
        __wbg_set_timestamp_writes_98bed1a8bbc6682d: function(arg0, arg1) {
            arg0.timestampWrites = arg1;
        },
        __wbg_set_tone_mapping_b3464f1baa4cff92: function(arg0, arg1) {
            arg0.toneMapping = arg1;
        },
        __wbg_set_topology_da25f2cc5af203d2: function(arg0, arg1) {
            arg0.topology = __wbindgen_enum_GpuPrimitiveTopology[arg1];
        },
        __wbg_set_type_ccf8472d40abcddf: function(arg0, arg1) {
            arg0.type = __wbindgen_enum_GpuSamplerBindingType[arg1];
        },
        __wbg_set_type_d09829f59932a0fc: function(arg0, arg1) {
            arg0.type = __wbindgen_enum_GpuBufferBindingType[arg1];
        },
        __wbg_set_unclipped_depth_04524a2b44e1e3c1: function(arg0, arg1) {
            arg0.unclippedDepth = arg1 !== 0;
        },
        __wbg_set_usage_a137f82ca163b0a9: function(arg0, arg1) {
            arg0.usage = arg1 >>> 0;
        },
        __wbg_set_usage_b2a2935f37bf3d08: function(arg0, arg1) {
            arg0.usage = arg1 >>> 0;
        },
        __wbg_set_usage_ba5b0f8b333ab325: function(arg0, arg1) {
            arg0.usage = arg1 >>> 0;
        },
        __wbg_set_usage_ddd42599bbba7779: function(arg0, arg1) {
            arg0.usage = arg1 >>> 0;
        },
        __wbg_set_vertex_0be5d146f9ff6f36: function(arg0, arg1) {
            arg0.vertex = arg1;
        },
        __wbg_set_view_dimension_0df554032f1f3a85: function(arg0, arg1) {
            arg0.viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
        },
        __wbg_set_view_dimension_4818d4c18ce5815e: function(arg0, arg1) {
            arg0.viewDimension = __wbindgen_enum_GpuTextureViewDimension[arg1];
        },
        __wbg_set_view_formats_4347dc8363331086: function(arg0, arg1, arg2) {
            arg0.viewFormats = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_view_formats_5797d2fff3c11808: function(arg0, arg1, arg2) {
            arg0.viewFormats = getArrayJsValueViewFromWasm0(arg1, arg2);
        },
        __wbg_set_view_gpu_texture_view_9b2d86b6b99d9fd9: function(arg0, arg1) {
            arg0.view = arg1;
        },
        __wbg_set_view_gpu_texture_view_c0f35f8857c25206: function(arg0, arg1) {
            arg0.view = arg1;
        },
        __wbg_set_visibility_9570b037224c4cc2: function(arg0, arg1) {
            arg0.visibility = arg1 >>> 0;
        },
        __wbg_set_width_4c3a2252e0dea033: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_set_width_9f685402c2cbee70: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_set_width_f52a20e39808b138: function(arg0, arg1) {
            arg0.width = arg1 >>> 0;
        },
        __wbg_set_write_mask_d45279e56abbfcb5: function(arg0, arg1) {
            arg0.writeMask = arg1 >>> 0;
        },
        __wbg_set_x_876d592971db129a: function(arg0, arg1) {
            arg0.x = arg1 >>> 0;
        },
        __wbg_set_y_2b1f5ac0dd5586a5: function(arg0, arg1) {
            arg0.y = arg1 >>> 0;
        },
        __wbg_set_z_ef005d82bc9d24e3: function(arg0, arg1) {
            arg0.z = arg1 >>> 0;
        },
        __wbg_shaderSource_108c27c8694ef4c0: function(arg0, arg1, arg2, arg3) {
            arg0.shaderSource(arg1, getStringFromWasm0(arg2, arg3));
        },
        __wbg_shaderSource_82a2cbfe223a86e0: function(arg0, arg1, arg2, arg3) {
            arg0.shaderSource(arg1, getStringFromWasm0(arg2, arg3));
        },
        __wbg_stack_486e69e44029849f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_8eb4cd83130a11a0: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_1e7044f654e934db: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_d8b50611246a6d92: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_fd0bc376bf0f8b42: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_stencilFuncSeparate_d7b5956ce3fe1272: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
        },
        __wbg_stencilFuncSeparate_fd17ac8961fd8359: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.stencilFuncSeparate(arg1 >>> 0, arg2 >>> 0, arg3, arg4 >>> 0);
        },
        __wbg_stencilMaskSeparate_5f3823dfe31289be: function(arg0, arg1, arg2) {
            arg0.stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_stencilMaskSeparate_d0910e6d55b4605a: function(arg0, arg1, arg2) {
            arg0.stencilMaskSeparate(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_stencilMask_4a8c4bda850006f1: function(arg0, arg1) {
            arg0.stencilMask(arg1 >>> 0);
        },
        __wbg_stencilMask_cb69051e26a265bb: function(arg0, arg1) {
            arg0.stencilMask(arg1 >>> 0);
        },
        __wbg_stencilOpSeparate_3fe9feaf978f0a17: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_stencilOpSeparate_cb0bb47587f022de: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.stencilOpSeparate(arg1 >>> 0, arg2 >>> 0, arg3 >>> 0, arg4 >>> 0);
        },
        __wbg_subgroupMaxSize_1527c5f7a8fe91bb: function(arg0) {
            const ret = arg0.subgroupMaxSize;
            return ret;
        },
        __wbg_subgroupMinSize_d6c5ad4bddc828e9: function(arg0) {
            const ret = arg0.subgroupMinSize;
            return ret;
        },
        __wbg_submit_ce44115121cd166c: function(arg0, arg1, arg2) {
            arg0.submit(getArrayJsValueViewFromWasm0(arg1, arg2));
        },
        __wbg_texImage2D_1b8d273f15622f0e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texImage2D_6fca8c58ea7e4c8e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texImage2D_8c5d6403fa51874d: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texImage3D_2bba05e03e9def07: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
            arg0.texImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8 >>> 0, arg9 >>> 0, arg10);
        }, arguments); },
        __wbg_texImage3D_3977c422ce82cf99: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10) {
            arg0.texImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8 >>> 0, arg9 >>> 0, arg10);
        }, arguments); },
        __wbg_texParameteri_1fea50207adbfd68: function(arg0, arg1, arg2, arg3) {
            arg0.texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
        },
        __wbg_texParameteri_e127aa385b821d68: function(arg0, arg1, arg2, arg3) {
            arg0.texParameteri(arg1 >>> 0, arg2 >>> 0, arg3);
        },
        __wbg_texStorage2D_eb27b3383d99c349: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.texStorage2D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        },
        __wbg_texStorage3D_cb53309842f2c8eb: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.texStorage3D(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5, arg6);
        },
        __wbg_texSubImage2D_0542ae723245dd04: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_0721d9d910ecf097: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_11c39463147c9d38: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_21b4316b692aa7bc: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_339324fe86d9879e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_6218009ef1e27c0e: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_79b4ac26a5580960: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_892cfa51f196ee93: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage2D_9b087860a7b15d0a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9) {
            arg0.texSubImage2D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7 >>> 0, arg8 >>> 0, arg9);
        }, arguments); },
        __wbg_texSubImage3D_6458575ddb6df6b3: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_747bc81ed2ed4116: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_7a674dd9403f8be3: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_805532d56bdd814d: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_ad76ff2925daf174: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_c2995dfe63f848dd: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_c65a616189ed3b19: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_texSubImage3D_f99458987d5b061a: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10, arg11) {
            arg0.texSubImage3D(arg1 >>> 0, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9 >>> 0, arg10 >>> 0, arg11);
        }, arguments); },
        __wbg_then_7a850dae4493f353: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_then_b830475380919203: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_toString_794a30e85c994f2a: function(arg0) {
            const ret = arg0.toString();
            return ret;
        },
        __wbg_unconfigure_0a07a0a40de8988d: function(arg0) {
            arg0.unconfigure();
        },
        __wbg_uniform1f_6067a9c69ed38c8b: function(arg0, arg1, arg2) {
            arg0.uniform1f(arg1, arg2);
        },
        __wbg_uniform1f_d08261cd87e412e0: function(arg0, arg1, arg2) {
            arg0.uniform1f(arg1, arg2);
        },
        __wbg_uniform1i_25a3df9f69c2f765: function(arg0, arg1, arg2) {
            arg0.uniform1i(arg1, arg2);
        },
        __wbg_uniform1i_89c631d199860f60: function(arg0, arg1, arg2) {
            arg0.uniform1i(arg1, arg2);
        },
        __wbg_uniform1ui_58cbbe1a93e44a02: function(arg0, arg1, arg2) {
            arg0.uniform1ui(arg1, arg2 >>> 0);
        },
        __wbg_uniform2fv_6a494ce09154afdb: function(arg0, arg1, arg2, arg3) {
            arg0.uniform2fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform2fv_f85b3b2779a6c2b9: function(arg0, arg1, arg2, arg3) {
            arg0.uniform2fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform2iv_5c591a2edfd26c45: function(arg0, arg1, arg2, arg3) {
            arg0.uniform2iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform2iv_63a2ba7206eaf2b3: function(arg0, arg1, arg2, arg3) {
            arg0.uniform2iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform2uiv_aaa076c2f50b5c86: function(arg0, arg1, arg2, arg3) {
            arg0.uniform2uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
        },
        __wbg_uniform3fv_2e139d144894f1ec: function(arg0, arg1, arg2, arg3) {
            arg0.uniform3fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform3fv_dfaba6b65cfc2c2b: function(arg0, arg1, arg2, arg3) {
            arg0.uniform3fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform3iv_700228c611ca323d: function(arg0, arg1, arg2, arg3) {
            arg0.uniform3iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform3iv_f32512d7179442f8: function(arg0, arg1, arg2, arg3) {
            arg0.uniform3iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform3uiv_c972de1ee5eb2b48: function(arg0, arg1, arg2, arg3) {
            arg0.uniform3uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
        },
        __wbg_uniform4f_b5ea01e0388c425e: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.uniform4f(arg1, arg2, arg3, arg4, arg5);
        },
        __wbg_uniform4f_b9880b3e743b1cf3: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.uniform4f(arg1, arg2, arg3, arg4, arg5);
        },
        __wbg_uniform4fv_0763a5cfb3d561c9: function(arg0, arg1, arg2, arg3) {
            arg0.uniform4fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform4fv_95b1afba56301916: function(arg0, arg1, arg2, arg3) {
            arg0.uniform4fv(arg1, getArrayF32FromWasm0(arg2, arg3));
        },
        __wbg_uniform4iv_1269cd3a3f01909d: function(arg0, arg1, arg2, arg3) {
            arg0.uniform4iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform4iv_84dc6c0b2ea09256: function(arg0, arg1, arg2, arg3) {
            arg0.uniform4iv(arg1, getArrayI32FromWasm0(arg2, arg3));
        },
        __wbg_uniform4uiv_13ca9c250b538d85: function(arg0, arg1, arg2, arg3) {
            arg0.uniform4uiv(arg1, getArrayU32FromWasm0(arg2, arg3));
        },
        __wbg_uniformBlockBinding_d97dbfc1886608c9: function(arg0, arg1, arg2, arg3) {
            arg0.uniformBlockBinding(arg1, arg2 >>> 0, arg3 >>> 0);
        },
        __wbg_uniformMatrix2fv_4a95dd9315ceafce: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix2fv_554c771038fd18e7: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix2x3fv_18f2ff07b55f7458: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix2x3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix2x4fv_2b66cdeb52233873: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix2x4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix3fv_303d7eef53df7ed2: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix3fv_a9db97f34b3706b3: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix3x2fv_67c65614f0e122e8: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix3x2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix3x4fv_e3bd46b1bf5227ed: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix3x4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix4fv_db6f9ad4f00e3b76: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix4fv_e411faf4de6021c5: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix4fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix4x2fv_c59133281828fe8e: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix4x2fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_uniformMatrix4x3fv_dc85828b88505234: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.uniformMatrix4x3fv(arg1, arg2 !== 0, getArrayF32FromWasm0(arg3, arg4));
        },
        __wbg_unmap_adaf93276fdf9aaf: function(arg0) {
            arg0.unmap();
        },
        __wbg_useProgram_7019ac9ef4bdbe96: function(arg0, arg1) {
            arg0.useProgram(arg1);
        },
        __wbg_useProgram_fcab4319331e0094: function(arg0, arg1) {
            arg0.useProgram(arg1);
        },
        __wbg_valueOf_24fc886d85bf55b5: function(arg0) {
            const ret = arg0.valueOf();
            return ret;
        },
        __wbg_vertexAttribDivisorANGLE_5bebd4490e25d0f4: function(arg0, arg1, arg2) {
            arg0.vertexAttribDivisorANGLE(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_vertexAttribDivisor_e2a9ba8c397af61b: function(arg0, arg1, arg2) {
            arg0.vertexAttribDivisor(arg1 >>> 0, arg2 >>> 0);
        },
        __wbg_vertexAttribIPointer_660e209bed7575ab: function(arg0, arg1, arg2, arg3, arg4, arg5) {
            arg0.vertexAttribIPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4, arg5);
        },
        __wbg_vertexAttribPointer_b6db7a62b9bda090: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
        },
        __wbg_vertexAttribPointer_ed019b681b1aaacf: function(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.vertexAttribPointer(arg1 >>> 0, arg2, arg3 >>> 0, arg4 !== 0, arg5, arg6);
        },
        __wbg_viewport_704a6159fb0395cb: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.viewport(arg1, arg2, arg3, arg4);
        },
        __wbg_viewport_f9d3214251b38c39: function(arg0, arg1, arg2, arg3, arg4) {
            arg0.viewport(arg1, arg2, arg3, arg4);
        },
        __wbg_width_71b671f940e3eb8c: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_width_b5e609025d3f7451: function(arg0) {
            const ret = arg0.width;
            return ret;
        },
        __wbg_writeBuffer_8b5bd251a89198bc: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
            arg0.writeBuffer(arg1, arg2, getArrayU8FromWasm0(arg3, arg4), arg5, arg6);
        }, arguments); },
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 604, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C_);
            return ret;
        },
        __wbindgen_generic_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("GPUDevice")], shim_idx: 604, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C__2);
            return ret;
        },
        __wbindgen_generic_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("GPUUncapturedErrorEvent")], shim_idx: 299, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCsk8bttHmVmDQ_7web_sys8features14gen_MouseEvent10MouseEventuKb1_ECs8B2jB4lLsNq_13inox_platform);
            return ret;
        },
        __wbindgen_generic_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("KeyboardEvent")], shim_idx: 299, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCsk8bttHmVmDQ_7web_sys8features14gen_MouseEvent10MouseEventuKb1_ECs8B2jB4lLsNq_13inox_platform_4);
            return ret;
        },
        __wbindgen_generic_0000000000000005: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("MouseEvent")], shim_idx: 299, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCsk8bttHmVmDQ_7web_sys8features14gen_MouseEvent10MouseEventuKb1_ECs8B2jB4lLsNq_13inox_platform_5);
            return ret;
        },
        __wbindgen_generic_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("any")], shim_idx: 604, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C__6);
            return ret;
        },
        __wbindgen_generic_0000000000000007: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [NamedExternref("undefined")], shim_idx: 604, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C__7);
            return ret;
        },
        __wbindgen_generic_0000000000000008: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 1, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress0_1__6invokeuKb1_ECs4yokGJ1ZxKM_13inox_launcher);
            return ret;
        },
        __wbindgen_generic_0000000000000009: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_generic_000000000000000a: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(F32)) -> NamedExternref("Float32Array")`.
            const ret = getArrayF32FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_generic_000000000000000b: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(I16)) -> NamedExternref("Int16Array")`.
            const ret = getArrayI16FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_generic_000000000000000c: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(I32)) -> NamedExternref("Int32Array")`.
            const ret = getArrayI32FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_generic_000000000000000d: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(I8)) -> NamedExternref("Int8Array")`.
            const ret = getArrayI8FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_generic_000000000000000e: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U16)) -> NamedExternref("Uint16Array")`.
            const ret = getArrayU16FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_generic_000000000000000f: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U32)) -> NamedExternref("Uint32Array")`.
            const ret = getArrayU32FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_generic_0000000000000010: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_generic_0000000000000011: function(arg0, arg1) {
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
        "./inox_launcher_bg.js": import0,
    };
}

function _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress0_1__6invokeuKb1_ECs4yokGJ1ZxKM_13inox_launcher(arg0, arg1) {
    wasm._RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress0_1__6invokeuKb1_ECs4yokGJ1ZxKM_13inox_launcher(arg0, arg1);
}

function _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCsk8bttHmVmDQ_7web_sys8features14gen_MouseEvent10MouseEventuKb1_ECs8B2jB4lLsNq_13inox_platform(arg0, arg1, arg2) {
    wasm._RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCsk8bttHmVmDQ_7web_sys8features14gen_MouseEvent10MouseEventuKb1_ECs8B2jB4lLsNq_13inox_platform(arg0, arg1, arg2);
}

function _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCsk8bttHmVmDQ_7web_sys8features14gen_MouseEvent10MouseEventuKb1_ECs8B2jB4lLsNq_13inox_platform_4(arg0, arg1, arg2) {
    wasm._RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCsk8bttHmVmDQ_7web_sys8features14gen_MouseEvent10MouseEventuKb1_ECs8B2jB4lLsNq_13inox_platform_4(arg0, arg1, arg2);
}

function _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCsk8bttHmVmDQ_7web_sys8features14gen_MouseEvent10MouseEventuKb1_ECs8B2jB4lLsNq_13inox_platform_5(arg0, arg1, arg2) {
    wasm._RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeNtNtNtCsk8bttHmVmDQ_7web_sys8features14gen_MouseEvent10MouseEventuKb1_ECs8B2jB4lLsNq_13inox_platform_5(arg0, arg1, arg2);
}

function _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C_(arg0, arg1, arg2) {
    const ret = wasm._RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C_(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C__2(arg0, arg1, arg2) {
    const ret = wasm._RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C__2(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C__6(arg0, arg1, arg2) {
    const ret = wasm._RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C__6(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function _RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C__7(arg0, arg1, arg2) {
    const ret = wasm._RINvNvNtNtCs5WdW3eUQXII_12wasm_bindgen7convert8closuress2_1__6invokeINtNtB8_3sys10JsNullableNtNtNtNtNtCsiKCLri6YpM5_4wgpu7backend6webgpu10webgpu_sys14gen_GpuAdapter10GpuAdapterEINtNtCsknUcikIyyBm_4core6result6ResultuNtB8_7JsErrorEKb1_EB1C__7(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}


const __wbindgen_enum_GpuAddressMode = ["clamp-to-edge", "repeat", "mirror-repeat"];


const __wbindgen_enum_GpuAutoLayoutMode = ["auto"];


const __wbindgen_enum_GpuBlendFactor = ["zero", "one", "src", "one-minus-src", "src-alpha", "one-minus-src-alpha", "dst", "one-minus-dst", "dst-alpha", "one-minus-dst-alpha", "src-alpha-saturated", "constant", "one-minus-constant", "src1", "one-minus-src1", "src1-alpha", "one-minus-src1-alpha"];


const __wbindgen_enum_GpuBlendOperation = ["add", "subtract", "reverse-subtract", "min", "max"];


const __wbindgen_enum_GpuBufferBindingType = ["uniform", "storage", "read-only-storage"];


const __wbindgen_enum_GpuCanvasAlphaMode = ["opaque", "premultiplied"];


const __wbindgen_enum_GpuCanvasToneMappingMode = ["standard", "extended"];


const __wbindgen_enum_GpuCompareFunction = ["never", "less", "equal", "less-equal", "greater", "not-equal", "greater-equal", "always"];


const __wbindgen_enum_GpuCullMode = ["none", "front", "back"];


const __wbindgen_enum_GpuFilterMode = ["nearest", "linear"];


const __wbindgen_enum_GpuFrontFace = ["ccw", "cw"];


const __wbindgen_enum_GpuIndexFormat = ["uint16", "uint32"];


const __wbindgen_enum_GpuLoadOp = ["load", "clear"];


const __wbindgen_enum_GpuMipmapFilterMode = ["nearest", "linear"];


const __wbindgen_enum_GpuPowerPreference = ["low-power", "high-performance"];


const __wbindgen_enum_GpuPrimitiveTopology = ["point-list", "line-list", "line-strip", "triangle-list", "triangle-strip"];


const __wbindgen_enum_GpuSamplerBindingType = ["filtering", "non-filtering", "comparison"];


const __wbindgen_enum_GpuStencilOperation = ["keep", "zero", "replace", "invert", "increment-clamp", "decrement-clamp", "increment-wrap", "decrement-wrap"];


const __wbindgen_enum_GpuStorageTextureAccess = ["write-only", "read-only", "read-write"];


const __wbindgen_enum_GpuStoreOp = ["store", "discard"];


const __wbindgen_enum_GpuTextureAspect = ["all", "stencil-only", "depth-only"];


const __wbindgen_enum_GpuTextureDimension = ["1d", "2d", "3d"];


const __wbindgen_enum_GpuTextureFormat = ["r8unorm", "r8snorm", "r8uint", "r8sint", "r16unorm", "r16snorm", "r16uint", "r16sint", "r16float", "rg8unorm", "rg8snorm", "rg8uint", "rg8sint", "r32uint", "r32sint", "r32float", "rg16unorm", "rg16snorm", "rg16uint", "rg16sint", "rg16float", "rgba8unorm", "rgba8unorm-srgb", "rgba8snorm", "rgba8uint", "rgba8sint", "bgra8unorm", "bgra8unorm-srgb", "rgb9e5ufloat", "rgb10a2uint", "rgb10a2unorm", "rg11b10ufloat", "rg32uint", "rg32sint", "rg32float", "rgba16unorm", "rgba16snorm", "rgba16uint", "rgba16sint", "rgba16float", "rgba32uint", "rgba32sint", "rgba32float", "stencil8", "depth16unorm", "depth24plus", "depth24plus-stencil8", "depth32float", "depth32float-stencil8", "bc1-rgba-unorm", "bc1-rgba-unorm-srgb", "bc2-rgba-unorm", "bc2-rgba-unorm-srgb", "bc3-rgba-unorm", "bc3-rgba-unorm-srgb", "bc4-r-unorm", "bc4-r-snorm", "bc5-rg-unorm", "bc5-rg-snorm", "bc6h-rgb-ufloat", "bc6h-rgb-float", "bc7-rgba-unorm", "bc7-rgba-unorm-srgb", "etc2-rgb8unorm", "etc2-rgb8unorm-srgb", "etc2-rgb8a1unorm", "etc2-rgb8a1unorm-srgb", "etc2-rgba8unorm", "etc2-rgba8unorm-srgb", "eac-r11unorm", "eac-r11snorm", "eac-rg11unorm", "eac-rg11snorm", "astc-4x4-unorm", "astc-4x4-unorm-srgb", "astc-5x4-unorm", "astc-5x4-unorm-srgb", "astc-5x5-unorm", "astc-5x5-unorm-srgb", "astc-6x5-unorm", "astc-6x5-unorm-srgb", "astc-6x6-unorm", "astc-6x6-unorm-srgb", "astc-8x5-unorm", "astc-8x5-unorm-srgb", "astc-8x6-unorm", "astc-8x6-unorm-srgb", "astc-8x8-unorm", "astc-8x8-unorm-srgb", "astc-10x5-unorm", "astc-10x5-unorm-srgb", "astc-10x6-unorm", "astc-10x6-unorm-srgb", "astc-10x8-unorm", "astc-10x8-unorm-srgb", "astc-10x10-unorm", "astc-10x10-unorm-srgb", "astc-12x10-unorm", "astc-12x10-unorm-srgb", "astc-12x12-unorm", "astc-12x12-unorm-srgb"];


const __wbindgen_enum_GpuTextureSampleType = ["float", "unfilterable-float", "depth", "sint", "uint"];


const __wbindgen_enum_GpuTextureViewDimension = ["1d", "2d", "2d-array", "cube", "cube-array", "3d"];


const __wbindgen_enum_GpuVertexFormat = ["uint8", "uint8x2", "uint8x4", "sint8", "sint8x2", "sint8x4", "unorm8", "unorm8x2", "unorm8x4", "snorm8", "snorm8x2", "snorm8x4", "uint16", "uint16x2", "uint16x4", "sint16", "sint16x2", "sint16x4", "unorm16", "unorm16x2", "unorm16x4", "snorm16", "snorm16x2", "snorm16x4", "float16", "float16x2", "float16x4", "float32", "float32x2", "float32x3", "float32x4", "uint32", "uint32x2", "uint32x3", "uint32x4", "sint32", "sint32x2", "sint32x3", "sint32x4", "unorm10-10-10-2", "unorm8x4-bgra"];


const __wbindgen_enum_GpuVertexStepMode = ["vertex", "instance"];

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

function getArrayI16FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt16ArrayMemory0().subarray(ptr / 2, ptr / 2 + len);
}

function getArrayI32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayI8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getArrayJsValueViewFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    return result;
}

function getArrayU16FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint16ArrayMemory0().subarray(ptr / 2, ptr / 2 + len);
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

let cachedInt16ArrayMemory0 = null;
function getInt16ArrayMemory0() {
    if (cachedInt16ArrayMemory0 === null || cachedInt16ArrayMemory0.byteLength === 0) {
        cachedInt16ArrayMemory0 = new Int16Array(wasm.memory.buffer);
    }
    return cachedInt16ArrayMemory0;
}

let cachedInt32ArrayMemory0 = null;
function getInt32ArrayMemory0() {
    if (cachedInt32ArrayMemory0 === null || cachedInt32ArrayMemory0.byteLength === 0) {
        cachedInt32ArrayMemory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32ArrayMemory0;
}

let cachedInt8ArrayMemory0 = null;
function getInt8ArrayMemory0() {
    if (cachedInt8ArrayMemory0 === null || cachedInt8ArrayMemory0.byteLength === 0) {
        cachedInt8ArrayMemory0 = new Int8Array(wasm.memory.buffer);
    }
    return cachedInt8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint16ArrayMemory0 = null;
function getUint16ArrayMemory0() {
    if (cachedUint16ArrayMemory0 === null || cachedUint16ArrayMemory0.byteLength === 0) {
        cachedUint16ArrayMemory0 = new Uint16Array(wasm.memory.buffer);
    }
    return cachedUint16ArrayMemory0;
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
    cachedInt16ArrayMemory0 = null;
    cachedInt32ArrayMemory0 = null;
    cachedInt8ArrayMemory0 = null;
    cachedUint16ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

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
        module_or_path = new URL('inox_launcher_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
