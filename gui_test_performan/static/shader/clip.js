let clip_vs_shader_name = "clip_vs";
let clip_fs_shader_name = "clip_fs";

let clip_vs_code = `
    precision highp float;

    // Attributes
    attribute vec3 position;
    attribute float skinIndex; // 网格索引

    // Uniforms
    uniform float meshNum;
    uniform mat4 viewMatrix;
    uniform mat4 projectMatrix;

    // Varyings
    varying float vPlaneIndex;

    void main(void) {
        
        vPlaneIndex = pow(2.0, skinIndex);
        
        vec4 pos;
        if (skinIndex < meshNum) {
            pos = projectMatrix * viewMatrix * vec4(position, 1.0);
        } else {
            pos = vec4(2.0, 2.0, 2.0, 1.0);
        }

        gl_Position = pos;
    }
`;
let clip_fs_code = `
    precision highp float;

    // Varyings
    varying float vPlaneIndex;

    void main(void) {
        float p = vPlaneIndex / 256.0;
        gl_FragColor = vec4(p);        
    }
`;