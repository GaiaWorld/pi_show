   `
    precision highp float;

    // Attributes
    attribute vec3 position;
    attribute vec2 uv0;
#ifdef VERTEX_COLOR
    attribute vec4 color;
#endif

    // Uniforms
    uniform mat4 world;
    uniform mat4 view;
    uniform mat4 projection;

    // Varyings
#ifdef VERTEX_COLOR
    varying vec4 vColor;
#endif
    varying vec2 vUV;
    varying vec2 vPosition;

    void main() {
        gl_Position = projection * view * world * vec4(position, 1.0);
#ifdef VERTEX_COLOR
        vColor = color;
#endif
        vUV = uv0;
        vPosition = position.xy;
    }