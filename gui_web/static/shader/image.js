let image_vs_shader_name = "image_vs";
let image_fs_shader_name = "image_fs";

let image_vs_code = `
    precision highp float;

    // Attributes
    attribute vec2 uv0;
    attribute vec3 position;
    
    // Uniforms
    uniform mat4 worldMatrix;
    uniform mat4 viewMatrix;
    uniform mat4 projectMatrix;
    uniform float zDepth;
   
    // Varyings
    varying vec2 vuv;
    
    void main(void) {
        vec4 p = (projectMatrix * viewMatrix * worldMatrix) * vec4(position.x, position.y, 1.0, 1.0);
        gl_Position = vec4(p.x, p.y, zDepth, 1.0);
        vuv = uv0;
    }
`;
let image_fs_code = `
    precision highp float;

    // Uniforms
    uniform float alpha;
    uniform sampler2D texture;

    #ifdef HSV
        /**
         * h: hue，色相，取值范围[-0.5, 0.5]，对应Photoshop的[-180, 180]
         *    + 注：最好不要越界，越界效果没有测试过，不保证正确性；
         *    + 注：基本能模拟PS的效果，最多只有1个像素值的误差
         *    + 比如：-0.3对应PS的 2 * -0.3 * 180 = -54度，PS的144度 对应 h的 144 * 0.5 / 180 = 0.4
         * 
         * s：saturate，饱和度，取值范围[-1, 1]，对应Photoshop的[-100, 100]
         *    + 注：当PS中的s>0的时候，公式比较复杂，判断特别多，所以这里用了近似公式。不能完全模拟饱和度
         *    + 注：但是，当s<0的时候（变灰），基本能模拟PS的效果，最多只有1个像素值的误差
         *
         * v：brightness，明度，取值范围[-1, 1]，对应Photoshop的[-100, 100]
         *    + 比如：-0.3对应PS的-30，PS的60对应v=0.6
         */
        uniform vec3 hsvValue;
    #endif

    #ifdef CLIP
    uniform float clipIndices;
    uniform sampler2D clipTexture;
    uniform float clipTextureSize;
    #endif

    // Varyings
    varying vec2 vuv;
    
    #ifdef HSV

    vec3 rgb2hsv(vec3 c)
    {
        vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
        vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
        vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    
        float d = q.x - min(q.w, q.y);
        float e = 1.0e-10;
        return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
    }
    
    vec3 hsv2rgb(vec3 c)
    {
        vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
        vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
        return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
    }
  
    #endif

    #ifdef CLIP 
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
    #endif

    void main(void) {
        #ifdef CLIP
            vec2 clipCoord = gl_FragCoord.xy / clipTextureSize;
            vec4 clipColor = texture2D(clipTexture, vec2(clipCoord));

            int index = int(clipIndices);
            int mask = int(clipColor.r * 256.0) + 128 * int(clipColor.g * 256.0) + 128 * 128 * int(clipColor.b * 256.0);
            
            bvec4 m1, m2, i1, i2;
            toBit(mask, m1, m2);
            toBit(index, i1, i2);

            bvec4 notM1 = bvec4(!m1.x, !m1.y, !m1.z, !m1.w);
            bvec4 notM2 = bvec4(!m2.x, !m2.y, !m2.z, !m2.w);
            if (!bitAnd(notM1, notM2, i1, i2)) {
                discard;
            }
        #endif

        vec4 c = texture2D(texture, vuv);
        
        #ifdef HSV
            vec3 hsv = rgb2hsv(c.rgb);
            hsv.r += hsvValue.r;
            c.rgb = hsv2rgb(hsv);

            // 注：saturate大于0时，公式和PS不大一样
            float gray = max(c.r, max(c.g, c.b)) + min(c.r, min(c.g, c.b));
            c.rgb = mix(c.rgb, vec3(0.5 * gray), -hsvValue.g);

            if (hsvValue.b >= 0.0) {
                c.rgb = mix(c.rgb, vec3(1.0), hsvValue.b);
            } else {
                c.rgb *= 1.0 + hsvValue.b;
            }
        #endif

        #ifdef GRAY
            c.rgb = dot(c.rgb, vec3(0.21, 0.71, 0.07));
        #endif
        
        float gray = max(c.r, max(c.g, c.b)) + min(c.r, min(c.g, c.b));
        c.rgb = mix(c.rgb, vec3(0.5 * gray), -1.9);

        gl_FragColor = vec4(c.rgb, c.a * alpha);        
    }
`;