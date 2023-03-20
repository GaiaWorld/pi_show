export class AtomTable{
	stringMap /*Map<number, string>*/ = new Map();
	numberMap /*Map<string, number>*/ = new Map();

	get_string(k) {
		return this.stringMap.get(k)
	}

	get_number(k) {
		return this.numberMap.get(k)
	}

	set(n, s) {
		this.stringMap.set(n, s);
		this.numberMap.set(s, n);
	}
}

export const atomTable = new AtomTable();

export function fillBackGround(canvas, ctx, x, y) {
	if (canvas.width < x) {
		canvas.width = x;
	}

	if (canvas.height < y) {
		canvas.height = y;
	}
	
	ctx.fillStyle = "#00f";
	ctx.fillRect(0, 0, canvas.width, canvas.height);
}

export function drawSdf(font/**number */, chars/**Uint32Array */, info/*object*/) {
	
}

export function drawCharWithStroke(ctx, ch_code, x, y) {
	var ch = String.fromCharCode(ch_code);
	//fillText 和 strokeText 的顺序对最终效果会有影响， 为了与css text-stroke保持一致， 应该fillText在前
	ctx.strokeText(ch, x, y);
	ctx.fillText(ch, x, y);
}

export function drawChar(ctx, ch_code, x, y) {
	var ch = String.fromCharCode(ch_code);
	//fillText 和 strokeText 的顺序对最终效果会有影响， 为了与css text-stroke保持一致， 应该fillText在前
    ctx.fillText(ch, x, y);
}

export function setFont(ctx, weight, fontSize, font/*number */, strokeWidth) {
	font = atomTable.get_string(font);
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
		ctx.lineWidth = strokeWidth;
		ctx.strokeStyle = "#f00";
	}
}

export function useVao() {
	var u = navigator.userAgent.toLowerCase(); 
	return u.indexOf("ipad") < 0 && u.indexOf("iphone") < 0;
}

export function measureText(ctx, ch, font_size, name/**number*/) {
	name = atomTable.get_string(name);
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