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
    
    uniform float uAlpha;
    uniform sampler2D uTexture;
#ifdef STROKE
    uniform float uBorder;
    uniform vec4 uStrokeColor;
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
        v = v * vColor;
#endif
        vec3 sample = texture2D(uTexture, vUV).rgb;
        float dist = median(sample.r, sample.g, sample.b);

        // float a = (dist - 0.5) * uPxRange + 0.5;
        // float a = clamp( (dist - 0.5) / fwidth(dist - 0.5) + 0.5, 0.0, 1.0);

        float d = fwidth(dist);
        float a = smoothstep(-d, d, dist - 0.5);
#ifdef STROKE
        c = mix(uStrokeColor, c, a);
        a = smoothstep(-d, d, dist - (0.5 - uBorder));
#endif
        gl_FragColor = vec4(c.rgb, a * c.a * uAlpha);
        
        if (gl_FragColor.a < 0.02) discard;
    }