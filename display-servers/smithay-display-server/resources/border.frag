precision mediump float; // set the percicion for floats
uniform vec2 size; // Size of the viewport to render to, always given by smithay
varying vec2 v_coords; // Location of the fragment (pixel), always given by smithay

// These are given by leftway
uniform vec3 color;
uniform float thickness;
uniform float halfThickness;


void main() {
    // vec2 center = size / 2.0 - vec2(0.5);
    // vec2 location = v_coords * size;
    // vec4 mix_color;

    // float distance = max(abs(location.x - center.x) - (size.x / 2.0 - halfThickness), abs(location.y - center.y) - (size.y / 2.0 - halfThickness));
    // float smoothedAlpha = 1.0 - smoothstep(0.0, 1.0, abs(distance) - (halfThickness));

    // mix_color = mix(vec4(0.0, 0.0, 0.0, 0.0), vec4(color, smoothedAlpha), smoothedAlpha);

    gl_FragColor = vec4(color, 1.0);
}

