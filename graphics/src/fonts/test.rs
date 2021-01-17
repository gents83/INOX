
#[test]
fn test_hershey_font() {
    use super::hershey::HersheyFont;
        
    const FONT_PATH: & str = "C:\\PROJECTS\\NRG\\data\\fonts\\futuram.jhf";

    println!();
    println!("- Char H -");
    println!();
    let h = HersheyFont::from_data("8 9MWOMOV RUMUV ROQUQ".as_bytes()).unwrap();
    h.print();

    let buf_h = h.compute_buffer();
    println!("Buffer[{}]", buf_h.len());
    println!();

    println!();
    println!("- Font futuram - ");
    println!();
    let font_data = ::std::fs::read(FONT_PATH).unwrap();
    let font = HersheyFont::from_data(font_data.as_slice()).unwrap();
    font.print();
    
    let buf_font = font.compute_buffer();
    println!("Buffer[{}]", buf_font.len());
}