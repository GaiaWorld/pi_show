pub fn clip_vertex_shader() -> String{
    r#"
    precision highp float;

    // Attributes
    attribute vec3 position;
    attribute float meshIndex;

    // Uniforms
    uniform float meshNum;

    uniform mat4 view;
    uniform mat4 projection;

    // Varyings
    varying float vPlaneIndex;

    void main(void) {
        
        vPlaneIndex = pow(2.0, meshIndex);
        
        vec4 pos;
        if (meshIndex < meshNum) {
            pos = projection * view * vec4(position, 1.0);
        } else {
            pos = vec4(2.0, 2.0, 2.0, 1.0);
        }

        gl_Position = pos;
    }
    "#.to_string()
}

pub fn clip_fragment_shader() -> String{
    r#"
   precision highp float;

    // Varyings
    varying float vPlaneIndex;

    void main(void) {
        float p = vPlaneIndex / 256.0;
        gl_FragColor = vec4(p);
    }
    "#.to_string()
}