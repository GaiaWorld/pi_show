pub fn hello_vertex_shader() -> String{
    r#"
        precision highp float;

        // Attributes
        attribute vec3 position;

        uniform vec3 uPosition;

        void main(void) {
            gl_Position = vec4(position + uPosition, 1.0);
        }
    "#.to_string()
}

pub fn hello_fragment_shader() -> String{
    r#"
        precision highp float;
        
        uniform vec4 uColor;

        #ifdef ALPHA
            uniform float uAlpha;
        #endif

        void main(void) {
            float alpha = 1.0;
            #ifdef ALPHA
                alpha *= uAlpha;    
            #endif
            gl_FragColor = vec4(uColor.rgb, alpha * uColor.a);
            gl_FragColor = vec4(1.0, 1.0, 0.0, 0.5);
        }
    "#.to_string()
}
