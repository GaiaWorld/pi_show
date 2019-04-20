pub fn char_block_vertex_shader() -> String{
    r#"
        precision highp float;

        // Attributes
        attribute vec3 position;
        attribute vec2 uv;

        // Uniforms
        uniform mat4 world;
        uniform mat4 view;
        uniform mat4 projection;

        // Varyings
        varying vec2 vpos;
        varying vec2 vuv;

        void main(void) {
            gl_Position = projection * view * world * vec4(position, 1.0);
            vuv = uv;
            vpos = position.xy;
        }
    "#.to_string()
}

pub fn char_block_fragment_shader() -> String{
    r#"
        precision highp float;

        // Uniforms
        uniform float fontClamp;   // 0-1的小数，超过这个值即认为有字体，默认传0.75
        uniform float smoothRange; // 0-4，越大越模糊，越小锯齿越明显，默认传1

        uniform vec2 extend; // 四边形大小
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

        #ifdef LINEAR_COLOR_GRADIENT_2
            uniform float colorAngle;
            uniform vec2 distance;
            uniform vec4 color1;
            uniform vec4 color2;
        #endif
        #ifdef LINEAR_COLOR_GRADIENT_4
            uniform float colorAngle;
            uniform vec4 distance;
            uniform vec4 color1;
            uniform vec4 color2;
            uniform vec4 color3;
            uniform vec4 color4;
        #endif
        #ifdef COLOR
            uniform vec4 color;
        #endif

        // Varyings
        varying vec2 vpos;
        varying vec2 vuv;
        
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

        vec4 getColorByPercent2(float percent, vec2 d, vec4 c1, vec4 c2) {
            vec4 color;
            if (percent < d.x) { 
                color = c1;
            } else if (percent < d.y) {
                percent = (percent - d.x) / (d.y - d.x);
                color = mix(c1, c2, percent);
            } else {
                color = c2;
            }
            return color;
        }

        vec4 getColorByPercent4(float percent, vec4 d, vec4 c1, vec4 c2, vec4 c3, vec4 c4) {
            vec4 color;
            if (percent < d.x) { 
                color = c1;
            } else if (percent < d.y) {
                percent = (percent - d.x) / (d.y - d.x);
                color = mix(c1, c2, percent);
            } else if (percent < d.z) { 
                percent = (percent - d.y) / (d.z - d.y);
                color = mix(c2, c3, percent);
            } else if (percent < d.w) { 
                percent = (percent - d.z) / (d.w - d.z);
                color = mix(c3, c4, percent);
            } else {
                color = c4;
            }
            return color;
        }

        float getLinearPercent(vec2 coord, vec2 size, float angle) {
            
            vec2 dir = vec2(sin(angle), cos(angle));

            float dmax, dmin;
            dmax = dmin = dot(dir, size);

            float temp = dot(dir, vec2(size.x, -size.y));
            if (temp > dmax) dmax = temp;
            if (temp < dmin) dmin = temp;
            
            temp = dot(dir, vec2(-size.x, -size.y));
            if (temp > dmax) dmax = temp;
            if (temp < dmin) dmin = temp;

            temp = dot(dir, vec2(-size.x, +size.y));
            if (temp > dmax) dmax = temp;
            if (temp < dmin) dmin = temp;
            
            float d = dot(dir, coord);
            return (d - dmin) / (dmax - dmin);
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
            float percent;
    #ifdef LINEAR_COLOR_GRADIENT_2
            percent = getLinearPercent(coord, extend, colorAngle);
            c = getColorByPercent2(percent, distance, color1, color2);
    #endif
    #ifdef LINEAR_COLOR_GRADIENT_4
            percent = getLinearPercent(coord, extend, colorAngle);
            c = getColorByPercent4(percent, distance, color1, color2, color3, color4);
    #endif
    #ifdef COLOR
            c = color;
    #endif

            float dist = texture2D(texture, vuv).a;
            float range = smoothRange * 1.4142 / extend[0];
            float a = smoothstep(fontClamp - range, fontClamp + range, dist);
            
    #ifdef STROKE
            c = mix(strokeColor, c, a);
            a = smoothstep(strokeClamp - range, strokeClamp + range, dist);
    #endif
            a = a * c.a * alpha;
            gl_FragColor = vec4(c.rgb, dist);
       }

    "#.to_string()
}