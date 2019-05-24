pub fn hello_vertex_shader() -> String{
    r#"
        precision highp float;

        // Attributes
        attribute vec3 position;

        void main(void) {
            gl_Position = vec4(position, 1.0);
        }
    "#.to_string()
}

pub fn hello_fragment_shader() -> String{
    r#"
        precision highp float;

        void main(void) {
            gl_FragColor = vec4(1.0, 0.0, 0.0, 0.0);
        }
    "#.to_string()
}
