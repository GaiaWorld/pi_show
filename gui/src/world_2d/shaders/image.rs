pub fn image_vertex_shader() -> String{
    r#"
    precision highp float;

    // Attributes
    attribute vec2 uv;
    attribute vec3 position;
    
    // Uniforms
    uniform vec4 uvOffsetScale;
    uniform mat4 worldViewProjection;
   
    // Varyings
    varying vec2 vuv;
    
    void main(void) {
        gl_Position = worldViewProjection * vec4(position, 1.0);
        vuv = uvOffsetScale.xy + uv * uvOffsetScale.zw;
    }
    "#.to_string()
}

pub fn image_fragment_shader() -> String{
    r#"
    precision highp float;

    // Uniforms
    uniform vec4 color;
    uniform sampler2D texture;
    
    // Varyings
    varying vec2 vuv;

    void main(void) {
        gl_FragColor = color * texture2D(texture, vuv);
    }
    "#.to_string()
}