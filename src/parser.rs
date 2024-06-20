use crate::lex::Token;
use std::fmt;
use std::rc::Rc;


use nom::bytes::complete::tag;
use nom::bytes::complete::take_until;
use nom::IResult;
use nom::error::ParseError;
use nom::error::ErrorKind;
use nom::character::complete::alpha1;
use nom::character::complete::anychar;

pub enum TKind{
    I64,
    Variable,
    Label,
    Reg,
    Mem,
}
impl fmt::Display for TKind{
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result{
        match self{
            TKind::I64 => write!(f, "I64"),
            TKind::Variable => write!(f, "Variable"),
            TKind::Label => write!(f, "Label"),
            TKind::Mem => write!(f, "Mem"),
            TKind::Reg => write!(f, "Reg"),
        }
    }
}
pub enum TType<'a>{
    Property(& 'a str),
    Value((TKind, & 'a str)),
    SectionName(& 'a str),
    DefineLabel(& 'a str),
    Instruction(& 'a str),
}
impl<'a> fmt::Display for TType<'a>{
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result{
        match self{
            TType::Property(s) => write!(f, "Property: {}", s),
            TType::Value((t, s)) => write!(f, "Value: {}({})",s, t ),
            TType::SectionName(s) => write!(f, "Section: {}", s),
            TType::Instruction(s) => write!(f, "Instruction: {}", s),
            TType::DefineLabel(s) => write!(f, "DefineLabel: {}", s),
        }
    }
}
pub enum Global<'a>{
    Valid(Rc<Label<'a>>),
    Invalid(& 'a str),
}

impl<'a> fmt::Display for Global<'a>{
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result{
        match self{
            Global::Valid(label)=> write!(f, "Valid Global: {}", label),
            Global::Invalid(name) => write!(f, "Invalid Global: {}", name),
        }
    }
}
pub struct Label<'a>{
    m_name: &'a str,
    m_pos: usize,
}
impl<'a> Label<'a>{
    pub fn new(name: & 'a str) -> Self{
        Self{m_name: name, m_pos: 0}
    }
}
impl<'a> fmt::Display for Label<'a>{
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result{
        write!(f, "Label: {}, pos: {}", self.m_name, self.m_pos)
    }
}
pub struct Parser<'a>{
    m_str: &'a str,
    m_tokens: & 'a Vec<Token<'a>>,
    m_size: usize,
    m_idx: usize,
    m_now_section: &'a str,
}
fn get_reg_r8_r16(input: &str) -> IResult<&str, char>{
    anychar(input)
}
impl<'a> Parser<'a>{
    pub fn new(_str: &'a str, tokens: & 'a Vec<Token>) -> Self{
        Self{m_str: _str, m_tokens: tokens, m_size: tokens.len(), m_idx: 0, m_now_section: ""}
    }
    fn reg_or_mem_imm(&mut self) -> TType<'a>{
        let f = self.consume();
        match f{
            &Token::Figure(n)=>{
                TType::Value((TKind::I64, n))
            },
            &Token::Reserved(n) =>{
                let pe = ParserError::new(self.m_str);
                pe.panic_from_word(n, "expect Reg or Mem");
                panic!();
            },
            &Token::Variable(n) => {
                let pe = ParserError::new(self.m_str);
                let c = get_str_first(n);
                if c == b'r' || c == b'R' {
                }
                return TType::Value((TKind::Reg, n))
            },
        }
    }
    pub fn parse(& mut self) ->
    Result<(Vec<TType>, Vec<Rc<Label<'a>>>, Vec<Global<'a>>), (&'a Token<'a>, &str)>{
        let mut ttypes = Vec::<TType>::new();
        let mut tlabel = Vec::<Rc<Label>>::new();
        let mut tglobal = Vec::<Global>::new();
        let pe = ParserError::new(self.m_str);
        loop {
            if self.has_value() == false {
                break;
            }
            let token = self.consume();
            let &Token::Variable(var) = token else{
                pe.panic_from_word(token.get(), "Syntax Error");
                return Err((token, "Syntax Error"));
            };
            if var == "bits"{
                let token = self.consume();
            }else if var == "section"{
                let var = self.expect(Token::Variable(""));
                ttypes.push(TType::SectionName(var));
                self.m_now_section = var;

            }else if var == "default" {
                let var = self.expect(Token::Variable(""));
            } else if var == "global" {
                let var = self.expect(Token::Variable(""));
                tglobal.push(Global::Invalid(var));
            } else{
                if self.m_now_section == "" {
                    return Err((token, "Define Section"));
                }
                if var == "mov" {
                    ttypes.push(TType::Instruction(var));
                    let var = self.expect(Token::Variable("")); // reg or mem
                    let var = self.expect(Token::Variable("")); // comma
                    let var = self.consume(); // imm or reg or mem
                }else if var == "add" {
                    ttypes.push(TType::Instruction(var));
                    let var = self.expect(Token::Variable("")); // reg or mem
                    let var = self.expect(Token::Variable("")); // comma
                    let var = self.consume(); // imm or reg or mem
                }else if var == "sub" {
                    ttypes.push(TType::Instruction(var));
                    let var = self.expect(Token::Variable("")); // reg or mem
                    let var = self.expect(Token::Variable("")); // comma
                    let var = self.consume(); // imm or reg or mem
                }else if var == "ret"{
                    ttypes.push(TType::Instruction(var));
                }else {// define label
                    self.expect(Token::Reserved(":"));
                    let label = Rc::new(Label::new(var));
                    tlabel.push(label.clone());
                    ttypes.push(TType::DefineLabel(var));
                    // global label の確認
                    for i in 0..tglobal.len(){
                        let &Global::Invalid(n) = &tglobal[i] else{
                            break;
                        };
                        if n == var {
                            tglobal.swap_remove(i);
                            tglobal.push(Global::Valid(label.clone()));
                        }
                    }
                }
            }
        }
        Ok((ttypes, tlabel, tglobal))
    }
    fn expect(&mut self, t: Token) -> &'a str{
        let token = self.consume();
        if std::mem::discriminant(&t) != std::mem::discriminant(token) {
            let pe = ParserError::new(self.m_str);
            let t_val = token.get();
            pe.panic_from_word(t_val, "Error: require: Syntax Error");
        }
        
        match token{
            Token::Figure(n) => n,
            Token::Variable(n) => n,
            Token::Reserved(n) => n,
        }
    }
    
    fn has_value(& self) -> bool{
        if self.m_idx < self.m_size {true } else {false}
    }
    fn peak(&mut self) -> &'a Token<'a>{
        if self.m_idx >= self.m_size {
            let pe = ParserError::new(self.m_str);
            pe.panic_from_pos(self.m_str.len(), "Error: Last");
            panic!("");
        }else{
            &self.m_tokens[self.m_idx]
        }
    }
    fn consume(&mut self) -> &'a Token<'a >{
        if self.m_idx >= self.m_size {
            let pe = ParserError::new(self.m_str);
            pe.panic_from_pos(self.m_str.len(), "Error: Last");
            panic!("");
        }else{
            let ret = &self.m_tokens[self.m_idx];
            self.m_idx += 1;
            return ret;
        }
    }
}
pub struct ParserError<'a >{
    m_str: & 'a str,
}

impl<'a> ParserError<'a>{
    pub fn new(_str: &'a str) -> Self{
        Self{m_str: _str}
    }
    pub fn panic_from_pos(&self, first : usize, message: &str){
        let mut lines = self.m_str.lines();
        let mut loopcnt: usize = 0;
        let mut linecnt = 0;
        let mut rawcnt = 0;
        let mut error_line : &str = "";
        while let Some(line) = lines.next(){
            loopcnt += 1;
            let linefirst = wrapper_pos(self.m_str, line) as usize;

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
        panic!();
    }
    pub fn panic_from_word(&self, word: & 'a str, message: & str){
        self.panic_from_pos(wrapper_pos(self.m_str, word) as usize, message)
    }
}
fn wrapper_pos(origin: &str, branch: &str) -> isize{
    unsafe{branch.as_ptr().offset_from(origin.as_ptr())}
}
fn get_str_first( input: & str) -> u8{
    unsafe{*input.as_ptr()}
}