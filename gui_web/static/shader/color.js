let color_vs_shader_name = "color_vs";
let color_fs_shader_name = "color_fs";

let color_vs_code = `
    precision highp float;

    // Attributes
    attribute vec2 position;
    
    #ifdef VERTEX_COLOR
        attribute vec4 color;
    #endif
    
    // Uniforms
    uniform mat4 worldMatrix;
    uniform mat4 viewMatrix;
    uniform mat4 projectMatrix;

    #ifdef CLIP_BOX
        uniform vec4 clipBox;
    #endif
    
    #ifdef VERTEX_COLOR
        varying vec4 vColor; 
    #endif
    
    #ifdef BOX_SHADOW_BLUR
        varying vec2 vPosition; 
    #endif

    #ifdef CLIP_BOX
        varying vec2 vClipBox;
    #endif
    
    void main(void) {
        vec4 p1 = viewMatrix * worldMatrix * vec4(position.x, position.y, 1.0, 1.0);
        vec4 p = projectMatrix * p1;
        
        gl_Position = vec4(p.x, p.y, worldMatrix[3].z, 1.0);

        #ifdef VERTEX_COLOR
            vColor = color;
        #endif

        #ifdef BOX_SHADOW_BLUR
            vPosition = position;
        #endif

        #ifdef CLIP_BOX
            vClipBox = vec2((p1.x - clipBox.x)/clipBox.z, (p1.y - clipBox.y)/clipBox.w);
        #endif
    }
`;

let color_fs_code = `
    precision highp float;

    // Uniforms
    uniform float blur;
    uniform float alpha;

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
    
    #ifdef BOX_SHADOW_BLUR
        uniform vec4 uRect; // xy是矩形最小点的坐标，zw是矩阵最大点的坐标；注：矩形必须排除阴影半径。
    #endif

    #ifdef VERTEX_COLOR
        varying vec4 vColor;
    #endif

    #ifdef UCOLOR
        uniform vec4 uColor;
    #endif

    #ifdef BOX_SHADOW_BLUR
        varying vec2 vPosition; 
    #endif

    #ifdef CLIP_BOX
        varying vec2 vClipBox;
    #endif

    #ifdef BOX_SHADOW_BLUR

        // 从webrender-15版本中拷贝过来的shader

        // An approximation of the error function, which is related to the integral of the Gaussian
        // function:
        //
        //     "erf"(x) = 2/sqrt(pi) int_0^x e^(-t^2) dt
        //              ~~ 1 - 1 / (1 + a_1 x + a_2 x^2 + a_3 x^3 + a_4 x^4)^4
        //
        // where:
        //
        //     a_1 = 0.278393, a_2 = 0.230389, a_3 = 0.000972, a_4 = 0.078108
        //
        // This approximation is accurate to '5 xx 10^-4', more than accurate enough for our purposes.
        //
        // See: https://en.wikipedia.org/wiki/Error_function#Approximation_with_elementary_functions
        float erf(float x) {
            bool negative = x < 0.0;
            if (negative)
                x = -x;
            float x2 = x * x;
            float x3 = x2 * x;
            float x4 = x2 * x2;
            float denom = 1.0 + 0.278393 * x + 0.230389 * x2 + 0.000972 * x3 + 0.078108 * x4;
            float result = 1.0 - 1.0 / (denom * denom * denom * denom);
            return negative ? -result : result;
        }

        // A useful helper for calculating integrals of the Gaussian function via the error function:
        //
        //      "erf"_sigma(x) = 2 int 1/sqrt(2 pi sigma^2) e^(-x^2/(2 sigma^2)) dx
        //                     = "erf"(x/(sigma sqrt(2)))
        float erfSigma(float x, float sigma) {
            return erf(x / (sigma * 1.4142135623730951));
        }

        // Returns the blurred color value from the box itself (not counting any rounded corners). 'p_0' is
        // the vector distance to the top left corner of the box; 'p_1' is the vector distance to its
        // bottom right corner.
        //
        //      "colorFromRect"_sigma(p_0, p_1)
        //          = int_{p_{0_y}}^{p_{1_y}} int_{p_{1_x}}^{p_{0_x}} G_sigma(y) G_sigma(x) dx dy
        //          = 1/4 ("erf"_sigma(p_{1_x}) - "erf"_sigma(p_{0_x}))
        //              ("erf"_sigma(p_{1_y}) - "erf"_sigma(p_{0_y}))
        float colorFromRect(vec2 p0, vec2 p1, float sigma) {
            return (erfSigma(p1.x, sigma) - erfSigma(p0.x, sigma)) *
                (erfSigma(p1.y, sigma) - erfSigma(p0.y, sigma)) / 4.0;
        }

       // The blurred color value for the point at 'pos' with the top left corner of the box at
        // 'p_{0_"rect"}' and the bottom right corner of the box at 'p_{1_"rect"}'.
        float getShadowAlpha(vec2 pos, vec2 ptMin, vec2 ptMax, float sigma) {
            // Compute the vector distances 'p_0' and 'p_1'.
            vec2 dMin = pos - ptMin, dMax = pos - ptMax;

            // Compute the basic color '"colorFromRect"_sigma(p_0, p_1)'. This is all we have to do if
            // the box is unrounded.
            return colorFromRect(dMin, dMax, sigma);
        }
    #endif

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
            
            bvec4 m1, m2, i1, i2;
            bvec4 notM1, notM2;
            
            float remain = clipIndices;
            
            int b = int(remain / 128.0 / 128.0);
            remain = remain - float(b) * 128.0 * 128.0;
            int mask = int(clipColor.b * 256.0);
            
            toBit(mask, m1, m2);
            toBit(b, i1, i2);
            notM1 = bvec4(!m1.x, !m1.y, !m1.z, !m1.w);
            notM2 = bvec4(!m2.x, !m2.y, !m2.z, !m2.w);
            if (!bitAnd(notM1, notM2, i1, i2)) {
                discard;
            }

            int g = int(remain / 128.0);
            remain = remain - float(g) * 128.0;
            mask = int(clipColor.g * 256.0);
            
            toBit(mask, m1, m2);
            toBit(g, i1, i2);
            notM1 = bvec4(!m1.x, !m1.y, !m1.z, !m1.w);
            notM2 = bvec4(!m2.x, !m2.y, !m2.z, !m2.w);
            if (!bitAnd(notM1, notM2, i1, i2)) {
                discard;
            }
            
            int r = int(remain);
            mask = int(clipColor.r * 256.0);
            
            toBit(mask, m1, m2);
            toBit(r, i1, i2);
            notM1 = bvec4(!m1.x, !m1.y, !m1.z, !m1.w);
            notM2 = bvec4(!m2.x, !m2.y, !m2.z, !m2.w);
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
            c.rgb = vec3(c.r * 0.299 + c.g * 0.587 + c.b * 0.114);
        #endif
        
        float blur_size = blur;

        #ifdef BOX_SHADOW_BLUR
            blur_size = 1.0;
            c.a = c.a * getShadowAlpha(vPosition, uRect.xy, uRect.zw, blur / 2.0);
        #endif

        c.a = c.a * alpha * blur_size;

        #ifdef CLIP_BOX
            float factor = min(1.0-abs(vClipBox.x), 1.0-abs(vClipBox.y));
            c.a *= step(0.0, factor);
        #endif

        gl_FragColor = c;

        if (gl_FragColor.a == 0.0) discard;
    }
`;