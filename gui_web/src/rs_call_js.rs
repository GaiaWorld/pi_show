/// 定义文字绘制， 图片加载的默认js函数， 可以通过条件编译参数去掉本模块， 届时， 用户应该在js中重新定义它们

pub fn define_js(){
    js!{
        window.__draw_text_canvas = function(world, textInfoList, c){
            for (var j = 0; j < textInfoList.list.length; j++) {
                
                var text_info = textInfoList.list[j];

                var canvas = c.canvas;
                var ctx = c.ctx;
                var fontName = text_info.font_size + "px " + text_info.font;
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
                        
                        ctx.strokeText(char_info.ch, hal_stroke_width, bottom);
                        ctx.fillText(char_info.ch, hal_stroke_width, bottom);
                    } else {
                        ctx.fillStyle = "#0f0";
                        ctx.textBaseline = "bottom";
                        ctx.fillText(char_info.ch, 0, bottom);

                    }
                    window.__jsObj = canvas;
                    Module._update_text_texture(world, char_info.x, char_info.y, canvas.height);
                }
            }
            Module._set_render_dirty(world);
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
        };
    }
}