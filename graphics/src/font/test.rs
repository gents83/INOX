

const FONT_PATH: &'static str = "C:\\PROJECTS\\NRG\\data\\fonts\\BasicFont.ttf";

#[test]
fn test_font()
{ 
    use super::font::*;

    let font = Font::create_from(FONT_PATH);
}