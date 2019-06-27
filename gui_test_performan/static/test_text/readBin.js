const { readFileSync } = require('fs');

let curOffset = 0;
const data = readFileSync('./21627_21937.json.bin');

const buffer2Arraybuffer = (buffer) => {
    const view = new Uint8Array(buffer);
    return view.buffer;
};

const getHead = (data) => {
    const buf = data.slice(0, 11);
    return buf.toString();
};

const getVersion = (data) => {
    const buf = data.slice(11, 12);
    const view = new Uint8Array(buf);

    return view[0];
};

const getFontName = (data) => {
    const lenBuf = data.slice(12, 13);
    const lenView = new Uint8Array(lenBuf);
    const len = lenView[0];

    // let nameEnd = len % 2 ? 13 + len : 13 + len + 1;
    const nameEnd = 13 + len;
    const nameBuf = data.slice(13, nameEnd);

    curOffset = nameEnd;

    return nameBuf.toString();
};

const getLineHeight = (data) => {
    const buf = data.slice(curOffset, curOffset + 1);
    const view = new Uint8Array(buf);

    curOffset += 1;

    return view[0];
};

const getPictureSize = (data) => {
    let buf = data.slice(curOffset, curOffset + 4);
    buf = buffer2Arraybuffer(buf);
    let view = new Uint8Array(buf);
    view = new Uint16Array(view.buffer);

    curOffset += 4;

    return [view[0], view[1]];
};

const getPadding = (data) => {
    let buf = data.slice(curOffset, curOffset + 8);
    buf = buffer2Arraybuffer(buf);
    const view = new Uint16Array(buf);

    curOffset += 8;

    return [view[0], view[1], view[2], view[3]];
}



const getCharsetMap = (data) => {
    const len = data.byteLength;
    const info = [];
    while(curOffset < len) {
        
        const infoBuf = buffer2Arraybuffer(data.slice(curOffset, curOffset + 12));
        

        const u16View = new Uint16Array(infoBuf, 0, 3);
        const s8View = new Int8Array(infoBuf, 6, 2);
        const u8View = new Uint8Array(infoBuf, 8);
        const [id, x, y] = u16View;
        const [xoffset, yoffset] = s8View;
        const [width, height, advance] = u8View;
        info.push({ id, x, y, xoffset, yoffset, width, height, advance });
        var  xx = new Uint8Array(infoBuf, 0, 2);
        console.log(id, curOffset, xx[0], xx[1]);
        curOffset += 12;
    }

    return info;
};

const parseBin = (data) => {
    const head = getHead(data);
    const version = getVersion(data);
    const fontName = getFontName(data);
    const lineHeight = getLineHeight(data);
    const [width, height] = getPictureSize(data);
    const padding = getPadding(data);
    const charsetMap = getCharsetMap(data);

    return { head, version, fontName, lineHeight, width, height, padding, charsetMap };
}
console.log(parseBin(data));
