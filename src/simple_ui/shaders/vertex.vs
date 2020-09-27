in vec3 a_position;
in vec4 a_color;
in vec2 a_texturePos;

out vec4 color;
out vec2 texturePos;

void main()
{
    gl_Position = vec4(a_position, 1.0);
    color = a_color;
    texturePos = a_texturePos;
}