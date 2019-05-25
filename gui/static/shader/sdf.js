let sdf_vs_shader_name = "sdf_vs";
let sdf_fs_shader_name = "sdf_fs";

let sdf_vs_code = `
    precision highp float;

    // Attributes
    attribute vec3 position;
    
    #ifdef VERTEX_COLOR
        attribute vec4 color;
    #endif
    
    // Uniforms
    uniform vec2 screenSize;
    uniform mat4 worldMatrix;
    uniform mat4 viewMatrix;
    uniform mat4 projectMatrix;

    // Varyings
    varying vec2 vpos;
    
    #ifdef VERTEX_COLOR
        varying vec4 vcolor;
    #endif
    
    void main(void) {
        gl_Position = (projectMatrix * viewMatrix * worldMatrix) * vec4(position, 1.0);
        vpos = position.xy;

        #ifdef VERTEX_COLOR
            vcolor = color;
        #endif

    }
`;
let sdf_fs_code = `
    precision highp float;

    // Uniforms
    uniform float blur;
    uniform vec2 extend;
    uniform float alpha;
    uniform vec2 screenSize;

    #ifdef SDF_RECT
        uniform float radius;
    #endif

    #ifdef STROKE
        uniform float strokeSize;
        uniform vec4 strokeColor;
    #endif
    
    #ifdef CLIP_PLANE
        uniform float clipIndices;
        uniform sampler2D clipTexture;
        uniform float clipTextureSize;
    #endif

    #ifdef

    // Varyings
    varying vec2 vpos;

    #ifdef VERTEX_COLOR
        varying vec4 vcolor;
    #endif

    #ifdef UCOLOR
        uniform vec4 uColor;
    #endif

    // 8位int型变二进制数组
    void toBit(int num, out bvec4 r1, out bvec4 r2) {
        for (int i = 0; i < 4; ++i) {
            r1[i] = (num / 2) * 2 != num;
            num = (num - int(r1[i])) / 2;
        }

        for (int i = 0; i < 4; ++i) {
            r2[i] = (num / 2) * 2 != num;
            num = (num - int(r2[i])) / 2;
        }
    }

    // 做与运算，返回true表示通过
    bool bitAnd(in bvec4 a1, in bvec4 a2, in bvec4 b1, in bvec4 b2) {
        
        bvec4 v1 = bvec4(a1.x && b1.x, a1.y && b1.y, a1.z && b1.z, a1.w && b1.w);
        bvec4 v2 = bvec4(a2.x && b2.x, a2.y && b2.y, a2.z && b2.z, a2.w && b2.w);

        return v1 == bvec4(false) && v2 == bvec4(false);
    }

    float sdfRect(vec2 coord, vec2 size, float r) {
        vec2 d = abs(coord) - size;
        return min(max(d.x, d.y), 0.0) + length(max(d, 0.0)) - r;
    }

    // http://iquilezles.org/www/articles/ellipsedist/ellipsedist.htm
    // https://www.shadertoy.com/view/4sS3zz

    float sdfEllipse(vec2 coord, in vec2 radius)
    {
        coord = abs(coord); 
        if(coord.x > coord.y) {
            coord = coord.yx;
            radius = radius.yx; 
        }
        
        float l = radius.y * radius.y - radius.x * radius.x;
        
        float m = radius.x * coord.x / l; 
        float n = radius.y * coord.y / l; 
        float m2 = m * m;
        float n2 = n * n;
        
        float c = (m2 + n2 - 1.0) / 3.0; 
        float c3 = c * c * c;
    
        float q = c3 + m2 * n2 * 2.0;
        float d = c3 + m2 * n2;
        float g = m + m * n2;
    
        float co;
    
        if(d < 0.0)
        {
            float h = acos(q / c3) / 3.0;
            float s = cos(h);
            float t = sin(h) * sqrt(3.0);
            float rx = sqrt( -c * (s + t + 2.0) + m2 );
            float ry = sqrt( -c * (s - t + 2.0) + m2 );
            co = (ry + sign(l) * rx + abs(g) / (rx * ry) - m) / 2.0;
        }
        else
        {
            float h = 2.0 * m * n * sqrt(d);
            float s = sign(q + h) * pow( abs(q + h), 1.0 / 3.0 );
            float u = sign(q - h) * pow(abs(q - h), 1.0 / 3.0 );
            float rx = -s - u - c * 4.0 + 2.0 * m2;
            float ry = (s - u) * sqrt(3.0);
            float rm = sqrt(rx * rx + ry * ry);
            co = (ry / sqrt(rm - rx) + 2.0 * g / rm - m) / 2.0;
        }
    
        float si = sqrt(1.0 - co * co);
    
        vec2 r = radius * vec2(co, si);
        
        return length(r - coord) * sign(coord.y - r.y);
    }

    void main(void) {

#ifdef CLIP_PLANE

        vec2 clipCoord = gl_FragCoord.xy / clipTextureSize;
        vec4 clipColor = texture2D(clipTexture, vec2(clipCoord));

        int index = int(clipIndices);
        int mask = int(clipColor.r * 256.0);
        
        bvec4 m1, m2, i1, i2;
        toBit(mask, m1, m2);
        toBit(index, i1, i2);

        bvec4 notM1 = bvec4(!m1.x, !m1.y, !m1.z, !m1.w);
        bvec4 notM2 = bvec4(!m2.x, !m2.y, !m2.z, !m2.w);
        if (!bitAnd(notM1, notM2, i1, i2)) {
            discard;
        }
#endif

        // gl_FragCoord的范围是[0, screenSize)，需要变成 [-screenSize/2, screenSize/2)
        vec2 coord = vpos;

        coord = coord / screenSize;
        vec2 size = extend / screenSize;

vec4 c = vec4(1.0);
#ifdef VERTEX_COLOR
    c = c * vcolor;
#endif

#ifdef UCOLOR
    c = c * uColor;
#endif

        float d;
#ifdef SDF_RECT
        float rectRadius = radius / min(screenSize.x, screenSize.y);
        d = sdfRect(coord, 2.0 * size, rectRadius);
#else
        d = sdfEllipse(coord, 2.0 * size);
#endif
        float antiBody = 1.0 - smoothstep(-0.002 * blur, 0.002 * blur, d);
        c.a = c.a * antiBody;

#ifdef STROKE
        vec2 fsStrokeSize = vec2(strokeSize / screenSize);
    
    #ifdef SDF_RECT
        d = sdfRect(coord, size + fsStrokeSize, rectRadius);
    #else
        d = sdfEllipse(coord, size + fsStrokeSize);
    #endif

        vec4 sc = strokeColor;
        float antiStroke = 1.0 - smoothstep(-0.002 * blur, 0.002 * blur, d);
        sc.a = sc.a * antiStroke;
        c = mix(sc, c, antiBody);        
#endif
        
        c.a = c.a * alpha;
        if (c.a < 0.02) {
            discard;
        }
        
        gl_FragColor = c;
    }
`;