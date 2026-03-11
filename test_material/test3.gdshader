shader_type canvas_item;
render_mode blend_add;

uniform float outline_thickness : hint_range(1.0, 5.0) = 1.0;
uniform float fade_factor = 1.0;

bool has_solid_neighbor(sampler2D texture_sampler, vec2 uv_coord, vec2 pixel_size)
{
	vec2 offset = outline_thickness * pixel_size;
	float sum_alpha = 0.0;

	for (float v = -1.0; v < 2.0; v++) {
		for (float h = -1.0; h < 2.0; h++) {
			sum_alpha += texture(texture_sampler, uv_coord + vec2(h, v) * offset).a;
		}
	}

	return sum_alpha > 0.0;
}

void fragment()
{
	float rb_uv_modifier = cos((UV.x + UV.y) * 15.0);
	float g_uv_modifier = cos((UV.x + UV.y) * 15.0);
	float cos_sign = rb_uv_modifier > 0.0 ? 1.0 : 0.0;

	float red_base = 0.215;
	float red_modifier = cos_sign * rb_uv_modifier * (1.0 - red_base);
	float final_red = red_base + red_modifier;

	float green_base = 0.757;
	float green_modifier = 0.150 * g_uv_modifier;
	float final_green = green_base + green_modifier;

	float blue_base = 0.921;
	float blue_modifier = cos_sign * rb_uv_modifier * (1.0 - blue_base);
	float final_blue = blue_base + blue_modifier;

	vec4 final_color = vec4(final_red, final_green, final_blue, 0.8 - fade_factor);
	vec4 white = vec4(1.0, 1.0, 1.0, 1.0 - fade_factor);

	if (has_solid_neighbor(TEXTURE, UV, TEXTURE_PIXEL_SIZE)) {
		COLOR = COLOR.a > 0.1 ? final_color : white;
	}
}