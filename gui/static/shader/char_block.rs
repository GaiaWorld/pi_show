pub fn char_block_vertex_shader() -> String{
    r#"
        precision highp float;

        // Attributes
        attribute vec3 position;
        attribute vec2 uv;
        
        #ifdef VERTEX_COLOR
            attribute vec4 color;            
        #endif

        // Uniforms
        uniform mat4 world;
        uniform mat4 view;
        uniform mat4 projection;

        // Varyings
        varying vec2 vpos;
        varying vec2 vuv;
        #ifdef VERTEX_COLOR
            varying vec4 vcolor;
        #endif
        
        void main(void) {
            vec4 p = vec4(position, 1.0);
            gl_Position = projection * view * world * p;
            vuv = uv;
            vpos = position.xy;
            #ifdef VERTEX_COLOR
                vcolor = color;
            #endif
        }
    "#.to_string()
}

pub fn char_block_fragment_shader() -> String{
    r#"
        precision highp float;

        // Uniforms
        uniform float fontClamp;   // 0-1的小数，超过这个值即认为有字体，默认传0.75
        uniform float smoothRange; // 0-4，越大越模糊，越小锯齿越明显，默认传1
        
        uniform vec4 sizeRange; // 四边形大小范围，(xmin, ymin, xmax, ymax)
        uniform float alpha;
        
        uniform sampler2D texture;

        #ifdef STROKE
            uniform vec4 strokeColor;
            uniform float strokeClamp; // 0-1的小数，超过这个值即认为有边框
        #endif
        
        #ifdef CLIP_PLANE
            uniform float clipIndices;
            uniform sampler2D clipTexture;
            uniform float clipTextureSize;
        #endif

        #ifdef UCOLOR
            uniform vec4 uColor;
        #endif

        // Varyings
        varying vec2 vpos;
        varying vec2 vuv;
        #ifdef VERTEX_COLOR
            varying vec4 vcolor;
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

            vec2 coord = vpos;

            vec4 c = vec4(1.0);
            #ifdef VERTEX_COLOR
                c = c * vcolor;
            #endif

            #ifdef UCOLOR
                c = c * uColor;
            #endif

            float dist = texture2D(texture, vuv).a;
            float a = smoothstep(fontClamp - smoothRange, fontClamp + smoothRange, dist);
            
    #ifdef STROKE
            c = mix(strokeColor, c, a);
            a = smoothstep(strokeClamp - smoothRange, strokeClamp + smoothRange, dist);
    #endif
            a = a * c.a * alpha;
            if (a < 0.02) {
                discard;
            }

            gl_FragColor = vec4(c.rgb, a);
       }

    "#.to_string()
}