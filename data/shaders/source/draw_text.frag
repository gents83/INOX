#define FONT_SIZE 16.
#define CHARACTER_SPACE 1.
#define LINE_SPACE 3.
#define FONT_SCALE vec2(1.0) / (FONT_SIZE/iResolution.xy)
#define FONT_HORIZONTAL_ADVANCE vec2((FONT_SIZE+CHARACTER_SPACE)*text_scale/iResolution.x, 0.)
#define FONT_VERTICAL_ADVANCE -vec2(0., (FONT_SIZE+LINE_SPACE)*text_scale/iResolution.y)

float font_outline = 3.;
float text_scale = 5.;

float drawLine (vec2 start, vec2 end, vec2 uv, float scale)
{
    float result = 0.;
    float size = scale * max((1./iResolution.x),(1./iResolution.y));
    
    float d = distance(start, end);
    float duv = distance(start, uv);
    float interp = clamp(duv/d, 0., 1.);

    //if point is on line, according to dist, it should match current uv 
    result = floor(1.-size+distance(uv, mix(start, end, interp)));
        
    return 1.0 - result;
}

float charH(vec2 pos, vec2 uv, float actual_val) {    
    vec2 rect = vec2(9.,9.) * (FONT_SCALE/text_scale); 
    float res = max(actual_val, drawLine(pos+vec2(0.0,0.0)/rect,pos+vec2(0.0,9.0)/rect, uv, font_outline));
    res = max(res, drawLine(pos+vec2(0.0,4.0)/rect,pos+vec2(6.0,4.0)/rect, uv, font_outline));
    res = max(res, drawLine(pos+vec2(6.0,0.0)/rect,pos+vec2(6.0,9.0)/rect, uv, font_outline));
    return res;
}

// END DATA

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{    
    vec3 background = vec3( 0.0, 0.0, 0.0 );

    vec2 uv = fragCoord.xy / iResolution.xy;
    float aspectRatio = iResolution.x / iResolution.y;

    float res=0.;
   
    vec3 col=vec3(1.,1.,1.);    
    vec2 cpos=vec2(0.2, 0.5);
    
    res = charH(cpos, uv, res);
    cpos += FONT_VERTICAL_ADVANCE;
    res = charH(cpos, uv, res);
    cpos += FONT_HORIZONTAL_ADVANCE;
    res = charH(cpos, uv, res);
        
    fragColor = vec4(vec3(res*col),1.0);
}