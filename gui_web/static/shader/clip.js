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
    varying vec3 vPlaneIndex;
    
    void main(void) {
        
        if (skinIndex < 7.0) {
            vPlaneIndex.r = pow(2.0, skinIndex);
            vPlaneIndex.g = 0.0;
            vPlaneIndex.b = 0.0;
        } else if (skinIndex < 14.0) {
            vPlaneIndex.r = 0.0;
            vPlaneIndex.g = pow(2.0, skinIndex - 7.0);
            vPlaneIndex.b = 0.0;
        } else {
            vPlaneIndex.r = 0.0;
            vPlaneIndex.g = 0.0;
            vPlaneIndex.b = pow(2.0, skinIndex - 14.0);
        }

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
    varying vec3 vPlaneIndex;

    void main(void) {
        gl_FragColor = vec4(vPlaneIndex.xyz / 256.0, 1.0);        
    }
`;