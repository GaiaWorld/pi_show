use std::cmp::Ordering;

use ucd::Codepoint;


/// 中文（包括日文韩文同用）的范围
const CASED_RANGE: [(usize, usize);1] = [(0x4e00,0x9fa5)];
/// 大写字符的范围
const CAP_RANGE: [(usize, usize);1] = [(65,90)];
/// 分隔符的范围（非字母数字的字符） (8216, 8221)-‘”  (12289, 12290)-。  (65292, 65307)-，；
const SEPARATOR_RANGE: [(usize, usize);7] = [(0,47), (58, 64), (91, 96), (123, 256), (8216, 8221), (12289, 12290), (65292, 65307)];
/// 空白符的范围, 12288为中文空格
const WHITESPACE_RANGE: [(usize, usize);2] = [(127,256), (12288, 12288)]; 

/// 字符是否为单字
pub fn is_cased(c: char) -> bool{
    let c = c as usize;
    if c < 0xff {
        return false;
    }
    match CASED_RANGE.binary_search_by(|&(start, end)| {
        if c < start {
            Ordering::Less
        }else if c > end {
            Ordering::Greater
        }else{
            Ordering::Equal
        }
    }) {
        Ok(_) => true,
        _ => false
    }
}
/// 字符是否为大写
pub fn is_capitalization(c: char) -> bool{
    let c = c as usize;
    match CAP_RANGE.binary_search_by(|&(start, end)| {
        if c < start {
            Ordering::Less
        }else if c > end {
            Ordering::Greater
        }else{
            Ordering::Equal
        }
    }) {
        Ok(_) => true,
        _ => false
    }
}
/// 字符是否为分隔符
pub fn is_separator(c: char) -> bool{
    if is_whitespace(c) {
        return true
    }
    let c = c as usize;
    match SEPARATOR_RANGE.binary_search_by(|&(start, end)| {
        if c < start {
            Ordering::Less
        }else if c > end {
            Ordering::Greater
        }else{
            Ordering::Equal
        }
    }) {
        Ok(_) => true,
        _ => false
    }
}
/// 字符是否为空白符
pub fn is_whitespace(c: char) -> bool{
    let c = c as usize;
    if c < 33 {
        return true;
    }
    match WHITESPACE_RANGE.binary_search_by(|&(start, end)| {
        if c < start {
            Ordering::Less
        }else if c > end {
            Ordering::Greater
        }else{
            Ordering::Equal
        }
    }) {
        Ok(_) => true,
        _ => false
    }
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
    
    
    println!("xxxxxxxxxxx:{}", c.is_cased()); 
    println!("xxxxxxxxxxx:{}", c1.is_cased());
    println!("xxxxxxxxxxx:{}", c2.is_cased()); 
    println!("xxxxxxxxxxx:{}", c3.is_cased());
    println!("xxxxxxxxxxx:{}", c4.is_cased());
    println!("xxxxxxxxxxx:{}", c5.is_cased());

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


