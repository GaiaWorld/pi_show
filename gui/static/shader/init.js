var gl = document.getElementById("canvas").getContext('webgl');
window.__gl = gl;

var engine = Module._create_engine();

// 设置图片shader
var __jsObj = color_vs_shader_name;
var __jsObj1 = color_vs_code;
Module._set_shader(engine);

var __jsObj = color_fs_shader_name;
var __jsObj1 = color_fs_code;
Module._set_shader(engine);

// 设置图片shader
var __jsObj = image_vs_shader_name;
var __jsObj1 = image_vs_code;
Module._set_shader(engine);

var __jsObj = image_fs_shader_name;
var __jsObj1 = image_fs_code;
Module._set_shader(engine);

// 设置文字shader
var __jsObj = text_vs_shader_name;
var __jsObj1 = text_vs_code;
Module._set_shader(engine);

var __jsObj = text_fs_shader_name;
var __jsObj1 = text_fs_code;
Module._set_shader(engine);


