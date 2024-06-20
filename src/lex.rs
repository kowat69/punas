use std::fmt;

use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::IResult;
use nom::multi::many0_count;
use nom::character::complete::multispace0;
use nom::character::complete::digit1;
use nom::character::complete::one_of;
use nom::character::complete::alpha1;
use nom::character::complete::alphanumeric0;
use nom::character::is_alphabetic;
use nom::character::is_digit;
use nom::character::is_alphanumeric;
use nom::branch::alt;


pub enum Token<'a>{
    Figure(& 'a str),
    Variable(& 'a str),
    Reserved(& 'a str),
}
impl fmt::Display for Token<'_>{
    fn fmt(&self, f: &mut fmt::Formatter)->fmt::Result{
        match self{
            Token::Figure(n) => write!(f,"{}", n),
            Token::Variable(n) => write!(f, "{}", n),
            Token::Reserved(n) => write!(f, "{}", n),
        }
    }
}
impl<'a> Token<'a>{
    pub fn get(&self) -> & 'a str{
        match self{
            Token::Figure(n) =>n,
            Token::Variable(n) =>n,
            Token::Reserved(n) =>n,
        }
    }
}

pub struct Lexer<'a>{
    m_str : &'a str,
}

impl<'a> Lexer<'a>{
    pub fn new(_str:&'a str) -> Self{
        Self{m_str: _str}
    }
    fn ignore_space(&self, input: & 'a str)-> IResult<&str, &str>{
        let (input, _) =  multispace0(input)?;
        Ok((input, ""))
    }
    fn ignore_comment(&self, input: & 'a str) -> IResult<&str, &str>{
        let (input, first) = alt((tag("//"), tag("/*")))(input)?;
        // multiple comments
        if first == "/*"{
            let (input, _) = take_until("*/")(input)?;
            let (input, _) = tag("*/")(input)?;
            Ok((input, ""))
        // single comment
        }else{
            let (input, _) = take_until("\n")(input)?;
            let (input, _) = tag("\n")(input)?;
            Ok((input, ""))
        }
    }
    fn get_figure(&self, input: & 'a str)->IResult<&str, Token>{
        let (input, figure) = digit1(input)?;
        Ok((input, Token::Figure(figure)))
    }
    // section global bits
    fn get_word(&self, input: & 'a str)->IResult<&str, Token> {
        let first = input;
        let mut input = input;
        let c = get_str_first(input);
        if get_str_first(input) == b'.' {
            let (_input, _) = nom::character::complete::char('.')(input)?;
            input = _input;
        }
        let c = get_str_first(input);
        if is_alphabetic(c) || c == b'_'{}
        else {
            let error : IResult<&str, Token> =
            Err(nom::Err::Error(nom::error::Error::new(first, nom::error::ErrorKind::AlphaNumeric)));
            return error;
        }
        loop{
            // get under vars or alphanum
            let (_input, word) = self.get_word_backward(input)?;
            input = _input;
            if word == ""{break}
        }
        //wordの計算
        let word = self.get_str_back(first, input);
        Ok((input, Token::Variable(word)))
    }
    
    fn get_others(&self, input: & 'a str) -> IResult<&str, Token>{
        let token : Token;
        let first = input;
        let result =
            one_of::<_, _, (&str, nom::error::ErrorKind)>(",:()[]")(input);
        if let Ok((input, c)) = result{
            let c = self.get_str_back(first, input);
            match c{
                "," => token = Token::Reserved(c),
                ":" => token = Token::Reserved(c),
                "(" => token = Token::Reserved(c),
                ")" => token = Token::Reserved(c),
                "[" => token = Token::Reserved(c),
                "]" => token = Token::Reserved(c),
                _ => panic!("Can't use \'{}\'", c),
            }
            return Ok((input, token));
        }
        let error : IResult<&str, Token> =
        Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::AlphaNumeric)));
        return error;
    }
    pub fn lex(&self)-> Result<Vec<Token>, (usize, String)>{
        let mut tokens: Vec<Token> = Vec::new();
        let input_len = self.m_str.len();
        let mut m_str = self.m_str;
        loop{
            if m_str == "" {break;}
            if let Ok((s, _)) = self.ignore_space(m_str){
                m_str = s;
            }
            if let Ok((s, _)) = self.ignore_comment(m_str){
                m_str = s;
            }
            let token: Token;
            if let Ok((s, t)) = self.get_figure(m_str) {
                println!("{}", s);
                m_str = s;
                token = t;
            }else if let Ok((s, t)) = self.get_word(m_str){
                m_str = s;
                token = t;
            }else if let Ok((s, t)) = self.get_others(m_str){
                m_str = s;
                token = t;
            }else {
                return Err((input_len - m_str.len(), String::from("Undefined lexical")));
            }
            tokens.push(token);
        }
        Ok(tokens)
    }
//TODO############################################
    fn get_word_backward(&self, input: & 'a str)->IResult<&str, &str>{
        let ch = get_str_first(input);
        if is_alphanumeric(ch){
            let (input, word) = alphanumeric0(input)?;
            Ok((input, word))
        }else if ch == b'_'{
            let (input, _) = many0_count(tag("_"))(input)?;
            // このあと計算イラン get_wordでする
            Ok((input, "_"))
        }else {
            Ok((input, ""))
        }
    }
    fn get_str_back(&self, p: & 'a str, pe: &'a str)-> &str{
        let word = unsafe{std::slice::from_raw_parts(
            p.as_ptr(),
            pe.as_ptr().offset_from(p.as_ptr()) as usize
        )};
        std::str::from_utf8(word).unwrap()
    }

}
pub struct LexerError<'a >{
    m_str: & 'a str,
}
impl<'a> LexerError<'a>{
    pub fn new(lexer:&Lexer<'a>) -> Self{
        Self{m_str: lexer.m_str}
    }
    pub fn panic(&self, first : usize, message: &str){
        let mut lines = self.m_str.lines();
        let mut loopcnt: usize = 0;
        let mut linecnt = 0;
        let mut rawcnt = 0;
        let mut error_line : &str = "";
        while let Some(line) = lines.next(){
            loopcnt += 1;
            let linefirst = wrapper_pos(self.m_str, line) as usize;

            println!("{}", linefirst);
            let lineend = linefirst + line.len();
            if linefirst <= first && first < lineend {
                linecnt = loopcnt;
                // 一応一行目芋締めから始まるため
                rawcnt = first - linefirst + 1;
                error_line = line;
                break;
            }
        }
        let mut spaces = String::from("");
        for _ in 1..rawcnt{
            spaces += " ";
        }
        println!("######ERROR");
        println!("{}, {}", linecnt, rawcnt);
        println!("{}", error_line);
        println!("{}^", spaces);
        println!("{}", message);
    }
}
fn wrapper_pos(origin: &str, branch: &str) -> isize{
    unsafe{branch.as_ptr().offset_from(origin.as_ptr())}
}
fn get_str_first( input: & str) -> u8{
    unsafe{*input.as_ptr()}
}
pub fn nom_test(input: &str)-> IResult<&str, Token>{
    let (input, figure) = digit1(input)?;
    Ok((input, Token::Figure(figure)))
}
