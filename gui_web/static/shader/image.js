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
         * h: hue，色相，对应css filter的hue-rotate，范围[0, 1]，按比例对应 [0, 360度]
         *    + 比如：0.5对应css的180度，0.7对应CSS的360*0.7=252度
         * s：saturate，饱和度，对应CSS filter的saturate，乘法关系，1.0表示保持包和度不变，0.4对应css filter的40%，2.0对应css filter的200%，等
         *    + 注：如果是css filter的grayscale(x%)，则s要设置的值应为：1 - x/100，比如grayscale(70%)，则s=0.3；
         * v：brightness，明度，对应CSS filter的brightness，乘法关系，1.0表示保持不变，0.4对应css filter的40%，2.0对应css filter的200% 等
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
    
    // #ifdef HSV
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
        // #endif

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
            hsv.g *= hsvValue.g;
            hsv.b *= hsvValue.b;
            c.rgb = hsv2rgb(hsv);
        #endif
        
        vec3 aa = rgb2hsv(c.rgb);
        aa.b *= 1.5;
        c.rgb = hsv2rgb(aa);

        #ifdef GRAY
            c.rgb = vec3(c.r * 0.299 + c.g * 0.587 + c.b * 0.114);
        #endif

        gl_FragColor = vec4(c.rgb, c.a * alpha);
    }
`;