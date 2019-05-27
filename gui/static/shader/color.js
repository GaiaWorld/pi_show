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

    // Varyings
    varying vec2 vpos;
    
    #ifdef VERTEX_COLOR
        varying vec4 vColor;
    #endif
    
    void main(void) {
        gl_Position = (projectMatrix * viewMatrix * worldMatrix) * vec4(position, 1.0);
        vpos = position.xy;

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

    // Varyings
    varying vec2 vpos;

    #ifdef VERTEX_COLOR
        varying vec4 vColor;
    #endif

    #ifdef UCOLOR
        uniform vec4 uColor;
    #endif

    void main(void) {

        vec4 c = vec4(1.0);
    #ifdef VERTEX_COLOR
        c = c * vColor;
    #endif

    #ifdef UCOLOR
        c = c * uColor;
    #endif
        
        c.a = c.a * alpha * blur;
        gl_FragColor = vColor;
    }
`;