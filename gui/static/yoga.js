var _a;
var require, module;
var CONSTANTS = {
    ALIGN_COUNT: 8,
    ALIGN_AUTO: 0,
    ALIGN_FLEX_START: 1,
    ALIGN_CENTER: 2,
    ALIGN_FLEX_END: 3,
    ALIGN_STRETCH: 4,
    ALIGN_BASELINE: 5,
    ALIGN_SPACE_BETWEEN: 6,
    ALIGN_SPACE_AROUND: 7,
  
    DIMENSION_COUNT: 2,
    DIMENSION_WIDTH: 0,
    DIMENSION_HEIGHT: 1,
  
    DIRECTION_COUNT: 3,
    DIRECTION_INHERIT: 0,
    DIRECTION_LTR: 1,
    DIRECTION_RTL: 2,
  
    DISPLAY_COUNT: 2,
    DISPLAY_FLEX: 0,
    DISPLAY_NONE: 1,
  
    EDGE_COUNT: 9,
    EDGE_LEFT: 0,
    EDGE_TOP: 1,
    EDGE_RIGHT: 2,
    EDGE_BOTTOM: 3,
    EDGE_START: 4,
    EDGE_END: 5,
    EDGE_HORIZONTAL: 6,
    EDGE_VERTICAL: 7,
    EDGE_ALL: 8,
  
    EXPERIMENTAL_FEATURE_COUNT: 1,
    EXPERIMENTAL_FEATURE_WEB_FLEX_BASIS: 0,
  
    FLEX_DIRECTION_COUNT: 4,
    FLEX_DIRECTION_COLUMN: 0,
    FLEX_DIRECTION_COLUMN_REVERSE: 1,
    FLEX_DIRECTION_ROW: 2,
    FLEX_DIRECTION_ROW_REVERSE: 3,
  
    JUSTIFY_COUNT: 6,
    JUSTIFY_FLEX_START: 0,
    JUSTIFY_CENTER: 1,
    JUSTIFY_FLEX_END: 2,
    JUSTIFY_SPACE_BETWEEN: 3,
    JUSTIFY_SPACE_AROUND: 4,
    JUSTIFY_SPACE_EVENLY: 5,
  
    LOG_LEVEL_COUNT: 6,
    LOG_LEVEL_ERROR: 0,
    LOG_LEVEL_WARN: 1,
    LOG_LEVEL_INFO: 2,
    LOG_LEVEL_DEBUG: 3,
    LOG_LEVEL_VERBOSE: 4,
    LOG_LEVEL_FATAL: 5,
  
    MEASURE_MODE_COUNT: 3,
    MEASURE_MODE_UNDEFINED: 0,
    MEASURE_MODE_EXACTLY: 1,
    MEASURE_MODE_AT_MOST: 2,
  
    NODE_TYPE_COUNT: 2,
    NODE_TYPE_DEFAULT: 0,
    NODE_TYPE_TEXT: 1,
  
    OVERFLOW_COUNT: 3,
    OVERFLOW_VISIBLE: 0,
    OVERFLOW_HIDDEN: 1,
    OVERFLOW_SCROLL: 2,
  
    POSITION_TYPE_COUNT: 2,
    POSITION_TYPE_RELATIVE: 0,
    POSITION_TYPE_ABSOLUTE: 1,
  
    PRINT_OPTIONS_COUNT: 3,
    PRINT_OPTIONS_LAYOUT: 1,
    PRINT_OPTIONS_STYLE: 2,
    PRINT_OPTIONS_CHILDREN: 4,
  
    UNIT_COUNT: 4,
    UNIT_UNDEFINED: 0,
    UNIT_POINT: 1,
    UNIT_PERCENT: 2,
    UNIT_AUTO: 3,
  
    WRAP_COUNT: 3,
    WRAP_NO_WRAP: 0,
    WRAP_WRAP: 1,
    WRAP_WRAP_REVERSE: 2
  };


  var _extends = Object.assign || function (target) { for (var i = 1; i < arguments.length; i++) { var source = arguments[i]; for (var key in source) { if (Object.prototype.hasOwnProperty.call(source, key)) { target[key] = source[key]; } } } return target; };

var _createClass = function () { function defineProperties(target, props) { for (var i = 0; i < props.length; i++) { var descriptor = props[i]; descriptor.enumerable = descriptor.enumerable || false; descriptor.configurable = true; if ("value" in descriptor) descriptor.writable = true; Object.defineProperty(target, descriptor.key, descriptor); } } return function (Constructor, protoProps, staticProps) { if (protoProps) defineProperties(Constructor.prototype, protoProps); if (staticProps) defineProperties(Constructor, staticProps); return Constructor; }; }();

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

function _classCallCheck(instance, Constructor) { if (!(instance instanceof Constructor)) { throw new TypeError("Cannot call a class as a function"); } }

/**
 * Copyright (c) 2014-present, Facebook, Inc.
 * All rights reserved.
 *
 * This source code is licensed under the BSD-style license found in the
 * LICENSE file in the root directory of this source tree. An additional grant
 * of patent rights can be found in the PATENTS file in the same directory.
 *
 * 
 * @format
 */

var Layout = function () {
  function Layout(left, right, top, bottom, width, height) {
    _classCallCheck(this, Layout);

    this.left = left;
    this.right = right;
    this.top = top;
    this.bottom = bottom;
    this.width = width;
    this.height = height;
  }

  _createClass(Layout, [{
    key: 'fromJS',
    value: function fromJS(expose) {
      expose(this.left, this.right, this.top, this.bottom, this.width, this.height);
    }
  }, {
    key: 'toString',
    value: function toString() {
      return '<Layout#' + this.left + ':' + this.right + ';' + this.top + ':' + this.bottom + ';' + this.width + ':' + this.height + '>';
    }
  }]);

  return Layout;
}();

var Size = function () {
  _createClass(Size, null, [{
    key: 'fromJS',
    value: function fromJS(_ref) {
      var width = _ref.width,
          height = _ref.height;

      return new Size(width, height);
    }
  }]);

  function Size(width, height) {
    _classCallCheck(this, Size);

    this.width = width;
    this.height = height;
  }

  _createClass(Size, [{
    key: 'fromJS',
    value: function fromJS(expose) {
      expose(this.width, this.height);
    }
  }, {
    key: 'toString',
    value: function toString() {
      return '<Size#' + this.width + 'x' + this.height + '>';
    }
  }]);

  return Size;
}();

var Value = function () {
  function Value(unit, value) {
    _classCallCheck(this, Value);

    this.unit = unit;
    this.value = value;
  }

  _createClass(Value, [{
    key: 'fromJS',
    value: function fromJS(expose) {
      expose(this.unit, this.value);
    }
  }, {
    key: 'toString',
    value: function toString() {
      switch (this.unit) {
        case CONSTANTS.UNIT_POINT:
          return String(this.value);
        case CONSTANTS.UNIT_PERCENT:
          return this.value + '%';
        case CONSTANTS.UNIT_AUTO:
          return 'auto';
        default:
          {
            return this.value + '?';
          }
      }
    }
  }, {
    key: 'valueOf',
    value: function valueOf() {
      return this.value;
    }
  }]);

  return Value;
}();

var Yoga = function (bind, lib) {
  function patch(prototype, name, fn) {
    var original = prototype[name];

    prototype[name] = function () {
      for (var _len = arguments.length, args = Array(_len), _key = 0; _key < _len; _key++) {
        args[_key] = arguments[_key];
      }

      return fn.call.apply(fn, [this, original].concat(args));
    };
  }

  var _arr = ['setPosition', 'setMargin', 'setFlexBasis', 'setWidth', 'setHeight', 'setMinWidth', 'setMinHeight', 'setMaxWidth', 'setMaxHeight', 'setPadding'];

  var _loop = function _loop() {
    var _methods;

    var fnName = _arr[_i];
    var methods = (_methods = {}, _defineProperty(_methods, CONSTANTS.UNIT_POINT, lib.Node.prototype[fnName]), _defineProperty(_methods, CONSTANTS.UNIT_PERCENT, lib.Node.prototype[fnName + 'Percent']), _defineProperty(_methods, CONSTANTS.UNIT_AUTO, lib.Node.prototype[fnName + 'Auto']), _methods);

    patch(lib.Node.prototype, fnName, function (original) {
      for (var _len2 = arguments.length, args = Array(_len2 > 1 ? _len2 - 1 : 0), _key2 = 1; _key2 < _len2; _key2++) {
        args[_key2 - 1] = arguments[_key2];
      }

      // We patch all these functions to add support for the following calls:
      // .setWidth(100) / .setWidth("100%") / .setWidth(.getWidth()) / .setWidth("auto")

      var value = args.pop();
      var unit = void 0,
          asNumber = void 0;

      if (value === 'auto') {
        unit = CONSTANTS.UNIT_AUTO;
        asNumber = undefined;
      } else if (value instanceof Value) {
        unit = value.unit;
        asNumber = value.valueOf();
      } else {
        unit = typeof value === 'string' && value.endsWith('%') ? CONSTANTS.UNIT_PERCENT : CONSTANTS.UNIT_POINT;
        asNumber = parseFloat(value);
        if (!Number.isNaN(value) && Number.isNaN(asNumber)) {
          throw new Error('Invalid value ' + value + ' for ' + fnName);
        }
      }

      if (!methods[unit]) throw new Error('Failed to execute "' + fnName + '": Unsupported unit \'' + value + '\'');

      if (asNumber !== undefined) {
        var _methods$unit;

        return (_methods$unit = methods[unit]).call.apply(_methods$unit, [this].concat(args, [asNumber]));
      } else {
        var _methods$unit2;

        return (_methods$unit2 = methods[unit]).call.apply(_methods$unit2, [this].concat(args));
      }
    });
  };

  for (var _i = 0; _i < _arr.length; _i++) {
    _loop();
  }

  patch(lib.Config.prototype, 'free', function () {
    // Since we handle the memory allocation ourselves (via lib.Config.create),
    // we also need to handle the deallocation
    lib.Config.destroy(this);
  });

  patch(lib.Node, 'create', function (_, config) {
    // We decide the constructor we want to call depending on the parameters
    return config ? lib.Node.createWithConfig(config) : lib.Node.createDefault();
  });

  patch(lib.Node.prototype, 'free', function () {
    // Since we handle the memory allocation ourselves (via lib.Node.create),
    // we also need to handle the deallocation
    lib.Node.destroy(this);
  });

  patch(lib.Node.prototype, 'freeRecursive', function () {
    for (var t = 0, T = this.getChildCount(); t < T; ++t) {
      this.getChild(0).freeRecursive();
    }
    this.free();
  });

  patch(lib.Node.prototype, 'setMeasureFunc', function (original, measureFunc) {
    // This patch is just a convenience patch, since it helps write more
    // idiomatic source code (such as .setMeasureFunc(null))
    // We also automatically convert the return value of the measureFunc
    // to a Size object, so that we can return anything that has .width and
    // .height properties
    if (measureFunc) {
      return original.call(this, function () {
        return Size.fromJS(measureFunc.apply(undefined, arguments));
      });
    } else {
      return this.unsetMeasureFunc();
    }
  });

  patch(lib.Node.prototype, 'calculateLayout', function (original) {
    var width = arguments.length > 1 && arguments[1] !== undefined ? arguments[1] : NaN;
    var height = arguments.length > 2 && arguments[2] !== undefined ? arguments[2] : NaN;
    var direction = arguments.length > 3 && arguments[3] !== undefined ? arguments[3] : CONSTANTS.DIRECTION_LTR;

    // Just a small patch to add support for the function default parameters
    return original.call(this, width, height, direction);
  });

  return _extends({
    Config: lib.Config,
    Node: lib.Node,
    Layout: bind('Layout', Layout),
    Size: bind('Size', Size),
    Value: bind('Value', Value),
    getInstanceCount: function getInstanceCount() {
      return lib.getInstanceCount.apply(lib, arguments);
    }
  }, CONSTANTS);
};


var Module=typeof Module!=="undefined"?Module:{};var moduleOverrides={};var key;for(key in Module){if(Module.hasOwnProperty(key)){moduleOverrides[key]=Module[key]}}Module["arguments"]=[];Module["thisProgram"]="./this.program";Module["quit"]=function(status,toThrow){throw toThrow};Module["preRun"]=[];Module["postRun"]=[];var ENVIRONMENT_IS_WEB=false;var ENVIRONMENT_IS_WORKER=false;var ENVIRONMENT_IS_NODE=false;var ENVIRONMENT_IS_SHELL=false;ENVIRONMENT_IS_WEB=typeof window==="object";ENVIRONMENT_IS_WORKER=typeof importScripts==="function";ENVIRONMENT_IS_NODE=typeof process==="object"&&typeof require==="function"&&!ENVIRONMENT_IS_WEB&&!ENVIRONMENT_IS_WORKER;ENVIRONMENT_IS_SHELL=!ENVIRONMENT_IS_WEB&&!ENVIRONMENT_IS_NODE&&!ENVIRONMENT_IS_WORKER;var scriptDirectory="";function locateFile(path){if(Module["locateFile"]){return Module["locateFile"](path,scriptDirectory)}else{return scriptDirectory+path}}if(ENVIRONMENT_IS_NODE){scriptDirectory=__dirname+"/";var nodeFS;var nodePath;Module["read"]=function shell_read(filename,binary){var ret;if(!nodeFS)nodeFS=require("fs");if(!nodePath)nodePath=require("path");filename=nodePath["normalize"](filename);ret=nodeFS["readFileSync"](filename);return binary?ret:ret.toString()};Module["readBinary"]=function readBinary(filename){var ret=Module["read"](filename,true);if(!ret.buffer){ret=new Uint8Array(ret)}assert(ret.buffer);return ret};if(process["argv"].length>1){Module["thisProgram"]=process["argv"][1].replace(/\\/g,"/")}Module["arguments"]=process["argv"].slice(2);if(typeof module!=="undefined"){module["exports"]=Module}process["on"]("uncaughtException",function(ex){if(!(ex instanceof ExitStatus)){throw ex}});process["on"]("unhandledRejection",abort);Module["quit"]=function(status){process["exit"](status)};Module["inspect"]=function(){return"[Emscripten Module object]"}}else if(ENVIRONMENT_IS_SHELL){if(typeof read!="undefined"){Module["read"]=function shell_read(f){return read(f)}}Module["readBinary"]=function readBinary(f){var data;if(typeof readbuffer==="function"){return new Uint8Array(readbuffer(f))}data=read(f,"binary");assert(typeof data==="object");return data};if(typeof scriptArgs!="undefined"){Module["arguments"]=scriptArgs}else if(typeof arguments!="undefined"){Module["arguments"]=arguments}if(typeof quit==="function"){Module["quit"]=function(status){quit(status)}}}else if(ENVIRONMENT_IS_WEB||ENVIRONMENT_IS_WORKER){if(ENVIRONMENT_IS_WORKER){scriptDirectory=self.location.href}else if(document.currentScript){scriptDirectory=document.currentScript.src}if(scriptDirectory.indexOf("blob:")!==0){scriptDirectory=scriptDirectory.substr(0,scriptDirectory.lastIndexOf("/")+1)}else{scriptDirectory=""}Module["read"]=function shell_read(url){var xhr=new XMLHttpRequest;xhr.open("GET",url,false);xhr.send(null);return xhr.responseText};if(ENVIRONMENT_IS_WORKER){Module["readBinary"]=function readBinary(url){var xhr=new XMLHttpRequest;xhr.open("GET",url,false);xhr.responseType="arraybuffer";xhr.send(null);return new Uint8Array(xhr.response)}}Module["readAsync"]=function readAsync(url,onload,onerror){var xhr=new XMLHttpRequest;xhr.open("GET",url,true);xhr.responseType="arraybuffer";xhr.onload=function xhr_onload(){if(xhr.status==200||xhr.status==0&&xhr.response){onload(xhr.response);return}onerror()};xhr.onerror=onerror;xhr.send(null)};Module["setWindowTitle"]=function(title){document.title=title}}else{}var out=Module["print"]||(typeof console!=="undefined"?console.log.bind(console):typeof print!=="undefined"?print:null);var err=Module["printErr"]||(typeof printErr!=="undefined"?printErr:typeof console!=="undefined"&&console.warn.bind(console)||out);for(key in moduleOverrides){if(moduleOverrides.hasOwnProperty(key)){Module[key]=moduleOverrides[key]}}moduleOverrides=undefined;var STACK_ALIGN=16;function dynamicAlloc(size){var ret=HEAP32[DYNAMICTOP_PTR>>2];var end=ret+size+15&-16;if(end<=_emscripten_get_heap_size()){HEAP32[DYNAMICTOP_PTR>>2]=end}else{return 0}return ret}function getNativeTypeSize(type){switch(type){case"i1":case"i8":return 1;case"i16":return 2;case"i32":return 4;case"i64":return 8;case"float":return 4;case"double":return 8;default:{if(type[type.length-1]==="*"){return 4}else if(type[0]==="i"){var bits=parseInt(type.substr(1));assert(bits%8===0,"getNativeTypeSize invalid bits "+bits+", type "+type);return bits/8}else{return 0}}}}function warnOnce(text){if(!warnOnce.shown)warnOnce.shown={};if(!warnOnce.shown[text]){warnOnce.shown[text]=1;err(text)}}var jsCallStartIndex=1;var functionPointers=new Array(0);var funcWrappers={};function dynCall(sig,ptr,args){if(args&&args.length){return Module["dynCall_"+sig].apply(null,[ptr].concat(args))}else{return Module["dynCall_"+sig].call(null,ptr)}}var tempRet0=0;var setTempRet0=function(value){tempRet0=value};var getTempRet0=function(){return tempRet0};var GLOBAL_BASE=8;var ABORT=false;var EXITSTATUS=0;function assert(condition,text){if(!condition){abort("Assertion failed: "+text)}}function getCFunc(ident){var func=Module["_"+ident];assert(func,"Cannot call unknown function "+ident+", make sure it is exported");return func}function ccall(ident,returnType,argTypes,args,opts){var toC={"string":function(str){var ret=0;if(str!==null&&str!==undefined&&str!==0){var len=(str.length<<2)+1;ret=stackAlloc(len);stringToUTF8(str,ret,len)}return ret},"array":function(arr){var ret=stackAlloc(arr.length);writeArrayToMemory(arr,ret);return ret}};function convertReturnValue(ret){if(returnType==="string")return UTF8ToString(ret);if(returnType==="boolean")return Boolean(ret);return ret}var func=getCFunc(ident);var cArgs=[];var stack=0;if(args){for(var i=0;i<args.length;i++){var converter=toC[argTypes[i]];if(converter){if(stack===0)stack=stackSave();cArgs[i]=converter(args[i])}else{cArgs[i]=args[i]}}}var ret=func.apply(null,cArgs);ret=convertReturnValue(ret);if(stack!==0)stackRestore(stack);return ret}function setValue(ptr,value,type,noSafe){type=type||"i8";if(type.charAt(type.length-1)==="*")type="i32";switch(type){case"i1":HEAP8[ptr>>0]=value;break;case"i8":HEAP8[ptr>>0]=value;break;case"i16":HEAP16[ptr>>1]=value;break;case"i32":HEAP32[ptr>>2]=value;break;case"i64":tempI64=[value>>>0,(tempDouble=value,+Math_abs(tempDouble)>=+1?tempDouble>+0?(Math_min(+Math_floor(tempDouble/+4294967296),+4294967295)|0)>>>0:~~+Math_ceil((tempDouble-+(~~tempDouble>>>0))/+4294967296)>>>0:0)],HEAP32[ptr>>2]=tempI64[0],HEAP32[ptr+4>>2]=tempI64[1];break;case"float":HEAPF32[ptr>>2]=value;break;case"double":HEAPF64[ptr>>3]=value;break;default:abort("invalid type for setValue: "+type)}}var ALLOC_NONE=3;var UTF8Decoder=typeof TextDecoder!=="undefined"?new TextDecoder("utf8"):undefined;function UTF8ArrayToString(u8Array,idx,maxBytesToRead){var endIdx=idx+maxBytesToRead;var endPtr=idx;while(u8Array[endPtr]&&!(endPtr>=endIdx))++endPtr;if(endPtr-idx>16&&u8Array.subarray&&UTF8Decoder){return UTF8Decoder.decode(u8Array.subarray(idx,endPtr))}else{var str="";while(idx<endPtr){var u0=u8Array[idx++];if(!(u0&128)){str+=String.fromCharCode(u0);continue}var u1=u8Array[idx++]&63;if((u0&224)==192){str+=String.fromCharCode((u0&31)<<6|u1);continue}var u2=u8Array[idx++]&63;if((u0&240)==224){u0=(u0&15)<<12|u1<<6|u2}else{u0=(u0&7)<<18|u1<<12|u2<<6|u8Array[idx++]&63}if(u0<65536){str+=String.fromCharCode(u0)}else{var ch=u0-65536;str+=String.fromCharCode(55296|ch>>10,56320|ch&1023)}}}return str}function UTF8ToString(ptr,maxBytesToRead){return ptr?UTF8ArrayToString(HEAPU8,ptr,maxBytesToRead):""}function stringToUTF8Array(str,outU8Array,outIdx,maxBytesToWrite){if(!(maxBytesToWrite>0))return 0;var startIdx=outIdx;var endIdx=outIdx+maxBytesToWrite-1;for(var i=0;i<str.length;++i){var u=str.charCodeAt(i);if(u>=55296&&u<=57343){var u1=str.charCodeAt(++i);u=65536+((u&1023)<<10)|u1&1023}if(u<=127){if(outIdx>=endIdx)break;outU8Array[outIdx++]=u}else if(u<=2047){if(outIdx+1>=endIdx)break;outU8Array[outIdx++]=192|u>>6;outU8Array[outIdx++]=128|u&63}else if(u<=65535){if(outIdx+2>=endIdx)break;outU8Array[outIdx++]=224|u>>12;outU8Array[outIdx++]=128|u>>6&63;outU8Array[outIdx++]=128|u&63}else{if(outIdx+3>=endIdx)break;outU8Array[outIdx++]=240|u>>18;outU8Array[outIdx++]=128|u>>12&63;outU8Array[outIdx++]=128|u>>6&63;outU8Array[outIdx++]=128|u&63}}outU8Array[outIdx]=0;return outIdx-startIdx}function stringToUTF8(str,outPtr,maxBytesToWrite){return stringToUTF8Array(str,HEAPU8,outPtr,maxBytesToWrite)}function lengthBytesUTF8(str){var len=0;for(var i=0;i<str.length;++i){var u=str.charCodeAt(i);if(u>=55296&&u<=57343)u=65536+((u&1023)<<10)|str.charCodeAt(++i)&1023;if(u<=127)++len;else if(u<=2047)len+=2;else if(u<=65535)len+=3;else len+=4}return len}var UTF16Decoder=typeof TextDecoder!=="undefined"?new TextDecoder("utf-16le"):undefined;function writeArrayToMemory(array,buffer){HEAP8.set(array,buffer)}function writeAsciiToMemory(str,buffer,dontAddNull){for(var i=0;i<str.length;++i){HEAP8[buffer++>>0]=str.charCodeAt(i)}if(!dontAddNull)HEAP8[buffer>>0]=0}function demangle(func){return func}function demangleAll(text){var regex=/__Z[\w\d_]+/g;return text.replace(regex,function(x){var y=demangle(x);return x===y?x:y+" ["+x+"]"})}function jsStackTrace(){var err=new Error;if(!err.stack){try{throw new Error(0)}catch(e){err=e}if(!err.stack){return"(no stack trace available)"}}return err.stack.toString()}var buffer,HEAP8,HEAPU8,HEAP16,HEAPU16,HEAP32,HEAPU32,HEAPF32,HEAPF64;function updateGlobalBufferViews(){Module["HEAP8"]=HEAP8=new Int8Array(buffer);Module["HEAP16"]=HEAP16=new Int16Array(buffer);Module["HEAP32"]=HEAP32=new Int32Array(buffer);Module["HEAPU8"]=HEAPU8=new Uint8Array(buffer);Module["HEAPU16"]=HEAPU16=new Uint16Array(buffer);Module["HEAPU32"]=HEAPU32=new Uint32Array(buffer);Module["HEAPF32"]=HEAPF32=new Float32Array(buffer);Module["HEAPF64"]=HEAPF64=new Float64Array(buffer)}var STACK_BASE=1152,DYNAMIC_BASE=5244032,DYNAMICTOP_PTR=896;var TOTAL_STACK=5242880;var TOTAL_MEMORY=Module["TOTAL_MEMORY"]||16777216;if(TOTAL_MEMORY<TOTAL_STACK)err("TOTAL_MEMORY should be larger than TOTAL_STACK, was "+TOTAL_MEMORY+"! (TOTAL_STACK="+TOTAL_STACK+")");if(Module["buffer"]){buffer=Module["buffer"]}else{{buffer=new ArrayBuffer(TOTAL_MEMORY)}Module["buffer"]=buffer}updateGlobalBufferViews();HEAP32[DYNAMICTOP_PTR>>2]=DYNAMIC_BASE;function callRuntimeCallbacks(callbacks){while(callbacks.length>0){var callback=callbacks.shift();if(typeof callback=="function"){callback();continue}var func=callback.func;if(typeof func==="number"){if(callback.arg===undefined){Module["dynCall_v"](func)}else{Module["dynCall_vi"](func,callback.arg)}}else{func(callback.arg===undefined?null:callback.arg)}}}var __ATPRERUN__=[];var __ATINIT__=[];var __ATMAIN__=[];var __ATPOSTRUN__=[];var runtimeInitialized=false;var runtimeExited=false;function preRun(){if(Module["preRun"]){if(typeof Module["preRun"]=="function")Module["preRun"]=[Module["preRun"]];while(Module["preRun"].length){addOnPreRun(Module["preRun"].shift())}}callRuntimeCallbacks(__ATPRERUN__)}function ensureInitRuntime(){if(runtimeInitialized)return;runtimeInitialized=true;callRuntimeCallbacks(__ATINIT__)}function preMain(){callRuntimeCallbacks(__ATMAIN__)}function exitRuntime(){runtimeExited=true}function postRun(){if(Module["postRun"]){if(typeof Module["postRun"]=="function")Module["postRun"]=[Module["postRun"]];while(Module["postRun"].length){addOnPostRun(Module["postRun"].shift())}}callRuntimeCallbacks(__ATPOSTRUN__)}function addOnPreRun(cb){__ATPRERUN__.unshift(cb)}function addOnPostRun(cb){__ATPOSTRUN__.unshift(cb)}var Math_abs=Math.abs;var Math_ceil=Math.ceil;var Math_floor=Math.floor;var Math_min=Math.min;var runDependencies=0;var runDependencyWatcher=null;var dependenciesFulfilled=null;function addRunDependency(id){runDependencies++;if(Module["monitorRunDependencies"]){Module["monitorRunDependencies"](runDependencies)}}function removeRunDependency(id){runDependencies--;if(Module["monitorRunDependencies"]){Module["monitorRunDependencies"](runDependencies)}if(runDependencies==0){if(runDependencyWatcher!==null){clearInterval(runDependencyWatcher);runDependencyWatcher=null}if(dependenciesFulfilled){var callback=dependenciesFulfilled;dependenciesFulfilled=null;callback()}}}Module["preloadedImages"]={};Module["preloadedAudios"]={};var memoryInitializer=null;var dataURIPrefix="data:application/octet-stream;base64,";function isDataURI(filename){return String.prototype.startsWith?filename.startsWith(dataURIPrefix):filename.indexOf(dataURIPrefix)===0}memoryInitializer="yoga.js.mem";var tempDoublePtr=1136;function _emscripten_get_heap_size(){return TOTAL_MEMORY}function abortOnCannotGrowMemory(requestedSize){abort("OOM")}function _emscripten_resize_heap(requestedSize){abortOnCannotGrowMemory(requestedSize)}function _emscripten_memcpy_big(dest,src,num){HEAPU8.set(HEAPU8.subarray(src,src+num),dest)}function ___setErrNo(value){if(Module["___errno_location"])HEAP32[Module["___errno_location"]()>>2]=value;return value}var ASSERTIONS=false;var asmGlobalArg={"Int8Array":Int8Array,"Int16Array":Int16Array,"Int32Array":Int32Array,"Uint8Array":Uint8Array,"Uint16Array":Uint16Array};var asmLibraryArg={"a":abort,"b":setTempRet0,"c":getTempRet0,"d":___setErrNo,"e":_emscripten_get_heap_size,"f":_emscripten_memcpy_big,"g":_emscripten_resize_heap,"h":abortOnCannotGrowMemory,"i":tempDoublePtr,"j":DYNAMICTOP_PTR};// EMSCRIPTEN_START_ASM
var asm=(/** @suppress {uselessCode} */ function(global,env,buffer) {
"use asm";var a=new global.Int8Array(buffer),b=new global.Int16Array(buffer),c=new global.Int32Array(buffer),d=new global.Uint8Array(buffer),e=new global.Uint16Array(buffer),f=env.i|0,g=env.j|0,h=0,i=0,j=0,k=0,l=0,m=0,n=0,o=0.0,p=env.a,q=env.b,r=env.c,s=env.d,t=env.e,u=env.f,v=env.g,w=env.h,x=1152,y=5244032,z=0.0;
// EMSCRIPTEN_START_FUNCS
function F(a){a=a|0;var b=0;b=x;x=x+a|0;x=x+15&-16;return b|0}function G(){return x|0}function H(a){a=a|0;x=a}function I(a,b){a=a|0;b=b|0;x=a;y=b}function J(){return 368}function K(a){a=a|0;var b=0,d=0,e=0,f=0,g=0,h=0,i=0,j=0,k=0,l=0,m=0,n=0,o=0,p=0,q=0,r=0,s=0,t=0,u=0,v=0,w=0;w=x;x=x+16|0;n=w;do if(a>>>0<245){k=a>>>0<11?16:a+11&-8;a=k>>>3;m=c[93]|0;d=m>>>a;if(d&3|0){b=(d&1^1)+a|0;a=412+(b<<1<<2)|0;d=a+8|0;e=c[d>>2]|0;f=e+8|0;g=c[f>>2]|0;if((g|0)==(a|0))c[93]=m&~(1<<b);else{c[g+12>>2]=a;c[d>>2]=g}v=b<<3;c[e+4>>2]=v|3;v=e+v+4|0;c[v>>2]=c[v>>2]|1;v=f;x=w;return v|0}l=c[95]|0;if(k>>>0>l>>>0){if(d|0){b=2<<a;b=d<<a&(b|0-b);b=(b&0-b)+-1|0;i=b>>>12&16;b=b>>>i;d=b>>>5&8;b=b>>>d;g=b>>>2&4;b=b>>>g;a=b>>>1&2;b=b>>>a;e=b>>>1&1;e=(d|i|g|a|e)+(b>>>e)|0;b=412+(e<<1<<2)|0;a=b+8|0;g=c[a>>2]|0;i=g+8|0;d=c[i>>2]|0;if((d|0)==(b|0)){a=m&~(1<<e);c[93]=a}else{c[d+12>>2]=b;c[a>>2]=d;a=m}v=e<<3;h=v-k|0;c[g+4>>2]=k|3;f=g+k|0;c[f+4>>2]=h|1;c[g+v>>2]=h;if(l|0){e=c[98]|0;b=l>>>3;d=412+(b<<1<<2)|0;b=1<<b;if(!(a&b)){c[93]=a|b;b=d;a=d+8|0}else{a=d+8|0;b=c[a>>2]|0}c[a>>2]=e;c[b+12>>2]=e;c[e+8>>2]=b;c[e+12>>2]=d}c[95]=h;c[98]=f;v=i;x=w;return v|0}g=c[94]|0;if(g){d=(g&0-g)+-1|0;f=d>>>12&16;d=d>>>f;e=d>>>5&8;d=d>>>e;h=d>>>2&4;d=d>>>h;i=d>>>1&2;d=d>>>i;j=d>>>1&1;j=c[676+((e|f|h|i|j)+(d>>>j)<<2)>>2]|0;d=j;i=j;j=(c[j+4>>2]&-8)-k|0;while(1){a=c[d+16>>2]|0;if(!a){a=c[d+20>>2]|0;if(!a)break}h=(c[a+4>>2]&-8)-k|0;f=h>>>0<j>>>0;d=a;i=f?a:i;j=f?h:j}h=i+k|0;if(h>>>0>i>>>0){f=c[i+24>>2]|0;b=c[i+12>>2]|0;do if((b|0)==(i|0)){a=i+20|0;b=c[a>>2]|0;if(!b){a=i+16|0;b=c[a>>2]|0;if(!b){d=0;break}}while(1){e=b+20|0;d=c[e>>2]|0;if(!d){e=b+16|0;d=c[e>>2]|0;if(!d)break;else{b=d;a=e}}else{b=d;a=e}}c[a>>2]=0;d=b}else{d=c[i+8>>2]|0;c[d+12>>2]=b;c[b+8>>2]=d;d=b}while(0);do if(f|0){b=c[i+28>>2]|0;a=676+(b<<2)|0;if((i|0)==(c[a>>2]|0)){c[a>>2]=d;if(!d){c[94]=g&~(1<<b);break}}else{v=f+16|0;c[((c[v>>2]|0)==(i|0)?v:f+20|0)>>2]=d;if(!d)break}c[d+24>>2]=f;b=c[i+16>>2]|0;if(b|0){c[d+16>>2]=b;c[b+24>>2]=d}b=c[i+20>>2]|0;if(b|0){c[d+20>>2]=b;c[b+24>>2]=d}}while(0);if(j>>>0<16){v=j+k|0;c[i+4>>2]=v|3;v=i+v+4|0;c[v>>2]=c[v>>2]|1}else{c[i+4>>2]=k|3;c[h+4>>2]=j|1;c[h+j>>2]=j;if(l|0){e=c[98]|0;b=l>>>3;d=412+(b<<1<<2)|0;b=1<<b;if(!(b&m)){c[93]=b|m;b=d;a=d+8|0}else{a=d+8|0;b=c[a>>2]|0}c[a>>2]=e;c[b+12>>2]=e;c[e+8>>2]=b;c[e+12>>2]=d}c[95]=j;c[98]=h}v=i+8|0;x=w;return v|0}else m=k}else m=k}else m=k}else if(a>>>0<=4294967231){a=a+11|0;k=a&-8;e=c[94]|0;if(e){f=0-k|0;a=a>>>8;if(a)if(k>>>0>16777215)j=31;else{m=(a+1048320|0)>>>16&8;q=a<<m;i=(q+520192|0)>>>16&4;q=q<<i;j=(q+245760|0)>>>16&2;j=14-(i|m|j)+(q<<j>>>15)|0;j=k>>>(j+7|0)&1|j<<1}else j=0;d=c[676+(j<<2)>>2]|0;a:do if(!d){d=0;a=0;q=61}else{a=0;i=k<<((j|0)==31?0:25-(j>>>1)|0);g=0;while(1){h=(c[d+4>>2]&-8)-k|0;if(h>>>0<f>>>0)if(!h){a=d;f=0;q=65;break a}else{a=d;f=h}q=c[d+20>>2]|0;d=c[d+16+(i>>>31<<2)>>2]|0;g=(q|0)==0|(q|0)==(d|0)?g:q;if(!d){d=g;q=61;break}else i=i<<1}}while(0);if((q|0)==61){if((d|0)==0&(a|0)==0){a=2<<j;a=(a|0-a)&e;if(!a){m=k;break}m=(a&0-a)+-1|0;h=m>>>12&16;m=m>>>h;g=m>>>5&8;m=m>>>g;i=m>>>2&4;m=m>>>i;j=m>>>1&2;m=m>>>j;d=m>>>1&1;a=0;d=c[676+((g|h|i|j|d)+(m>>>d)<<2)>>2]|0}if(!d){i=a;h=f}else q=65}if((q|0)==65){g=d;while(1){m=(c[g+4>>2]&-8)-k|0;d=m>>>0<f>>>0;f=d?m:f;a=d?g:a;d=c[g+16>>2]|0;if(!d)d=c[g+20>>2]|0;if(!d){i=a;h=f;break}else g=d}}if(((i|0)!=0?h>>>0<((c[95]|0)-k|0)>>>0:0)?(l=i+k|0,l>>>0>i>>>0):0){g=c[i+24>>2]|0;b=c[i+12>>2]|0;do if((b|0)==(i|0)){a=i+20|0;b=c[a>>2]|0;if(!b){a=i+16|0;b=c[a>>2]|0;if(!b){b=0;break}}while(1){f=b+20|0;d=c[f>>2]|0;if(!d){f=b+16|0;d=c[f>>2]|0;if(!d)break;else{b=d;a=f}}else{b=d;a=f}}c[a>>2]=0}else{v=c[i+8>>2]|0;c[v+12>>2]=b;c[b+8>>2]=v}while(0);do if(g){a=c[i+28>>2]|0;d=676+(a<<2)|0;if((i|0)==(c[d>>2]|0)){c[d>>2]=b;if(!b){e=e&~(1<<a);c[94]=e;break}}else{v=g+16|0;c[((c[v>>2]|0)==(i|0)?v:g+20|0)>>2]=b;if(!b)break}c[b+24>>2]=g;a=c[i+16>>2]|0;if(a|0){c[b+16>>2]=a;c[a+24>>2]=b}a=c[i+20>>2]|0;if(a){c[b+20>>2]=a;c[a+24>>2]=b}}while(0);b:do if(h>>>0<16){v=h+k|0;c[i+4>>2]=v|3;v=i+v+4|0;c[v>>2]=c[v>>2]|1}else{c[i+4>>2]=k|3;c[l+4>>2]=h|1;c[l+h>>2]=h;b=h>>>3;if(h>>>0<256){d=412+(b<<1<<2)|0;a=c[93]|0;b=1<<b;if(!(a&b)){c[93]=a|b;b=d;a=d+8|0}else{a=d+8|0;b=c[a>>2]|0}c[a>>2]=l;c[b+12>>2]=l;c[l+8>>2]=b;c[l+12>>2]=d;break}b=h>>>8;if(b)if(h>>>0>16777215)d=31;else{u=(b+1048320|0)>>>16&8;v=b<<u;t=(v+520192|0)>>>16&4;v=v<<t;d=(v+245760|0)>>>16&2;d=14-(t|u|d)+(v<<d>>>15)|0;d=h>>>(d+7|0)&1|d<<1}else d=0;b=676+(d<<2)|0;c[l+28>>2]=d;a=l+16|0;c[a+4>>2]=0;c[a>>2]=0;a=1<<d;if(!(e&a)){c[94]=e|a;c[b>>2]=l;c[l+24>>2]=b;c[l+12>>2]=l;c[l+8>>2]=l;break}b=c[b>>2]|0;c:do if((c[b+4>>2]&-8|0)!=(h|0)){e=h<<((d|0)==31?0:25-(d>>>1)|0);while(1){d=b+16+(e>>>31<<2)|0;a=c[d>>2]|0;if(!a)break;if((c[a+4>>2]&-8|0)==(h|0)){b=a;break c}else{e=e<<1;b=a}}c[d>>2]=l;c[l+24>>2]=b;c[l+12>>2]=l;c[l+8>>2]=l;break b}while(0);u=b+8|0;v=c[u>>2]|0;c[v+12>>2]=l;c[u>>2]=l;c[l+8>>2]=v;c[l+12>>2]=b;c[l+24>>2]=0}while(0);v=i+8|0;x=w;return v|0}else m=k}else m=k}else m=-1;while(0);d=c[95]|0;if(d>>>0>=m>>>0){b=d-m|0;a=c[98]|0;if(b>>>0>15){v=a+m|0;c[98]=v;c[95]=b;c[v+4>>2]=b|1;c[a+d>>2]=b;c[a+4>>2]=m|3}else{c[95]=0;c[98]=0;c[a+4>>2]=d|3;v=a+d+4|0;c[v>>2]=c[v>>2]|1}v=a+8|0;x=w;return v|0}h=c[96]|0;if(h>>>0>m>>>0){t=h-m|0;c[96]=t;v=c[99]|0;u=v+m|0;c[99]=u;c[u+4>>2]=t|1;c[v+4>>2]=m|3;v=v+8|0;x=w;return v|0}if(!(c[211]|0)){c[213]=4096;c[212]=4096;c[214]=-1;c[215]=-1;c[216]=0;c[204]=0;c[211]=n&-16^1431655768;a=4096}else a=c[213]|0;i=m+48|0;j=m+47|0;g=a+j|0;f=0-a|0;k=g&f;if(k>>>0<=m>>>0){v=0;x=w;return v|0}a=c[203]|0;if(a|0?(l=c[201]|0,n=l+k|0,n>>>0<=l>>>0|n>>>0>a>>>0):0){v=0;x=w;return v|0}d:do if(!(c[204]&4)){d=c[99]|0;e:do if(d){e=820;while(1){n=c[e>>2]|0;if(n>>>0<=d>>>0?(n+(c[e+4>>2]|0)|0)>>>0>d>>>0:0)break;a=c[e+8>>2]|0;if(!a){q=128;break e}else e=a}b=g-h&f;if(b>>>0<2147483647){a=ha(b|0)|0;if((a|0)==((c[e>>2]|0)+(c[e+4>>2]|0)|0)){if((a|0)!=(-1|0)){h=b;g=a;q=145;break d}}else{e=a;q=136}}else b=0}else q=128;while(0);do if((q|0)==128){d=ha(0)|0;if((d|0)!=(-1|0)?(b=d,o=c[212]|0,p=o+-1|0,b=((p&b|0)==0?0:(p+b&0-o)-b|0)+k|0,o=c[201]|0,p=b+o|0,b>>>0>m>>>0&b>>>0<2147483647):0){n=c[203]|0;if(n|0?p>>>0<=o>>>0|p>>>0>n>>>0:0){b=0;break}a=ha(b|0)|0;if((a|0)==(d|0)){h=b;g=d;q=145;break d}else{e=a;q=136}}else b=0}while(0);do if((q|0)==136){d=0-b|0;if(!(i>>>0>b>>>0&(b>>>0<2147483647&(e|0)!=(-1|0))))if((e|0)==(-1|0)){b=0;break}else{h=b;g=e;q=145;break d}a=c[213]|0;a=j-b+a&0-a;if(a>>>0>=2147483647){h=b;g=e;q=145;break d}if((ha(a|0)|0)==(-1|0)){ha(d|0)|0;b=0;break}else{h=a+b|0;g=e;q=145;break d}}while(0);c[204]=c[204]|4;q=143}else{b=0;q=143}while(0);if(((q|0)==143?k>>>0<2147483647:0)?(t=ha(k|0)|0,p=ha(0)|0,r=p-t|0,s=r>>>0>(m+40|0)>>>0,!((t|0)==(-1|0)|s^1|t>>>0<p>>>0&((t|0)!=(-1|0)&(p|0)!=(-1|0))^1)):0){h=s?r:b;g=t;q=145}if((q|0)==145){b=(c[201]|0)+h|0;c[201]=b;if(b>>>0>(c[202]|0)>>>0)c[202]=b;j=c[99]|0;f:do if(j){b=820;while(1){a=c[b>>2]|0;d=c[b+4>>2]|0;if((g|0)==(a+d|0)){q=154;break}e=c[b+8>>2]|0;if(!e)break;else b=e}if(((q|0)==154?(u=b+4|0,(c[b+12>>2]&8|0)==0):0)?g>>>0>j>>>0&a>>>0<=j>>>0:0){c[u>>2]=d+h;v=(c[96]|0)+h|0;t=j+8|0;t=(t&7|0)==0?0:0-t&7;u=j+t|0;t=v-t|0;c[99]=u;c[96]=t;c[u+4>>2]=t|1;c[j+v+4>>2]=40;c[100]=c[215];break}if(g>>>0<(c[97]|0)>>>0)c[97]=g;d=g+h|0;b=820;while(1){if((c[b>>2]|0)==(d|0)){q=162;break}a=c[b+8>>2]|0;if(!a)break;else b=a}if((q|0)==162?(c[b+12>>2]&8|0)==0:0){c[b>>2]=g;l=b+4|0;c[l>>2]=(c[l>>2]|0)+h;l=g+8|0;l=g+((l&7|0)==0?0:0-l&7)|0;b=d+8|0;b=d+((b&7|0)==0?0:0-b&7)|0;k=l+m|0;i=b-l-m|0;c[l+4>>2]=m|3;g:do if((j|0)==(b|0)){v=(c[96]|0)+i|0;c[96]=v;c[99]=k;c[k+4>>2]=v|1}else{if((c[98]|0)==(b|0)){v=(c[95]|0)+i|0;c[95]=v;c[98]=k;c[k+4>>2]=v|1;c[k+v>>2]=v;break}a=c[b+4>>2]|0;if((a&3|0)==1){h=a&-8;e=a>>>3;h:do if(a>>>0<256){a=c[b+8>>2]|0;d=c[b+12>>2]|0;if((d|0)==(a|0)){c[93]=c[93]&~(1<<e);break}else{c[a+12>>2]=d;c[d+8>>2]=a;break}}else{g=c[b+24>>2]|0;a=c[b+12>>2]|0;do if((a|0)==(b|0)){d=b+16|0;e=d+4|0;a=c[e>>2]|0;if(!a){a=c[d>>2]|0;if(!a){a=0;break}}else d=e;while(1){f=a+20|0;e=c[f>>2]|0;if(!e){f=a+16|0;e=c[f>>2]|0;if(!e)break;else{a=e;d=f}}else{a=e;d=f}}c[d>>2]=0}else{v=c[b+8>>2]|0;c[v+12>>2]=a;c[a+8>>2]=v}while(0);if(!g)break;d=c[b+28>>2]|0;e=676+(d<<2)|0;do if((c[e>>2]|0)!=(b|0)){v=g+16|0;c[((c[v>>2]|0)==(b|0)?v:g+20|0)>>2]=a;if(!a)break h}else{c[e>>2]=a;if(a|0)break;c[94]=c[94]&~(1<<d);break h}while(0);c[a+24>>2]=g;d=b+16|0;e=c[d>>2]|0;if(e|0){c[a+16>>2]=e;c[e+24>>2]=a}d=c[d+4>>2]|0;if(!d)break;c[a+20>>2]=d;c[d+24>>2]=a}while(0);b=b+h|0;f=h+i|0}else f=i;b=b+4|0;c[b>>2]=c[b>>2]&-2;c[k+4>>2]=f|1;c[k+f>>2]=f;b=f>>>3;if(f>>>0<256){d=412+(b<<1<<2)|0;a=c[93]|0;b=1<<b;if(!(a&b)){c[93]=a|b;b=d;a=d+8|0}else{a=d+8|0;b=c[a>>2]|0}c[a>>2]=k;c[b+12>>2]=k;c[k+8>>2]=b;c[k+12>>2]=d;break}b=f>>>8;do if(!b)e=0;else{if(f>>>0>16777215){e=31;break}u=(b+1048320|0)>>>16&8;v=b<<u;t=(v+520192|0)>>>16&4;v=v<<t;e=(v+245760|0)>>>16&2;e=14-(t|u|e)+(v<<e>>>15)|0;e=f>>>(e+7|0)&1|e<<1}while(0);b=676+(e<<2)|0;c[k+28>>2]=e;a=k+16|0;c[a+4>>2]=0;c[a>>2]=0;a=c[94]|0;d=1<<e;if(!(a&d)){c[94]=a|d;c[b>>2]=k;c[k+24>>2]=b;c[k+12>>2]=k;c[k+8>>2]=k;break}b=c[b>>2]|0;i:do if((c[b+4>>2]&-8|0)!=(f|0)){e=f<<((e|0)==31?0:25-(e>>>1)|0);while(1){d=b+16+(e>>>31<<2)|0;a=c[d>>2]|0;if(!a)break;if((c[a+4>>2]&-8|0)==(f|0)){b=a;break i}else{e=e<<1;b=a}}c[d>>2]=k;c[k+24>>2]=b;c[k+12>>2]=k;c[k+8>>2]=k;break g}while(0);u=b+8|0;v=c[u>>2]|0;c[v+12>>2]=k;c[u>>2]=k;c[k+8>>2]=v;c[k+12>>2]=b;c[k+24>>2]=0}while(0);v=l+8|0;x=w;return v|0}b=820;while(1){a=c[b>>2]|0;if(a>>>0<=j>>>0?(v=a+(c[b+4>>2]|0)|0,v>>>0>j>>>0):0)break;b=c[b+8>>2]|0}f=v+-47|0;a=f+8|0;a=f+((a&7|0)==0?0:0-a&7)|0;f=j+16|0;a=a>>>0<f>>>0?j:a;b=a+8|0;d=h+-40|0;t=g+8|0;t=(t&7|0)==0?0:0-t&7;u=g+t|0;t=d-t|0;c[99]=u;c[96]=t;c[u+4>>2]=t|1;c[g+d+4>>2]=40;c[100]=c[215];d=a+4|0;c[d>>2]=27;c[b>>2]=c[205];c[b+4>>2]=c[206];c[b+8>>2]=c[207];c[b+12>>2]=c[208];c[205]=g;c[206]=h;c[208]=0;c[207]=b;b=a+24|0;do{u=b;b=b+4|0;c[b>>2]=7}while((u+8|0)>>>0<v>>>0);if((a|0)!=(j|0)){g=a-j|0;c[d>>2]=c[d>>2]&-2;c[j+4>>2]=g|1;c[a>>2]=g;b=g>>>3;if(g>>>0<256){d=412+(b<<1<<2)|0;a=c[93]|0;b=1<<b;if(!(a&b)){c[93]=a|b;b=d;a=d+8|0}else{a=d+8|0;b=c[a>>2]|0}c[a>>2]=j;c[b+12>>2]=j;c[j+8>>2]=b;c[j+12>>2]=d;break}b=g>>>8;if(b)if(g>>>0>16777215)e=31;else{u=(b+1048320|0)>>>16&8;v=b<<u;t=(v+520192|0)>>>16&4;v=v<<t;e=(v+245760|0)>>>16&2;e=14-(t|u|e)+(v<<e>>>15)|0;e=g>>>(e+7|0)&1|e<<1}else e=0;d=676+(e<<2)|0;c[j+28>>2]=e;c[j+20>>2]=0;c[f>>2]=0;b=c[94]|0;a=1<<e;if(!(b&a)){c[94]=b|a;c[d>>2]=j;c[j+24>>2]=d;c[j+12>>2]=j;c[j+8>>2]=j;break}b=c[d>>2]|0;j:do if((c[b+4>>2]&-8|0)!=(g|0)){e=g<<((e|0)==31?0:25-(e>>>1)|0);while(1){d=b+16+(e>>>31<<2)|0;a=c[d>>2]|0;if(!a)break;if((c[a+4>>2]&-8|0)==(g|0)){b=a;break j}else{e=e<<1;b=a}}c[d>>2]=j;c[j+24>>2]=b;c[j+12>>2]=j;c[j+8>>2]=j;break f}while(0);u=b+8|0;v=c[u>>2]|0;c[v+12>>2]=j;c[u>>2]=j;c[j+8>>2]=v;c[j+12>>2]=b;c[j+24>>2]=0}}else{v=c[97]|0;if((v|0)==0|g>>>0<v>>>0)c[97]=g;c[205]=g;c[206]=h;c[208]=0;c[102]=c[211];c[101]=-1;c[106]=412;c[105]=412;c[108]=420;c[107]=420;c[110]=428;c[109]=428;c[112]=436;c[111]=436;c[114]=444;c[113]=444;c[116]=452;c[115]=452;c[118]=460;c[117]=460;c[120]=468;c[119]=468;c[122]=476;c[121]=476;c[124]=484;c[123]=484;c[126]=492;c[125]=492;c[128]=500;c[127]=500;c[130]=508;c[129]=508;c[132]=516;c[131]=516;c[134]=524;c[133]=524;c[136]=532;c[135]=532;c[138]=540;c[137]=540;c[140]=548;c[139]=548;c[142]=556;c[141]=556;c[144]=564;c[143]=564;c[146]=572;c[145]=572;c[148]=580;c[147]=580;c[150]=588;c[149]=588;c[152]=596;c[151]=596;c[154]=604;c[153]=604;c[156]=612;c[155]=612;c[158]=620;c[157]=620;c[160]=628;c[159]=628;c[162]=636;c[161]=636;c[164]=644;c[163]=644;c[166]=652;c[165]=652;c[168]=660;c[167]=660;v=h+-40|0;t=g+8|0;t=(t&7|0)==0?0:0-t&7;u=g+t|0;t=v-t|0;c[99]=u;c[96]=t;c[u+4>>2]=t|1;c[g+v+4>>2]=40;c[100]=c[215]}while(0);b=c[96]|0;if(b>>>0>m>>>0){t=b-m|0;c[96]=t;v=c[99]|0;u=v+m|0;c[99]=u;c[u+4>>2]=t|1;c[v+4>>2]=m|3;v=v+8|0;x=w;return v|0}}c[(J()|0)>>2]=12;v=0;x=w;return v|0}function L(a){a=a|0;var b=0,d=0,e=0,f=0,g=0,h=0,i=0,j=0;if(!a)return;d=a+-8|0;f=c[97]|0;a=c[a+-4>>2]|0;b=a&-8;j=d+b|0;do if(!(a&1)){e=c[d>>2]|0;if(!(a&3))return;h=d+(0-e)|0;g=e+b|0;if(h>>>0<f>>>0)return;if((c[98]|0)==(h|0)){a=j+4|0;b=c[a>>2]|0;if((b&3|0)!=3){i=h;b=g;break}c[95]=g;c[a>>2]=b&-2;c[h+4>>2]=g|1;c[h+g>>2]=g;return}d=e>>>3;if(e>>>0<256){a=c[h+8>>2]|0;b=c[h+12>>2]|0;if((b|0)==(a|0)){c[93]=c[93]&~(1<<d);i=h;b=g;break}else{c[a+12>>2]=b;c[b+8>>2]=a;i=h;b=g;break}}f=c[h+24>>2]|0;a=c[h+12>>2]|0;do if((a|0)==(h|0)){b=h+16|0;d=b+4|0;a=c[d>>2]|0;if(!a){a=c[b>>2]|0;if(!a){a=0;break}}else b=d;while(1){e=a+20|0;d=c[e>>2]|0;if(!d){e=a+16|0;d=c[e>>2]|0;if(!d)break;else{a=d;b=e}}else{a=d;b=e}}c[b>>2]=0}else{i=c[h+8>>2]|0;c[i+12>>2]=a;c[a+8>>2]=i}while(0);if(f){b=c[h+28>>2]|0;d=676+(b<<2)|0;if((c[d>>2]|0)==(h|0)){c[d>>2]=a;if(!a){c[94]=c[94]&~(1<<b);i=h;b=g;break}}else{i=f+16|0;c[((c[i>>2]|0)==(h|0)?i:f+20|0)>>2]=a;if(!a){i=h;b=g;break}}c[a+24>>2]=f;b=h+16|0;d=c[b>>2]|0;if(d|0){c[a+16>>2]=d;c[d+24>>2]=a}b=c[b+4>>2]|0;if(b){c[a+20>>2]=b;c[b+24>>2]=a;i=h;b=g}else{i=h;b=g}}else{i=h;b=g}}else{i=d;h=d}while(0);if(h>>>0>=j>>>0)return;a=j+4|0;e=c[a>>2]|0;if(!(e&1))return;if(!(e&2)){if((c[99]|0)==(j|0)){j=(c[96]|0)+b|0;c[96]=j;c[99]=i;c[i+4>>2]=j|1;if((i|0)!=(c[98]|0))return;c[98]=0;c[95]=0;return}if((c[98]|0)==(j|0)){j=(c[95]|0)+b|0;c[95]=j;c[98]=h;c[i+4>>2]=j|1;c[h+j>>2]=j;return}f=(e&-8)+b|0;d=e>>>3;do if(e>>>0<256){b=c[j+8>>2]|0;a=c[j+12>>2]|0;if((a|0)==(b|0)){c[93]=c[93]&~(1<<d);break}else{c[b+12>>2]=a;c[a+8>>2]=b;break}}else{g=c[j+24>>2]|0;a=c[j+12>>2]|0;do if((a|0)==(j|0)){b=j+16|0;d=b+4|0;a=c[d>>2]|0;if(!a){a=c[b>>2]|0;if(!a){d=0;break}}else b=d;while(1){e=a+20|0;d=c[e>>2]|0;if(!d){e=a+16|0;d=c[e>>2]|0;if(!d)break;else{a=d;b=e}}else{a=d;b=e}}c[b>>2]=0;d=a}else{d=c[j+8>>2]|0;c[d+12>>2]=a;c[a+8>>2]=d;d=a}while(0);if(g|0){a=c[j+28>>2]|0;b=676+(a<<2)|0;if((c[b>>2]|0)==(j|0)){c[b>>2]=d;if(!d){c[94]=c[94]&~(1<<a);break}}else{e=g+16|0;c[((c[e>>2]|0)==(j|0)?e:g+20|0)>>2]=d;if(!d)break}c[d+24>>2]=g;a=j+16|0;b=c[a>>2]|0;if(b|0){c[d+16>>2]=b;c[b+24>>2]=d}a=c[a+4>>2]|0;if(a|0){c[d+20>>2]=a;c[a+24>>2]=d}}}while(0);c[i+4>>2]=f|1;c[h+f>>2]=f;if((i|0)==(c[98]|0)){c[95]=f;return}}else{c[a>>2]=e&-2;c[i+4>>2]=b|1;c[h+b>>2]=b;f=b}a=f>>>3;if(f>>>0<256){d=412+(a<<1<<2)|0;b=c[93]|0;a=1<<a;if(!(b&a)){c[93]=b|a;a=d;b=d+8|0}else{b=d+8|0;a=c[b>>2]|0}c[b>>2]=i;c[a+12>>2]=i;c[i+8>>2]=a;c[i+12>>2]=d;return}a=f>>>8;if(a)if(f>>>0>16777215)e=31;else{h=(a+1048320|0)>>>16&8;j=a<<h;g=(j+520192|0)>>>16&4;j=j<<g;e=(j+245760|0)>>>16&2;e=14-(g|h|e)+(j<<e>>>15)|0;e=f>>>(e+7|0)&1|e<<1}else e=0;a=676+(e<<2)|0;c[i+28>>2]=e;c[i+20>>2]=0;c[i+16>>2]=0;b=c[94]|0;d=1<<e;a:do if(!(b&d)){c[94]=b|d;c[a>>2]=i;c[i+24>>2]=a;c[i+12>>2]=i;c[i+8>>2]=i}else{a=c[a>>2]|0;b:do if((c[a+4>>2]&-8|0)!=(f|0)){e=f<<((e|0)==31?0:25-(e>>>1)|0);while(1){d=a+16+(e>>>31<<2)|0;b=c[d>>2]|0;if(!b)break;if((c[b+4>>2]&-8|0)==(f|0)){a=b;break b}else{e=e<<1;a=b}}c[d>>2]=i;c[i+24>>2]=a;c[i+12>>2]=i;c[i+8>>2]=i;break a}while(0);h=a+8|0;j=c[h>>2]|0;c[j+12>>2]=i;c[h>>2]=i;c[i+8>>2]=j;c[i+12>>2]=a;c[i+24>>2]=0}while(0);j=(c[101]|0)+-1|0;c[101]=j;if(j|0)return;a=828;while(1){a=c[a>>2]|0;if(!a)break;else a=a+8|0}c[101]=-1;return}function M(a){a=a|0;L(a);return}function N(a){a=a|0;return}function O(a){a=a|0;N(a);M(a);return}function P(a){a=a|0;return}function Q(a){a=a|0;return}function R(a,b,d){a=a|0;b=b|0;d=d|0;var e=0,f=0,g=0,h=0;h=x;x=x+64|0;f=h;if(!(V(a,b,0)|0))if((b|0)!=0?(g=Z(b,24,8,0)|0,(g|0)!=0):0){b=f+4|0;e=b+52|0;do{c[b>>2]=0;b=b+4|0}while((b|0)<(e|0));c[f>>2]=g;c[f+8>>2]=a;c[f+12>>2]=-1;c[f+48>>2]=1;C[c[(c[g>>2]|0)+28>>2]&3](g,f,c[d>>2]|0,1);if((c[f+24>>2]|0)==1){c[d>>2]=c[f+16>>2];b=1}else b=0}else b=0;else b=1;x=h;return b|0}function S(a,b,d,e,f,g){a=a|0;b=b|0;d=d|0;e=e|0;f=f|0;g=g|0;if(V(a,c[b+8>>2]|0,g)|0)Y(0,b,d,e,f);return}function T(b,d,e,f,g){b=b|0;d=d|0;e=e|0;f=f|0;g=g|0;var h=0;do if(!(V(b,c[d+8>>2]|0,g)|0)){if(V(b,c[d>>2]|0,g)|0){if((c[d+16>>2]|0)!=(e|0)?(h=d+20|0,(c[h>>2]|0)!=(e|0)):0){c[d+32>>2]=f;c[h>>2]=e;g=d+40|0;c[g>>2]=(c[g>>2]|0)+1;if((c[d+36>>2]|0)==1?(c[d+24>>2]|0)==2:0)a[d+54>>0]=1;c[d+44>>2]=4;break}if((f|0)==1)c[d+32>>2]=1}}else X(0,d,e,f);while(0);return}function U(a,b,d,e){a=a|0;b=b|0;d=d|0;e=e|0;if(V(a,c[b+8>>2]|0,0)|0)W(0,b,d,e);return}function V(a,b,c){a=a|0;b=b|0;c=c|0;return (a|0)==(b|0)|0}function W(b,d,e,f){b=b|0;d=d|0;e=e|0;f=f|0;var g=0;b=d+16|0;g=c[b>>2]|0;do if(g){if((g|0)!=(e|0)){f=d+36|0;c[f>>2]=(c[f>>2]|0)+1;c[d+24>>2]=2;a[d+54>>0]=1;break}b=d+24|0;if((c[b>>2]|0)==2)c[b>>2]=f}else{c[b>>2]=e;c[d+24>>2]=f;c[d+36>>2]=1}while(0);return}function X(a,b,d,e){a=a|0;b=b|0;d=d|0;e=e|0;var f=0;if((c[b+4>>2]|0)==(d|0)?(f=b+28|0,(c[f>>2]|0)!=1):0)c[f>>2]=e;return}function Y(b,d,e,f,g){b=b|0;d=d|0;e=e|0;f=f|0;g=g|0;a[d+53>>0]=1;do if((c[d+4>>2]|0)==(f|0)){a[d+52>>0]=1;b=d+16|0;f=c[b>>2]|0;if(!f){c[b>>2]=e;c[d+24>>2]=g;c[d+36>>2]=1;if(!((g|0)==1?(c[d+48>>2]|0)==1:0))break;a[d+54>>0]=1;break}if((f|0)!=(e|0)){g=d+36|0;c[g>>2]=(c[g>>2]|0)+1;a[d+54>>0]=1;break}f=d+24|0;b=c[f>>2]|0;if((b|0)==2){c[f>>2]=g;b=g}if((b|0)==1?(c[d+48>>2]|0)==1:0)a[d+54>>0]=1}while(0);return}function Z(d,e,f,g){d=d|0;e=e|0;f=f|0;g=g|0;var h=0,i=0,j=0,k=0,l=0,m=0,n=0,o=0,p=0;p=x;x=x+64|0;n=p;m=c[d>>2]|0;o=d+(c[m+-8>>2]|0)|0;m=c[m+-4>>2]|0;c[n>>2]=f;c[n+4>>2]=d;c[n+8>>2]=e;c[n+12>>2]=g;d=n+16|0;e=n+20|0;g=n+24|0;h=n+28|0;i=n+32|0;j=n+40|0;k=d;l=k+36|0;do{c[k>>2]=0;k=k+4|0}while((k|0)<(l|0));b[d+36>>1]=0;a[d+38>>0]=0;a:do if(V(m,f,0)|0){c[n+48>>2]=1;E[c[(c[m>>2]|0)+20>>2]&3](m,n,o,o,1,0);d=(c[g>>2]|0)==1?o:0}else{D[c[(c[m>>2]|0)+24>>2]&3](m,n,o,1,0);switch(c[n+36>>2]|0){case 0:{d=(c[j>>2]|0)==1&(c[h>>2]|0)==1&(c[i>>2]|0)==1?c[e>>2]|0:0;break a}case 1:break;default:{d=0;break a}}if((c[g>>2]|0)!=1?!((c[j>>2]|0)==0&(c[h>>2]|0)==1&(c[i>>2]|0)==1):0){d=0;break}d=c[d>>2]|0}while(0);x=p;return d|0}function _(a){a=a|0;N(a);M(a);return}function $(a,b,d,e,f,g){a=a|0;b=b|0;d=d|0;e=e|0;f=f|0;g=g|0;if(V(a,c[b+8>>2]|0,g)|0)Y(0,b,d,e,f);else{a=c[a+8>>2]|0;E[c[(c[a>>2]|0)+20>>2]&3](a,b,d,e,f,g)}return}function aa(b,d,e,f,g){b=b|0;d=d|0;e=e|0;f=f|0;g=g|0;var h=0,i=0,j=0;do if(!(V(b,c[d+8>>2]|0,g)|0)){if(!(V(b,c[d>>2]|0,g)|0)){i=c[b+8>>2]|0;D[c[(c[i>>2]|0)+24>>2]&3](i,d,e,f,g);break}if((c[d+16>>2]|0)!=(e|0)?(h=d+20|0,(c[h>>2]|0)!=(e|0)):0){c[d+32>>2]=f;i=d+44|0;if((c[i>>2]|0)==4)break;f=d+52|0;a[f>>0]=0;j=d+53|0;a[j>>0]=0;b=c[b+8>>2]|0;E[c[(c[b>>2]|0)+20>>2]&3](b,d,e,e,1,g);if(a[j>>0]|0)if(!(a[f>>0]|0)){f=1;b=11}else b=15;else{f=0;b=11}do if((b|0)==11){c[h>>2]=e;j=d+40|0;c[j>>2]=(c[j>>2]|0)+1;if((c[d+36>>2]|0)==1?(c[d+24>>2]|0)==2:0){a[d+54>>0]=1;if(f){b=15;break}else{f=4;break}}if(f)b=15;else f=4}while(0);if((b|0)==15)f=3;c[i>>2]=f;break}if((f|0)==1)c[d+32>>2]=1}else X(0,d,e,f);while(0);return}function ba(a,b,d,e){a=a|0;b=b|0;d=d|0;e=e|0;if(V(a,c[b+8>>2]|0,0)|0)W(0,b,d,e);else{a=c[a+8>>2]|0;C[c[(c[a>>2]|0)+28>>2]&3](a,b,d,e)}return}function ca(a){a=a|0;return}function da(a,b,d){a=a|0;b=b|0;d=d|0;var e=0,f=0;f=x;x=x+16|0;e=f;c[e>>2]=c[d>>2];a=A[c[(c[a>>2]|0)+16>>2]&1](a,b,e)|0;if(a)c[d>>2]=c[e>>2];x=f;return a&1|0}function ea(a){a=a|0;if(!a)a=0;else a=(Z(a,24,80,0)|0)!=0&1;return a|0}function fa(b,d,e){b=b|0;d=d|0;e=e|0;var f=0,g=0,h=0;if((e|0)>=8192){u(b|0,d|0,e|0)|0;return b|0}h=b|0;g=b+e|0;if((b&3)==(d&3)){while(b&3){if(!e)return h|0;a[b>>0]=a[d>>0]|0;b=b+1|0;d=d+1|0;e=e-1|0}e=g&-4|0;f=e-64|0;while((b|0)<=(f|0)){c[b>>2]=c[d>>2];c[b+4>>2]=c[d+4>>2];c[b+8>>2]=c[d+8>>2];c[b+12>>2]=c[d+12>>2];c[b+16>>2]=c[d+16>>2];c[b+20>>2]=c[d+20>>2];c[b+24>>2]=c[d+24>>2];c[b+28>>2]=c[d+28>>2];c[b+32>>2]=c[d+32>>2];c[b+36>>2]=c[d+36>>2];c[b+40>>2]=c[d+40>>2];c[b+44>>2]=c[d+44>>2];c[b+48>>2]=c[d+48>>2];c[b+52>>2]=c[d+52>>2];c[b+56>>2]=c[d+56>>2];c[b+60>>2]=c[d+60>>2];b=b+64|0;d=d+64|0}while((b|0)<(e|0)){c[b>>2]=c[d>>2];b=b+4|0;d=d+4|0}}else{e=g-4|0;while((b|0)<(e|0)){a[b>>0]=a[d>>0]|0;a[b+1>>0]=a[d+1>>0]|0;a[b+2>>0]=a[d+2>>0]|0;a[b+3>>0]=a[d+3>>0]|0;b=b+4|0;d=d+4|0}}while((b|0)<(g|0)){a[b>>0]=a[d>>0]|0;b=b+1|0;d=d+1|0}return h|0}function ga(b,d,e){b=b|0;d=d|0;e=e|0;var f=0,g=0,h=0,i=0;h=b+e|0;d=d&255;if((e|0)>=67){while(b&3){a[b>>0]=d;b=b+1|0}f=h&-4|0;i=d|d<<8|d<<16|d<<24;g=f-64|0;while((b|0)<=(g|0)){c[b>>2]=i;c[b+4>>2]=i;c[b+8>>2]=i;c[b+12>>2]=i;c[b+16>>2]=i;c[b+20>>2]=i;c[b+24>>2]=i;c[b+28>>2]=i;c[b+32>>2]=i;c[b+36>>2]=i;c[b+40>>2]=i;c[b+44>>2]=i;c[b+48>>2]=i;c[b+52>>2]=i;c[b+56>>2]=i;c[b+60>>2]=i;b=b+64|0}while((b|0)<(f|0)){c[b>>2]=i;b=b+4|0}}while((b|0)<(h|0)){a[b>>0]=d;b=b+1|0}return h-e|0}function ha(a){a=a|0;var b=0,d=0;d=c[g>>2]|0;b=d+a|0;if((a|0)>0&(b|0)<(d|0)|(b|0)<0){w(b|0)|0;s(12);return -1}if((b|0)>(t()|0)){if(!(v(b|0)|0)){s(12);return -1}}else c[g>>2]=b;return d|0}function ia(a,b,c,d){a=a|0;b=b|0;c=c|0;d=d|0;return A[a&1](b|0,c|0,d|0)|0}function ja(a,b){a=a|0;b=b|0;B[a&7](b|0)}function ka(a,b,c,d,e){a=a|0;b=b|0;c=c|0;d=d|0;e=e|0;C[a&3](b|0,c|0,d|0,e|0)}function la(a,b,c,d,e,f){a=a|0;b=b|0;c=c|0;d=d|0;e=e|0;f=f|0;D[a&3](b|0,c|0,d|0,e|0,f|0)}function ma(a,b,c,d,e,f,g){a=a|0;b=b|0;c=c|0;d=d|0;e=e|0;f=f|0;g=g|0;E[a&3](b|0,c|0,d|0,e|0,f|0,g|0)}function na(a,b,c){a=a|0;b=b|0;c=c|0;p(0);return 0}function oa(a){a=a|0;p(1)}function pa(a,b,c,d){a=a|0;b=b|0;c=c|0;d=d|0;p(2)}function qa(a,b,c,d,e){a=a|0;b=b|0;c=c|0;d=d|0;e=e|0;p(3)}function ra(a,b,c,d,e,f){a=a|0;b=b|0;c=c|0;d=d|0;e=e|0;f=f|0;p(4)}

// EMSCRIPTEN_END_FUNCS
var A=[na,R];var B=[oa,N,O,P,Q,_,oa,oa];var C=[pa,U,ba,pa];var D=[qa,T,aa,qa];var E=[ra,S,$,ra];return{___cxa_can_catch:da,___cxa_is_pointer_type:ea,___errno_location:J,_free:L,_malloc:K,_memcpy:fa,_memset:ga,_sbrk:ha,dynCall_iiii:ia,dynCall_vi:ja,dynCall_viiii:ka,dynCall_viiiii:la,dynCall_viiiiii:ma,establishStackSpace:I,stackAlloc:F,stackRestore:H,stackSave:G}})


// EMSCRIPTEN_END_ASM
(asmGlobalArg,asmLibraryArg,buffer);var ___cxa_can_catch=Module["___cxa_can_catch"]=asm["___cxa_can_catch"];var ___cxa_is_pointer_type=Module["___cxa_is_pointer_type"]=asm["___cxa_is_pointer_type"];var ___errno_location=Module["___errno_location"]=asm["___errno_location"];var _free=Module["_free"]=asm["_free"];var _malloc=Module["_malloc"]=asm["_malloc"];var _memcpy=Module["_memcpy"]=asm["_memcpy"];var _memset=Module["_memset"]=asm["_memset"];var _sbrk=Module["_sbrk"]=asm["_sbrk"];var establishStackSpace=Module["establishStackSpace"]=asm["establishStackSpace"];var stackAlloc=Module["stackAlloc"]=asm["stackAlloc"];var stackRestore=Module["stackRestore"]=asm["stackRestore"];var stackSave=Module["stackSave"]=asm["stackSave"];var dynCall_iiii=Module["dynCall_iiii"]=asm["dynCall_iiii"];var dynCall_vi=Module["dynCall_vi"]=asm["dynCall_vi"];var dynCall_viiii=Module["dynCall_viiii"]=asm["dynCall_viiii"];var dynCall_viiiii=Module["dynCall_viiiii"]=asm["dynCall_viiiii"];var dynCall_viiiiii=Module["dynCall_viiiiii"]=asm["dynCall_viiiiii"];Module["asm"]=asm;if(memoryInitializer){if(!isDataURI(memoryInitializer)){memoryInitializer=locateFile(memoryInitializer)}if(ENVIRONMENT_IS_NODE||ENVIRONMENT_IS_SHELL){var data=Module["readBinary"](memoryInitializer);HEAPU8.set(data,GLOBAL_BASE)}else{addRunDependency("memory initializer");var applyMemoryInitializer=function(data){if(data.byteLength)data=new Uint8Array(data);HEAPU8.set(data,GLOBAL_BASE);if(Module["memoryInitializerRequest"])delete Module["memoryInitializerRequest"].response;removeRunDependency("memory initializer")};var doBrowserLoad=function(){Module["readAsync"](memoryInitializer,applyMemoryInitializer,function(){throw"could not load memory initializer "+memoryInitializer})};if(Module["memoryInitializerRequest"]){var useRequest=function(){var request=Module["memoryInitializerRequest"];var response=request.response;if(request.status!==200&&request.status!==0){console.warn("a problem seems to have happened with Module.memoryInitializerRequest, status: "+request.status+", retrying "+memoryInitializer);doBrowserLoad();return}applyMemoryInitializer(response)};if(Module["memoryInitializerRequest"].response){setTimeout(useRequest,0)}else{Module["memoryInitializerRequest"].addEventListener("load",useRequest)}}else{doBrowserLoad()}}}function ExitStatus(status){this.name="ExitStatus";this.message="Program terminated with exit("+status+")";this.status=status}ExitStatus.prototype=new Error;ExitStatus.prototype.constructor=ExitStatus;dependenciesFulfilled=function runCaller(){if(!Module["calledRun"])run();if(!Module["calledRun"])dependenciesFulfilled=runCaller};function run(args){args=args||Module["arguments"];if(runDependencies>0){return}preRun();if(runDependencies>0)return;if(Module["calledRun"])return;function doRun(){if(Module["calledRun"])return;Module["calledRun"]=true;if(ABORT)return;ensureInitRuntime();preMain();if(Module["onRuntimeInitialized"])Module["onRuntimeInitialized"]();postRun()}if(Module["setStatus"]){Module["setStatus"]("Running...");setTimeout(function(){setTimeout(function(){Module["setStatus"]("")},1);doRun()},1)}else{doRun()}}Module["run"]=run;function abort(what){if(Module["onAbort"]){Module["onAbort"](what)}if(what!==undefined){out(what);err(what);what=JSON.stringify(what)}else{what=""}ABORT=true;EXITSTATUS=1;throw"abort("+what+"). Build with -s ASSERTIONS=1 for more info."}Module["abort"]=abort;if(Module["preInit"]){if(typeof Module["preInit"]=="function")Module["preInit"]=[Module["preInit"]];while(Module["preInit"].length>0){Module["preInit"].pop()()}}Module["noExitRuntime"]=true;run();

var ran = false;
var ret = null;

nbind.init({}, function (err, result) {
  if (ran) {
    return;
  }

  ran = true;

  if (err) {
    throw err;
  }

  ret = result;
});


if (!ran) {
  throw new Error("Failed to load the yoga module - it needed to be loaded synchronously, but didn't");
}

// $FlowFixMe ret will not be null here
var yoga = Yoga(ret.bind, ret.lib);

var pix = 1000;
var pix1  = pix;
var root = yoga.Node.create();
root.setWidth(pix);
root.setHeight(pix);
root.setJustifyContent(yoga.JUSTIFY_CENTER);


var createNode = function(root){
    var node = yoga.Node.create();
    node.setWidth(pix);
    node.setHeight(1);
    root.insertChild(node);
    return node;
}

var time = new Date().getTime();

for(var i = 0; i< 1000; i++){
    var root1 = createNode(root);
//     for(var j = 0; j < 10; j++){
//         var root2 = createNode(root1);
// //         for(var k = 0; k < 10; k++){
// //             var root3 = createNode(root2);
// //             for(var q = 0; q < 10; q++){
// //                 var root4 = createNode(root3);
// //                 for(var r = 0; r < 10; r++){
// //                     var root5 = createNode(root4);
// //                 }
// //             }
// //         }
//     }
}

console.log("create--------------------------", new Date().getTime() - time);

root.calculateLayout(pix1, pix1, yoga.DIRECTION_LTR);
console.log("create and calculateLayout--------------------------", new Date().getTime() - time);

root.setHeight(2000);
root.calculateLayout(2000, 2000, yoga.DIRECTION_LTR);
console.log("create and calculateLayout--------------------------", new Date().getTime() - time);

// var node1 = yoga.Node.create();
// node1.setWidth(100);
// node1.setHeight(100);

// var node2 = yoga.Node.create();
// node2.setWidth(100);
// node2.setHeight(100);
// node2.setJustifyContent(yoga.JUSTIFY_CENTER);

// var node3 = yoga.Node.create();
// node3.setWidth(50);
// node3.setHeight(50);
// node2.insertChild(node3, 0);

// root.insertChild(node1, 0);
// root.insertChild(node2, 1);

// root.calculateLayout(500, 300, yoga.DIRECTION_LTR);
// console.log(root.getComputedLayout());
// // {left: 0, top: 0, width: 500, height: 300}
// console.log(node1.getComputedLayout());
// // {left: 150, top: 0, width: 100, height: 100}
// console.log(node2.getComputedLayout());
// // {left: 250, top: 0, width: 100, height: 100}