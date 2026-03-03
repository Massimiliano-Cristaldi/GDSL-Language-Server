shader_type canvas_item;

uniform float fade_factor = 1.0;

//This is a single line comment/*
/*
 * This is a multiline comment
 */
void fragment() {
	const float radius = 4.0;
	
	float screen_width = 1.0 / SCREEN_PIXEL_SIZE.x;
	float screen_height = 1.0 / SCREEN_PIXEL_SIZE.y;
	float screen_ratio = screen_width / screen_height;
	
	float x_pos = abs(SCREEN_UV.x - 0.5) * radius * screen_ratio;
	float y_pos = abs(SCREEN_UV.y - 0.5) * radius;
	
	float alpha = sqrt(pow(x_pos, 2) + pow(y_pos, 2) + 0.1);

	COLOR.a = alpha + fade_factor;
}
