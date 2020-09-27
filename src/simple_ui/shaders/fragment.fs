precision mediump float;
in vec4 color;
in vec2 texturePos;

out vec4 color_out;

// Includes all characters and icons to be used for rendering.
uniform sampler2D textureAtlas;

void main()
{   
    if (texturePos.x != 0 || texturePos.y != 0) {
        color_out = vec4(color.rgb, texture(textureAtlas, texturePos).r);
    } else {
        color_out = color;
    }
}