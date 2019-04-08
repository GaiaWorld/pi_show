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

    uniform float angles[8];
    uniform vec4 translateScale[8];

    // Varyings
    varying float vPlaneIndex;

    void main(void) {
        
        vPlaneIndex = pow(2.0, meshIndex);
        
        vec4 pos;
        if (meshIndex < meshNum) {
            vec2 p;

            // 缩放
            vec4 ts = translateScale[int(meshIndex)];
            p = position.xy * ts.zw;
            
            // 旋转
            float angle = angles[int(meshIndex)];
            float c = cos(angle), s = sin(angle);
            
            float x = c * p.x - s * p.y;
            float y = s * p.x + c * p.y;

            // 平移
            p = vec2(x, y) + ts.xy;

            pos = projection * view * vec4(p, 0.0, 1.0);
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