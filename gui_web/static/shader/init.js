var gl = document.getElementById("canvas").getContext('webgl');
window.__gl = gl;
window.__fbo = undefined;

// 测试：渲染GUI到fbo
// window.__fbo = init_fbo(gl, 1000, 700);

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

// 设置Canvas文字shader
var __jsObj = canvas_text_vs_shader_name;
var __jsObj1 = canvas_text_vs_code;
Module._set_shader(engine);

var __jsObj = canvas_text_fs_shader_name;
var __jsObj1 = canvas_text_fs_code;
Module._set_shader(engine);


// 设置裁剪shader
var __jsObj = clip_vs_shader_name;
var __jsObj1 = clip_vs_code;
Module._set_shader(engine);

var __jsObj = clip_fs_shader_name;
var __jsObj1 = clip_fs_code;
Module._set_shader(engine);

function init_fbo(gl, width, height) {
    var fbo = gl.createFramebuffer();
    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);

    var rb = gl.createRenderbuffer();
    gl.bindRenderbuffer(gl.RENDERBUFFER, rb);
    let size = 1024;
    gl.renderbufferStorage(gl.RENDERBUFFER, gl.DEPTH_COMPONENT16, size, size);

    var texture = gl.createTexture();
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, size, size, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
    
    gl.framebufferTexture2D(gl.FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture, 0);
    gl.framebufferRenderbuffer(gl.FRAMEBUFFER, gl.DEPTH_ATTACHMENT, gl.RENDERBUFFER, rb);

    render_texture(gl, texture, width, height);

    return fbo;
}

function render_texture(gl, texture, width, height) {

    var buf = init_buffer(gl, width, height);
    var program = init_program(gl);

    render(gl, texture, buf, program, width, height);
}

function render_fram(){
    Module._set_render_dirty(gui); 
    Module._render(gui); 
    requestAnimationFrame(() => {
        render_fram();
    });
}

function render(gl, texture, buf, program, width, height) {

    var impl = function () {
        requestAnimationFrame(impl);    

        gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        
        gl.viewport(0, 0, width, height);

        gl.clearColor(0, 0, 0, 1);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        
        gl.blendFunc(gl.ALPHA, gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

        gl.useProgram(program);

        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, texture);

        gl.enableVertexAttribArray(0);
        gl.bindBuffer(gl.ARRAY_BUFFER, buf.position);
        gl.vertexAttribPointer(0, 3, gl.FLOAT, false, 0, 0);
        
        gl.enableVertexAttribArray(1);
        gl.bindBuffer(gl.ARRAY_BUFFER, buf.uv);
        gl.vertexAttribPointer(1, 2, gl.FLOAT, false, 0, 0);
        
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, buf.indices);
        gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);
    }

    impl();
}


function init_buffer(gl, width, height) {
    var position = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, position);
    var data = new Float32Array([-1, -1, 0, 1, -1, 0, 1, 1, 0, -1, 1, 0]);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
 
    var uv = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, uv);
    data = new Float32Array([0, 0, width/1024, 0, width/1024, height/1024, 0, height/1024]);
    gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
 
    var indices = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indices);
    data = new Uint16Array([0, 1, 2, 0, 2, 3]);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, data, gl.STATIC_DRAW);

    return {
        position, uv, indices
    };
}

function init_program(gl) {
    var vs = gl.createShader(gl.VERTEX_SHADER);
    var code = `
        precision highp float;

        // Attributes
        attribute vec3 position;
        attribute vec2 uv;
        
        // Varyings
        varying vec2 vuv;
        
        void main(void) {
            gl_Position = vec4(position, 1.0);
            vuv = uv;
        }
    `;
    
    gl.shaderSource(vs, code);
    gl.compileShader(vs);

    var fs = gl.createShader(gl.FRAGMENT_SHADER);
    code = `
        precision highp float;

        uniform sampler2D texture;

        // Varyings
        varying vec2 vuv;

        void main(void) {
            gl_FragColor = texture2D(texture, vuv);
            if (gl_FragColor.a < 0.02) {
                discard;
            }
        }
    `;
    gl.shaderSource(fs, code);
    gl.compileShader(fs);

    var program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);

    gl.bindAttribLocation(program, 0, "position");
    gl.bindAttribLocation(program, 1, "uv");

    gl.linkProgram(program);

    let location = gl.getUniformLocation(program, "texture");
    
    gl.useProgram(program);
    gl.uniform1i(location, 0);

    return program;
}

var YGEdgeLeft = 0;
var YGEdgeTop = 1;
var YGEdgeRight = 2;
var YGEdgeBottom = 3;
var YGEdgeStart = 4;
var YGEdgeEnd = 5;
var YGEdgeHorizontal = 6;
var YGEdgeVertical = 7;
var YGEdgeAll = 8;

var YGWrapNoWrap = 0;
var YGWrapWrap = 1; 
var YGWrapWrapReverse = 2;

var RUST_BACKTRACE = 1;
var FitType_None = 0;
var FitType_Fill = 1;
var FitType_Contain = 2;
var FitType_Cover = 3;
var FitType_ScaleDown = 4;

var Text_Align_Left = 0;
var Text_Align_Right = 1;
var Text_Align_Center = 2;
var Text_Align_Justify = 3;

var WhiteSpace_Normal = 0; //	默认。空白会被浏览器忽略(其实是所有的空白被合并成一个空格), 超出范围会换行。
var WhiteSpace_Nowrap = 1; //	空白会被浏览器忽略(其实是所有的空白被合并成一个空格), 超出范围文本也不会换行，文本会在在同一行上继续，直到遇到 <br> 标签为止。
var WhiteSpace_PreWrap = 2; //	保留所有空白符序列，超出范围会换行。
var WhiteSpace_Pre = 3; //	保留空白符，超出范围不会换行(利用yoga无法支持， 暂不支持)
var WhiteSpace_PreLine = 4; //	合并空白符序列，如果存在换行符，优先保留换行符， 超出范围会换行。
 

// 绘制canvas文字， 并更新到纹理
window.__draw_text_canvas = function(world, textInfoList, c){
    setTimeout(function(){
        for (var j = 0; j < textInfoList.list.length; j++) {
        
            var text_info = textInfoList.list[j];
            var canvas = c.canvas;
            var ctx = c.ctx;
            var fontName = text_info.weight + " " + text_info.font_size + "px " + text_info.font;
            var hal_stroke_width = text_info.stroke_width/2;
            var bottom = text_info.size[1] - hal_stroke_width;
            for (var i = 0; i < text_info.chars.length; i++) {
                var char_info = text_info.chars[i];
                canvas.width = char_info.width;
                canvas.height = text_info.size[1]; 
                ctx.fillStyle = "#00f"; 
                ctx.font = fontName;
                ctx.fillRect(0, 0, canvas.width, canvas.height);
                if (text_info.stroke_width > 0.0) {
                    ctx.lineWidth = text_info.stroke_width;
                    ctx.fillStyle = "#0f0";
                    ctx.strokeStyle = "#f00";
                    ctx.textBaseline = "bottom";
                    
                    //fillText 和 strokeText 的顺序对最终效果会有影响， 为了与css text-stroke保持一致， 应该fillText
                    ctx.fillText(char_info.ch, hal_stroke_width, bottom);
                    ctx.strokeText(char_info.ch, hal_stroke_width, bottom);
                } else {
                    ctx.fillStyle = "#0f0";
                    ctx.textBaseline = "bottom";
                    ctx.fillText(char_info.ch, 0, bottom);
    
                }
                window.__jsObj = canvas;
                // document.body.append(canvas);
                Module._update_text_texture(world, char_info.x, char_info.y, canvas.height);
            }
        }
        Module._set_render_dirty(world);
    }, 0);
};

window.__load_image = function(gui, image_name){
    var image = new Image();
    image.onload = () => {
        window.__jsObj = image;
        window.__jsObj1 = image_name;
        var opacity = 0;
        if (image_name.endsWith("png")) {
            opacity = 1;
        }
        Module._load_image_success(gui, opacity , 0);
    };
    image.src = image_name;
}