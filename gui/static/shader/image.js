let image_vs_shader_name = "image_vs";
let image_fs_shader_name = "image_fs";

let image_vs_code = `
    precision highp float;

    // Attributes
    attribute vec2 uv;
    attribute vec3 position;
    
    // Uniforms
    uniform vec4 uvOffsetScale;
    uniform mat4 worldMatrix;
    uniform mat4 viewMatrix;
    uniform mat4 projectMatrix;
   
    // Varyings
    varying vec2 vuv;
    
    void main(void) {
        gl_Position = (projectMatrix * viewMatrix * worldMatrix) * vec4(position, 1.0);
        vuv = uvOffsetScale.xy + uv * uvOffsetScale.zw;
    }
`;
let image_fs_code = `
    precision highp float;

    // Uniforms
    uniform float alpha;
    uniform vec4 color;
    uniform sampler2D texture;

    // Varyings
    varying vec2 vuv;

    void main(void) {
        vec4 c = color * texture2D(texture, vuv);
        gl_FragColor = vec4(c.rgb, c.a * alpha);
    }
`;