let text_vs_shader_name = "text_vs";
let text_fs_shader_name = "text_fs";

let text_vs_code = `
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
let text_fs_code = `
    precision highp float;

    // Uniforms
    uniform float alpha;
    uniform vec4 color;
    uniform sampler2D texture;

    // Varyings
    varying vec2 vuv;

    void main(void) {
        vec4 c = color;
        c = texture2D(texture, vuv);
        gl_FragColor = vec4(c.rgb, c.a * alpha);
        gl_FragColor = vec4(c.rgb, 1.0);
    }
`;