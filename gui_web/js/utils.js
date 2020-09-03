export function fillBackGround(canvas, ctx, x, y) {
	canvas.width = x;
	canvas.height = y;
	ctx.fillStyle = "#00f";
	ctx.fillRect(0, 0, canvas.width, canvas.height);
}

export function drawCharWithStroke(ctx, ch_code, x) {
	var ch = String.fromCharCode(ch_code);
	//fillText 和 strokeText 的顺序对最终效果会有影响， 为了与css text-stroke保持一致， 应该fillText在前
	ctx.strokeText(ch, x, 0);
	ctx.fillText(ch, x, 0);
}

export function drawChar(ctx, ch_code, x) {
	var ch = String.fromCharCode(ch_code);
	//fillText 和 strokeText 的顺序对最终效果会有影响， 为了与css text-stroke保持一致， 应该fillText在前
    ctx.fillText(ch, x, 0);
}

export function setFont(ctx, weight, fontSize, font, strokeWidth) {
	var weight;
	if (weight <= 300 ) {
		weight = "lighter";
	} else if (weight < 700 ) {
		weight = "normal";
	} else if (weight < 900 ) {
		weight = "bold";
	} else {
		weight = "bolder";
	}
	ctx.font = weight + " " + fontSize + "px " + font;
	ctx.fillStyle = "#0f0";
	ctx.textBaseline = "top";

	if(strokeWidth > 0) {
		ctx.lineWidth = stroke_width;
		ctx.strokeStyle = "#f00";
	}
}

export function set_class(world, node, class_arr){
	// console.log("_set_class", node, class_arr);
	var old = Module._add_class_start(world, node);
	for (var i = 0; i < class_arr.length; i++) {
		Module._add_class(world, node, class_arr[i], i);
	}
	Module._add_class_end(world, node, old);
}

export function __load_image(gui, image_name, r_type){
	var image = new Image();
	image.onload = function() {
		window.__jsObj = image;
		window.__jsObj1 = image_name;
		var opacity = 0;
		if (image_name.endsWith("png")) {
			opacity = 1;
		}
		Module._load_image_success(gui, opacity, -1, r_type);
	};
	image.src = image_name;
};

export function useVao() {
	var u = navigator.userAgent.toLowerCase(); 
	return u.indexOf("ipad") < 0 && u.indexOf("iphone") < 0;
}

export function measureText(ctx, ch, font_size, name) {
	ctx.font = font_size + "px " + name;
	return ctx.measureText(String.fromCharCode(ch)).width;
}

export function loadImage(image_name, callback) {
	var image = new Image();
	image.onload = function() {
		callback(image_name.endsWith("png")?1:0, -1, 0, image_name, image.width, image.height, image);
	};
	image.src = image_name;
}