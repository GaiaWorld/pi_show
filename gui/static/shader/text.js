let text_vs_shader_name = "text_vs";
let text_fs_shader_name = "text_fs";

let text_vs_code = `
    precision highp float;

    // Attributes
    attribute vec3 position;
    attribute vec2 uv0;
    #ifdef VERTEX_COLOR
    attribute vec4 color;
    #endif

    // Uniforms
    uniform mat4 worldMatrix;
    uniform mat4 viewMatrix;
    uniform mat4 projectMatrix;

    // Varyings
    #ifdef VERTEX_COLOR
    varying vec4 vColor;
    #endif
    varying vec2 vUV;
    varying vec2 vPosition;

    void main() {
        gl_Position = projectMatrix * viewMatrix * worldMatrix * vec4(position, 1.0);
    #ifdef VERTEX_COLOR
        vColor = color;
    #endif
        vUV = uv0;
        vPosition = position.xy;
    }
`;
let text_fs_code = `
    #extension GL_OES_standard_derivatives : enable

    precision highp float;

    #ifdef CLIP_PLANE
    uniform float clipIndices;
    uniform sampler2D clipTexture;
    uniform float clipTextureSize;
    #endif

    // Varyings
    #ifdef VERTEX_COLOR
    varying vec4 vColor;
    #endif
    
    varying vec2 vUV;
    varying vec2 vPosition;

    // Uniforms
    // uniform float uPxRange;

    uniform float alpha;
    uniform sampler2D texture;
    #ifdef STROKE
    uniform float strokeSize;
    uniform vec4 strokeColor;
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

    float median(float r, float g, float b) {
        return max(min(r, g), min(max(r, g), b));
    }

    void main() {

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

        vec4 c = vec4(1.0);
    #ifdef VERTEX_COLOR        
        c = c * vColor;
    #endif

    #ifdef UCOLOR
        c = c * uColor;
    #endif
    
    vec3 sample = texture2D(texture, vUV).rgb;
        float dist = median(sample.r, sample.g, sample.b);

        // float a = (dist - 0.5) * uPxRange + 0.5;
        // float a = clamp( (dist - 0.5) / fwidth(dist - 0.5) + 0.5, 0.0, 1.0);

        float d = fwidth(dist);
        float a = smoothstep(-d, d, dist - 0.5);
    #ifdef STROKE
        c = mix(strokeColor, c, a);
        a = smoothstep(-d, d, dist - (0.5 - strokeSize));
    #endif
        gl_FragColor = vec4(c.rgb, a * c.a * alpha);
        
        //if (gl_FragColor.a < 0.02) discard;
    }
`;