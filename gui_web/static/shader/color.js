let color_vs_shader_name = "color_vs";
let color_fs_shader_name = "color_fs";

let color_vs_code = `
    precision highp float;

    // Attributes
    attribute vec3 position;
    
    #ifdef VERTEX_COLOR
        attribute vec4 color;
    #endif
    
    // Uniforms
    uniform mat4 worldMatrix;
    uniform mat4 viewMatrix;
    uniform mat4 projectMatrix;
    uniform float zDepth;
    
    #ifdef VERTEX_COLOR
        varying vec4 vColor; 
    #endif
    
    void main(void) {
        vec4 p = (projectMatrix * viewMatrix * worldMatrix) * vec4(position.x, position.y, 1.0, 1.0);
        gl_Position = vec4(p.x, p.y, zDepth, 1.0);

        #ifdef VERTEX_COLOR
            vColor = color;
        #endif
    }
`;
let color_fs_code = `
    precision highp float;

    // Uniforms
    uniform float blur;
    uniform float alpha;

    #ifdef HSV
        uniform vec3 hsvValue;
    #endif

    #ifdef CLIP
        uniform float clipIndices;
        uniform sampler2D clipTexture;
        uniform float clipTextureSize;
    #endif

    #ifdef VERTEX_COLOR
        varying vec4 vColor;
    #endif

    #ifdef UCOLOR
        uniform vec4 uColor;
    #endif

    #ifdef HSV

        vec3 rgb2hcv(vec3 RGB)
        {
            // Based on work by Sam Hocevar and Emil Persson
            vec4 P = mix(vec4(RGB.bg, -1.0, 2.0/3.0), vec4(RGB.gb, 0.0, -1.0/3.0), step(RGB.b, RGB.g));
            vec4 Q = mix(vec4(P.xyw, RGB.r), vec4(RGB.r, P.yzx), step(P.x, RGB.r));
            float C = Q.x - min(Q.w, Q.y);
            float H = abs((Q.w - Q.y) / (6.0 * C + 1e-10) + Q.z);
            return vec3(H, C, Q.x);
        }

        vec3 rgb2hsv(vec3 RGB)
        {
            vec3 HCV = rgb2hcv(RGB);
            float L = HCV.z - HCV.y * 0.5;
            float S = HCV.y / (1.0 - abs(L * 2.0 - 1.0) + 1e-10);
            return vec3(HCV.x, S, L);
        }

        vec3 hsv2rgb(vec3 c)
        {
            c = vec3(fract(c.x), clamp(c.yz, 0.0, 1.0));
            vec3 rgb = clamp(abs(mod(c.x * 6.0 + vec3(0.0, 4.0, 2.0), 6.0) - 3.0) - 1.0, 0.0, 1.0);
            return c.z + c.y * (rgb - 0.5) * (1.0 - abs(2.0 * c.z - 1.0));
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
            vec4 c = vec4(1.0);
        #ifdef VERTEX_COLOR
            c = c * vColor;
        #endif

        #ifdef UCOLOR
            c = c * uColor;
        #endif
            
        #ifdef HSV
            vec3 hsv = rgb2hsv(c.rgb);
            hsv += hsvValue;
            c.rgb = hsv2rgb(hsv);
        #endif
        
        #ifdef GRAY
            c.rgb = vec3(c.r * 0.299 + c.g * 0.587 + c.b * 0.114);
        #endif

        c.a = c.a * alpha * blur;
        gl_FragColor = c;
    }
`;