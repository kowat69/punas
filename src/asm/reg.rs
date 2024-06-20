
pub const RAX:u8 = 0b0000;
pub const RCX:u8 = 0b0001;
pub const RDX:u8 = 0b0010;
pub const RBX:u8 = 0b0011;
pub const RSP:u8 = 0b0100;
pub const RBP:u8 = 0b0101;
pub const RSI:u8 = 0b0110;
pub const RDI:u8 = 0b0111;

pub const R8  :u8 = 0b1000;
pub const R9  :u8 = 0b1001;
pub const R10 :u8 = 0b1010;
pub const R11 :u8 = 0b1011;
pub const R12 :u8 = 0b1100;
pub const R13 :u8 = 0b1101;
pub const R14 :u8 = 0b1110;
pub const R15 :u8 = 0b1111;

pub const REX_W:u8 = 3;
pub const REX_R:u8 = 2;
pub const REX_X:u8 = 1;
pub const REX_B:u8 = 0;
pub enum Value<'a>{
    Figure(& 'a str),
    Reg(u8, u8),// modr/m
}
pub fn create_modrm(modf: u8, reg: u8, rm: u8) -> u8{
    modf << 6 | reg << 3| rm
}
pub fn create_rex(w: u8, r: u8, x: u8, b: u8) -> u8{
    0x40 | w << 3 | r << 2 | x << 1 | b<< 0
}
pub fn reg<'a>(rm: & 'a str) -> Result<Value<'a>, &str>{
    let _r = rm.to_lowercase();

    match _r.as_str(){
        "rax" => Ok(Value::Reg(RAX, 8)),
        "rcx" => Ok(Value::Reg(RCX, 8)),
        "rdx" => Ok(Value::Reg(RDX, 8)),
        "rbx" => Ok(Value::Reg(RBX, 8)),
        "rsp" => Ok(Value::Reg(RSP, 8)),
        "rbp" => Ok(Value::Reg(RBP, 8)),
        "rsi" => Ok(Value::Reg(RSI, 8)),
        "rdi" => Ok(Value::Reg(RDI, 8)),
        "r8" => Ok(Value::Reg(R8, 8)),
        "r9" => Ok(Value::Reg(R9, 8)),
        "r10" => Ok(Value::Reg(R10, 8)),
        "r11" => Ok(Value::Reg(R11, 8)),
        "r12" => Ok(Value::Reg(R12, 8)),
        "r13" => Ok(Value::Reg(R13, 8)),
        "r14" => Ok(Value::Reg(R14, 8)),
        "r15" => Ok(Value::Reg(R15, 8)),
        _ => {
            return Err(rm);
        },
    }
}
