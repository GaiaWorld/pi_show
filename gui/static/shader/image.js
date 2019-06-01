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
   
    // Varyings
    varying vec2 vuv;
    
    void main(void) {
        gl_Position = (projectMatrix * viewMatrix * worldMatrix) * vec4(position, 1.0);
        vuv = uv0;
    }
`;
let image_fs_code = `
    precision highp float;

    // Uniforms
    uniform float alpha;
    uniform sampler2D texture;

    #ifdef CLIP
    uniform float clipIndices;
    uniform sampler2D clipTexture;
    uniform float clipTextureSize;
    #endif

    // Varyings
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

    void main(void) {
        #ifdef CLIP
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

        vec4 c = texture2D(texture, vuv);
        gl_FragColor = vec4(c.rgb, c.a * alpha);
        gl_FragColor = vec4(c.rgb, 1.0);
    }
`;