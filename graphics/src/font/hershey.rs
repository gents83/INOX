/*
    USE RESTRICTION:
    ----------------------------------------------------------------
	This distribution of the Hershey Fonts may be used by anyone for
	any purpose, commercial or otherwise, providing that:
		1. The following acknowledgements must be distributed with
			the font data:
			- The Hershey Fonts were originally created by Dr.
				A. V. Hershey while working at the U. S.
				National Bureau of Standards.
			- The format of the Font data in this distribution
				was originally created by
					James Hurt
					Cognition, Inc.
					900 Technology Park Drive
					Billerica, MA 01821
					(mit-eddie!ci-dandelion!hurt)
		2. The font data in this distribution may be converted into
			any other format *EXCEPT* the format distributed by
			the U.S. NTIS (which organization holds the rights
			to the distribution and use of the font data in that
			particular format). Not that anybody would really
			*want* to use their format... each point is described
			in eight bytes as "xxx yyy:", where xxx and yyy are
            the coordinate values as ASCII numbers.
    ----------------------------------------------------------------
    The format of the files is described as follows:

    The structure is bascially as follows: each character consists of 
    - a number 1->4000 (not all used) in column 0:4, 
    - the number of vertices in columns 5:7, 
    - the left hand position in column 8, 
    - the right hand position in column 9, 
    - and finally the vertices in single character pairs. 
    All coordinates are given relative to the ascii value of 'R'. 
    If the coordinate value is " R" that indicates a pen up operation.
    As an example consider the 8th symbol

    8 9MWOMOV RUMUV ROQUQ

    It has 9 coordinate pairs (this includes the left and right position).
    The left position is 'M' - 'R' = -5
    The right position is 'W' - 'R' = 5
    The first coordinate is "OM" = (-3,-5)
    The second coordinate is "OV" = (-3,4)
    Raise the pen " R"
    Move to "UM" = (3,-5)
    Draw to "UV" = (3,4)
    Raise the pen " R"
    Move to "OQ" = (-3,-1)
    Draw to "UQ" = (3,-1)
    Drawing this out on a piece of paper will reveal it represents an 'H'
*/

use nrg_math::*;

const TAB: char = '\t';
const NEW_LINE: char = '\n';
const RETURN_LINE: char = '\r';

const PEN_UP: char = 'R';
const MAX_GLYPH_ID_LENGTH: usize = 5;
const MAX_GLYPH_COORDS_LENGTH: usize = 8;
const FIRST_ASCII_CHAR: usize = 32;


struct Glyph {
    id: u32,
    width: Vector2i,
    rect: Vector4i,
    data: Vec<Vector2i>,
}

pub struct HersheyFont {
    glyphs: Vec<Glyph>,
}

impl HersheyFont {
    pub fn from_data(data: &[u8]) -> Option<Self> {
        let mut glyphs: Vec<Glyph> = Vec::new();
        let mut glyph_index:usize = 0;

        let mut pos_index = 0;
        let mut i:usize = 0;
        let mut min_y = i32::MAX;

        let c:Vec<char> = data.iter().map(|c| *c as char).collect();
        while i < c.len() {
            
            i = Self::skip_whitespaces(i, &mut pos_index, &c);

            let mut str = String::default();
            while pos_index < MAX_GLYPH_ID_LENGTH && c[i].is_numeric() {
                str.push(c[i]);
                i = Self::advance(i, &mut pos_index, &c);
            }
            let glyph_id: u32 = Self::convert(&str); 
            
            i = Self::skip_whitespaces(i, &mut pos_index, &c);
            
            str = String::default();
            while pos_index < MAX_GLYPH_COORDS_LENGTH && (c[i].is_numeric() || c[i].is_whitespace()) {
                if c[i].is_numeric() {
                    str.push(c[i]);
                }
                i = Self::advance(i, &mut pos_index, &c);
            }
            let num_coords: u32 = Self::convert(&str); 
                        
            i = Self::skip_whitespaces(i, &mut pos_index, &c);

            let left = Self::hershey_val_conversion( c[i] );
            i = Self::advance(i, &mut pos_index, &c);
            let right = Self::hershey_val_conversion( c[i] );
            i = Self::advance(i, &mut pos_index, &c);

            glyphs.push( Glyph {
                id: glyph_id as _,
                width: Vector2i::new(left, right),
                rect: Vector4i::default(),
                data: Vec::new(),
            });

            let mut pairs: i32 = num_coords as i32 - 1;
            let mut is_pen_up: bool = true;
            let mut pos: Vector2i = Vector2i::default();
            let mut rect: Vector4i = Vector4i::new(i32::MAX, i32::MAX, -i32::MAX, -i32::MAX);

            while pairs > 0 {     
                pairs -= 1;

                let c1 = c[i];
                i = Self::advance(i, &mut pos_index, &c);
                let c2 = c[i];
                i = Self::advance(i, &mut pos_index, &c);

                let x = Self::hershey_val_conversion( c1 );
                let y = Self::hershey_val_conversion( c2 );

                if !is_pen_up {
                    if c1.is_whitespace() && c2 == PEN_UP {
                        is_pen_up = true;
                    }
                    else {
                        glyphs[glyph_index].data.push(pos);
                        pos = Vector2i::new(x, y);
                        glyphs[glyph_index].data.push(pos);
                        rect.x = i32::min(rect.x, x);
                        rect.y = i32::min(rect.y, y);
                        rect.z = i32::max(rect.z, x);
                        rect.w = i32::max(rect.w, y);
                    }
                }
                else {
                    is_pen_up = false;
                    pos = Vector2i::new(x, y);
                    rect.x = i32::min(rect.x, x);
                    rect.y = i32::min(rect.y, y);
                    rect.z = i32::max(rect.z, x);
                    rect.w = i32::max(rect.w, y);
                }
            }
            if glyphs[glyph_index].data.is_empty() {
                glyphs[glyph_index].data.push(pos);
                glyphs[glyph_index].data.push(pos);
                rect = Vector4i::new(0,0,0,0);
            }
            glyphs[glyph_index].rect = rect;

            min_y = i32::min(min_y, rect.y);
            glyph_index += 1;
        } 

        Self::update_data(&mut glyphs, min_y);

        Some(HersheyFont {
            glyphs,
        })
    }

    pub fn print(&self) {
        for (index, g) in self.glyphs.iter().enumerate() {
            println!("\nGlyph[{}] #{}:", g.id, index+FIRST_ASCII_CHAR);
            println!("Num Coords: {}", g.data.len());
            println!("Width = {} ", g.width);
            println!("Real Width = {} ", g.rect.x.abs() + g.rect.z.abs());
            println!("Num Pairs = {} ", g.data.len());    
            for v in g.data.iter() {
                print!("{} ", v);  
            }
            println!();
        }
    }

    pub fn compute_index(&self, c: char) -> usize {
        let mut index = 0;
        
        for g in self.glyphs.iter() {
            if g.id == c as u32 {
                break;
            }
            index += g.data.len() * 2;
            index += 2;
        }

        index
    }

    pub fn compute_buffer(&self) -> Vec<f32> {
        let mut data:Vec<f32> = Vec::new();
        
        for g in self.glyphs.iter() {
            data.push(g.data.len() as f32 * 2.0);
            data.push(g.rect.x.abs() as f32 + g.rect.z.abs() as f32);
            for v in g.data.iter() {
                data.push(v.x as f32);
                data.push(v.y as f32);
            }
        }

        data
    }
}


impl HersheyFont {

    fn update_data(glyphs: &mut Vec<Glyph>, min_y: i32) {
        for g in glyphs {
            for v in g.data.iter_mut() {
                v.x += -g.rect.x;
                v.y += -min_y;
            }
        }
    }

    fn advance(mut i: usize, pos_index:&mut usize, c: &[char]) -> usize {
        *pos_index += 1;
        i += 1;
        if i >= c.len() {
            return c.len() as _
        }
        while c[i] == NEW_LINE || c[i] == RETURN_LINE {
            *pos_index = 0;
            i += 1;
            if i >= c.len() {
                return c.len() as _
            }
        }
        i
    }

    fn skip_whitespaces(mut i: usize, pos_index:&mut usize, c: &[char]) -> usize {
        while i < c.len() && c[i].is_whitespace() || c[i] == TAB || c[i] == NEW_LINE || c[i] == RETURN_LINE {
            if c[i] == NEW_LINE || c[i] == RETURN_LINE {
                *pos_index = 0;
            }
            else {
                *pos_index += 1;
            }  
            i += 1;              
        } 
        i
    }

    fn convert<T>(s: &str) -> T 
    where T: std::str::FromStr + Sized + std::fmt::Debug + Default {
        match s.parse::<T>() {
            Ok(a) => a,
            _ => {
                println!("Error reading chars in string {}", s);
                T::default()
            },
        }
    }

    fn hershey_val_conversion(character: char) -> i32 {
        character as i32 - 'R' as i32
    }
}