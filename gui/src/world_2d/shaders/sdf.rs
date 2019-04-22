pub fn sdf_vertex_shader() -> String{
    r#"
        precision highp float;

        // Attributes
        attribute vec3 position;

        // Uniforms
        uniform vec2 screenSize;
        uniform mat4 worldViewProjection;
    
        // Varyings
        varying vec2 vpos;

        void main(void) {
            gl_Position = worldViewProjection * vec4(position, 1.0);
            vpos = position.xy;
        }
    "#.to_string()
}

pub fn sdf_fragment_shader() -> String{
    r#"
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
        #ifdef ELLIPSE_COLOR_GRADIENT
            uniform float sizeType;
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
    
        // 0代表最近边，1代表最远边，2代表最近角，3代表最远角
        float getEllipsePercent(vec2 coord, vec2 size, float sizeType) {
            if (sizeType > 1.5) {
                size = size * sqrt(2.0);
            }
            
            float a2 = size.x * size.x;
            float b2 = size.y * size.y;
            
            // y = k * x 和 椭圆的交点到原点距离的平方
            float linearEpsSq;
            
            if (coord.x == 0.0) {
                linearEpsSq = b2;
            } else {
                float k = coord.y / coord.x;
                float k2 = k * k; 
                linearEpsSq = (k2 + 1.0) * a2 * b2 / (b2 + a2 * k2); 
            }
            
            return length(coord) / sqrt(linearEpsSq);
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

            vec4 c;
            float percent;
    #ifdef LINEAR_COLOR_GRADIENT_2
            percent = getLinearPercent(coord, extend, colorAngle);
            c = getColorByPercent2(percent, distance, color1, color2);
    #endif
    #ifdef LINEAR_COLOR_GRADIENT_4
            percent = getLinearPercent(coord, extend, colorAngle);
            c = getColorByPercent4(percent, distance, color1, color2, color3, color4);
    #endif
    #ifdef ELLIPSE_COLOR_GRADIENT
            percent = getEllipsePercent(coord, extend, sizeType);
            c = getColorByPercent4(percent, distance, color1, color2, color3, color4);
    #endif
    #ifdef COLOR
            c = color;
    #endif

            coord = coord / screenSize;
            vec2 size = extend / screenSize;
            
            float d;
    #ifdef SDF_RECT
            float rectRadius = radius / min(screenSize.x, screenSize.y);
            d = sdfRect(coord, size, rectRadius);
    #else
            d = sdfEllipse(coord, size);
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
    "#.to_string()
}