use ucd::{Codepoint};

// pub struct TextLayout1{
//     // pub direction: Direction, //设置文本方向
//     pub text_align: TextAlign, //对齐方式
//     pub letter_spacing: f32, //字符间距， 单位：像素
//     pub line_height: LineHeight, //设置行高
//     pub text_indent: f32, // 缩进， 单位： 像素
//     // pub vertical_align: VerticalAlign, //设置元素的垂直对齐
//     pub white_space: WhiteSpace,
// }

pub trait TextLayout {
    // fn direction() -> Direction; //设置文本方向
    fn text_align(&self) -> TextAlign; //对齐方式
    fn letter_spacing(&self) -> f32; //字符间距， 单位：像素
    fn line_height(&self) -> LineHeight; //设置行高
    fn text_indent(&self) -> f32; // 缩进， 单位： 像素
    fn white_space(&self) -> WhiteSpace;
    fn cal_layout<M: FontMeasure>(&self, text: &str, width: f32, font: &M) -> Layout{
        let font_size = font.size();
        let line_height = match self.line_height(){
            LineHeight::Normal => font_size,
            LineHeight::Length(v) => {v},
            LineHeight::Number(v) => {font_size * v},
            LineHeight::Percent(v) => {font_size * v/100.0}
        };

        // match &self.direction {
        //     Direction::Ltr => {
               let uvs = match self.white_space(){
                    WhiteSpace::Pre | WhiteSpace::PreWrap => cal_chars_layout(&self.text_align(), text.chars(), width, font, font_size, line_height),
                    _ => cal_chars_layout(&self.text_align(), filter_white_space(&self.white_space(), text).into_iter(), width, font, font_size, line_height)
                };
        //     },
        //     Direction::Rtl => {
        //         match self.white_space{
        //             WhiteSpace::Pre | WhiteSpace::PreWrap =>  self.cal_chars_layout(text.chars().rev(), width, height, font, font_size, line_height),
        //             _ => self.cal_chars_layout(filter_white_space(&self.white_space, text).into_iter().rev(), width, height, font, font_size, line_height)
        //         };
        //     },
        // }
        
        Layout{
            width: 0.0, 
            height: 0.0,
            uvs: uvs,
        }
    }
}

fn cal_chars_layout<M: FontMeasure, I: Iterator<Item=char>>(text_layout: &TextAlign, chars: I, width: f32, font: &M, font_size: f32, line_height: f32) -> Vec<UV>{
    let mut uvs = Vec::with_capacity(0);
    match text_layout {
        TextAlign::Left => {
            let (mut u1, mut v1) = (0.0, (line_height - font_size)/2.0);
            let mut v2 = v1 + font_size; 
            for c in chars {
                let c_w = font.measure_text(&c); //字符宽度
                let mut u2 =  u1 + c_w;
                if u2 > width {
                    u1 = 0.0;
                    u2 = c_w;
                    v1 += line_height;
                    v2 += line_height;
                }
                uvs.push(UV{u1,v1,u2,v2});
            }
        },
        TextAlign::Right => {
            let (mut u2, mut v1) = (width, (line_height - font_size)/2.0);
            let mut v2 = v1 + font_size; 
            for c in chars {
                let c_w = font.measure_text(&c); //字符宽度
                let mut u1 =  u2 - c_w;
                if u1 < 0.0 {
                    u1 = width - c_w;
                    u2 = width;
                    v1 += line_height;
                    v2 += line_height;
                }
                uvs.push(UV{u1,v1,u2,v2});
            }
        },
        TextAlign::Center => {
            //TODO
            panic!("TextAlign::Center");
        },
        TextAlign::Justify => {
            //TODO
            panic!("TextAlign::Justify");
        },
    }
    uvs
}

// 单词或空白的片段
pub struct SplitWorldSpace<'a> {
    chars: &'a Vec<char>,
    offset: usize,
}

impl<'a> Iterator for SplitWorldSpace<'a> {
    type Item = (usize, usize);
    //返回一个单词或文字或空白符
    fn next(&mut self) -> Option<(usize, usize)> {
        if self.offset == self.chars.len() {
            return None
        }else {
            let c = self.chars[self.offset];
            if c.is_white() || !c.is_alpha() { // 如果字符为单字字符（如中文，韩文，日文）或空白字符， 直接返回该字符
                self.offset += 1;
                return Some((self.offset - 1, 1));
            }else {
                let start = self.offset;
                self.offset += 1;
                loop {
                    if self.offset == self.chars.len() {
                        return Some((start, self.offset - start))
                    }else {
                        let c = self.chars[self.offset];
                        if c.is_white() || !c.is_alpha() {
                            return Some((start, self.offset - start));
                        }else { //是单词的一个元素
                            self.offset += 1;
                        }
                    }
                }
            }
        }
    }
    // add code here
}

//劈分字符串， 返回一个迭代器，该迭代器将返回字符串切片，这些切片是空白符或单词或单字字符
pub fn split_for_word_space(c: &Vec<char>) -> SplitWorldSpace{
    SplitWorldSpace{
        chars: c,
        offset: 0,
    }
}


// //设置文本方向。
// pub enum Direction {
//     Ltr,	//默认。文本方向从左到右。
//     Rtl, //	文本方向从右到左。
// }

//设置行高
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum LineHeight{
    Normal, //设置合理的行间距（等于font-size）
    Length(f32), //固定像素
    Number(f32), //设置数字，此数字会与当前的字体尺寸相乘来设置行间距。
    Percent(f32),   //	基于当前字体尺寸的百分比行间距.
}

//对齐元素中的文本
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum TextAlign{
    Left,	//把文本排列到左边。默认值：由浏览器决定。
    Right,	//把文本排列到右边。
    Center,	//把文本排列到中间。
    Justify,	//实现两端对齐文本效果。
}

//	设置元素的垂直对齐
// pub enum VerticalAlign{
//     // Sub, //	垂直对齐文本的下标。
//     // Super, //	垂直对齐文本的上标
//     // Top,//	把元素的顶端与行中最高元素的顶端对齐
//     // Bottom, //	把元素的底端与行中最低的元素的顶端对齐。
//     BaseBottom(f32),//	默认。元素的底部与基线对齐。
//     BaseMiddle(f32), //	把此元素中间与基线对齐。
//     BaseTop(f32), //	把此元素的顶端与基线对齐。
//     Length(f32), // 元素的底部与基线对齐, 并下降指定像素（该值可以为负， 为负时表示升高）
//     Percent(f32), //	元素的底部与基线对齐, 并下降"line-height" 属性的百分比值（该值可以为负， 为负时表示升高）
// }

//设置元素中空白的处理方式
#[derive(Debug, Clone, Copy, EnumDefault)]
pub enum WhiteSpace{
    Normal, //	默认。空白会被浏览器忽略(其实是所有的空白被合并成一个空格), 超出范围会换行。
    Nowrap, //	空白会被浏览器忽略(其实是所有的空白被合并成一个空格), 超出范围文本也不会换行，文本会在在同一行上继续，直到遇到 <br> 标签为止。
    PreWrap, //	保留所有空白符序列，超出范围会换行。
    Pre, //	保留空白符，超出范围不会换行
    PreLine, //	合并空白符序列，如果存在换行符，优先保留换行符。
}

impl WhiteSpace {
    pub fn allow_wrap(&self) -> bool {
        match *self {
            WhiteSpace::Nowrap |
            WhiteSpace::Pre => false,
            WhiteSpace::Normal |
            WhiteSpace::PreWrap |
            WhiteSpace::PreLine => true,
        }
    }

    pub fn preserve_newlines(&self) -> bool {
        match *self {
            WhiteSpace::Normal |
            WhiteSpace::Nowrap => false,
            WhiteSpace::Pre |
            WhiteSpace::PreWrap |
            WhiteSpace::PreLine => true,
        }
    }

    pub fn preserve_spaces(&self) -> bool {
        match *self {
            WhiteSpace::Normal |
            WhiteSpace::Nowrap |
            WhiteSpace::PreLine => false,
            WhiteSpace::Pre |
            WhiteSpace::PreWrap => true,
        }
    }
}

pub struct Layout{
    pub width: f32, // 文本整体的宽
    pub height: f32, // 文本整体的高
    pub uvs: Vec<UV>  //个字符的uv
}

pub struct UV{
    pub u1: f32, // 左上u
    pub v1: f32, // 左上v
    pub u2: f32, // 左上u
    pub v2: f32, // 左上v
}

pub trait FontMeasure{
    fn measure_text(&self, c: &char) -> f32; //width
    fn size(&self) -> f32; //height
}

fn filter_white_space(white_space: &WhiteSpace, text: &str) -> Vec<char>{
    let mut arr = Vec::new();
    if text.len() == 0 {
        return arr;
    }

    let mut chars = text.chars();
    let mut pre = chars.next().unwrap();
    if !pre.is_white() {
        arr.push(pre);
    }

    match white_space {
        WhiteSpace::Normal | WhiteSpace::Nowrap => {
            loop {
                let next = match chars.next() {
                    Some(p) => p,
                    None => break,
                };
                // 将忽略字符串头部和尾部的所有空白符， 字符之间的空白符将合并成一个空格
                
                if !next.is_white() {
                    if pre.is_white() && arr.len() > 0 {
                        arr.push(space()); 
                    }
                    arr.push(next);
                }
                pre = next;
            }
        },
        WhiteSpace::PreLine => {
            loop {
                let next = match chars.next() {
                    Some(p) => p,
                    None => break,
                };
                if next == next_line() {
                    arr.push(next);
                    pre = next;
                } else if !next.is_white() {
                    if pre.is_white() && pre != next_line() && arr.len() > 0 {
                        arr.push(space()); 
                    }
                    arr.push(next);
                    pre = next;
                } else if !pre.is_white(){
                    pre = next;
                }
            }
        },
        _ => {},
    }
    arr
}

//空格
fn space() -> char{
    char::from(32)
}

//换行
fn next_line() -> char{
    char::from(10)
}

#[test]
fn test_filter_white_space() {
    let white_space = WhiteSpace::Normal;
    assert_eq!(vec_to_str(&filter_white_space(&white_space, "  \n  axxz    yx\t\t\n\txxx\t\n   ")), "axxz yx xxx".to_string());
    let white_space = WhiteSpace::Nowrap;
    assert_eq!(vec_to_str(&filter_white_space(&white_space, "  \n  axxz    yx\t\t\n\txxx\t\n   ")), "axxz yx xxx".to_string());

    let white_space = WhiteSpace::PreLine;
    assert_eq!(vec_to_str(&filter_white_space(&white_space, "  \n\n \n  axxz    yx\t\t\n\t xxx\t\n   ")), "\n\n\naxxz yx\nxxx\n".to_string());
}

#[test]
fn test_ucd() {
    use ucd::Codepoint;
    let c = 'a';
    let c1 = '我';
    let c2 = '장';
    let c3 = 'ρ';
    let c4 = 'A';
    let c5 = 'た';
    
    
    println!("xxxxxxxxxxx:{}", c.is_alpha()); 
    println!("xxxxxxxxxxx:{}", c1.is_alpha());
    println!("xxxxxxxxxxx:{}", c2.is_alpha()); 
    println!("xxxxxxxxxxx:{}", c3.is_alpha());
    println!("xxxxxxxxxxx:{}", c4.is_alpha());
    println!("xxxxxxxxxxx:{}", c5.is_alpha());

    let s = "Löwe 老虎 Léopard";
    assert!(s.is_char_boundary(0));
    // start of `老`
    assert!(s.is_char_boundary(6));
    assert!(s.is_char_boundary(s.len()));

    // second byte of `ö`
    assert!(!s.is_char_boundary(2));

    // third byte of `老`
    assert!(!s.is_char_boundary(8));
    
}

#[test]
fn test_split_for_word_space() {
    let arr = " \nabc xx 我y \nzρz장ρρ ".chars().collect();
    let mut indexs = Vec::new();
    {
        let split = split_for_word_space(&arr);
        for i in split {
            indexs.push(i);
        }
    }
    
    let word = |offset: usize, len: usize|->String{
        let mut s = "".to_string();
        for i in 0..len {
            s += arr[offset + i].to_string().as_str();
        }
        s
    };

    assert_eq!(indexs.len(), 14);
    assert_eq!(word(indexs[0].0, indexs[0].1), " ");
    assert_eq!(word(indexs[1].0, indexs[1].1), "\n");
    assert_eq!(word(indexs[2].0, indexs[2].1), "abc");
    assert_eq!(word(indexs[3].0, indexs[3].1), " ");
    assert_eq!(word(indexs[4].0, indexs[4].1), "xx");
    assert_eq!(word(indexs[5].0, indexs[5].1), " ");
    assert_eq!(word(indexs[6].0, indexs[6].1), "我");
    assert_eq!(word(indexs[7].0, indexs[7].1), "y");
    assert_eq!(word(indexs[8].0, indexs[8].1), " ");
    assert_eq!(word(indexs[9].0, indexs[9].1), "\n");
    assert_eq!(word(indexs[10].0, indexs[10].1), "zρz");
    assert_eq!(word(indexs[11].0, indexs[11].1), "장");
    assert_eq!(word(indexs[12].0, indexs[12].1), "ρρ");
    assert_eq!(word(indexs[13].0, indexs[13].1), " ");
    // for (offset, len) in indexs.iter(){
    //     for i in 0..*len {
    //         s += arr[*offset + i].to_string().as_str();
    //     }
    // }
    // print!("{:?}", s);
}

#[cfg(test)]
fn vec_to_str(arr: &Vec<char>) -> String {
    let mut r = "".to_string();
    for v in arr.iter(){
        let mut b = [0; 2];
        r += v.encode_utf8(&mut b);
    }
    r
}


