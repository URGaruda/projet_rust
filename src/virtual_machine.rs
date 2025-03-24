#![allow(warnings)]
use std::env;
use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use std::convert::TryInto;
use std::sync::LazyLock;
use std::ops::{Add, Sub, Mul, Div, Rem};
use std::collections::{hash_map, HashMap};
pub const KB :usize=1024;
pub const MB :usize=1024*KB;
pub const GB :usize=1024*MB;


pub static mut STACK: Registre = Registre{ liste : Vec::new()};
pub static mut INSTRUCTION: Vec<(u32,i32,i32,i32)> = Vec::new();
pub static mut CONSTANTES: Const_list = Const_list {liste : Vec::new()};
pub static mut GLOBAL_Key: Vec<String> =  Vec::new();
pub static mut GLOBAL_Value:Vec<TypeLua> = Vec::new();
pub static mut FUNC_BODY: Vec<(u32,u32)> = Vec::new(); //pour le moment on suppose qu'il y a pas de fonction inbriquer 
pub static mut FB_POINTER:i32 =0;
pub static mut CONST_POINTER:i32 =0;
pub static mut PC :i32 = 0;

pub const OPCODE_NAMES: [&str; 38] = [
        "MOVE", "LOADK", "LOADBOOL", "LOADNIL", "GETUPVAL", "GETGLOBAL",
        "GETTABLE", "SETGLOBAL", "SETUPVAL", "SETTABLE", "NEWTABLE", "SELF",
        "ADD", "SUB", "MUL", "DIV", "MOD", "POW", "UNM", "NOT", "LEN",
        "CONCAT", "JMP", "EQ", "LT", "LE", "TEST", "TESTSET", "CALL", "TAILCALL",
        "RETURN", "FORLOOP", "FORPREP", "TFORLOOP", "SETLIST", "CLOSE", "CLOSURE",
        "VARARG"
];
#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub prototype: String,
    pub upvalues: Vec<TypeLua>,  
}

#[derive(Debug, PartialEq)]
pub enum type_inst {
    IABC,
    IABx,
    IAsBx,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeLua {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String), 
    Primitive(glb_func),
    Closure(Closure),
}

impl PartialOrd for TypeLua {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (TypeLua::Number(a), TypeLua::Number(b)) => a.partial_cmp(b),
            (TypeLua::String(a), TypeLua::String(b)) => a.partial_cmp(b),
            _ => None, // Return None for unsupported comparisons
        }
    }
}



impl Add for TypeLua { // Merci copilot (sinon j'allais juste créer une fonction add(typeLua,typeLua))
    type Output = TypeLua;

    fn add(self, other: TypeLua) -> TypeLua {
        match (self, other) {
            (TypeLua::Number(a), TypeLua::Number(b)) => TypeLua::Number(a + b),
            _ => TypeLua::Nil, // Return Nil for unsupported operations
        }
    }
}
impl Sub for TypeLua {
    type Output = TypeLua;

    fn sub(self, other: TypeLua) -> TypeLua {
        match (self, other) {
            (TypeLua::Number(a), TypeLua::Number(b)) => TypeLua::Number(a - b),
            _ => TypeLua::Nil, 
        }
    }
}
impl Mul for TypeLua {
    type Output = TypeLua;

    fn mul(self, other: TypeLua) -> TypeLua {
        match (self, other) {
            (TypeLua::Number(a), TypeLua::Number(b)) => TypeLua::Number(a * b),
            _ => TypeLua::Nil, 
        }
    }
}
impl Div for TypeLua {
    type Output = TypeLua;

    fn div(self, other: TypeLua) -> TypeLua {
        match (self, other) {
            (TypeLua::Number(a), TypeLua::Number(b)) => TypeLua::Number(a / b),
            _ => TypeLua::Nil, 
        }
    }
}
impl Rem for TypeLua {
    type Output = TypeLua;

    fn rem(self, other: TypeLua) -> TypeLua {
        match (self, other) {
            (TypeLua::Number(a), TypeLua::Number(b)) => TypeLua::Number(a - b),
            _ => TypeLua::Nil, 
        }
    }
}
impl TypeLua {
    pub fn pow(self,other:TypeLua)-> TypeLua {
        match (self, other) {
            (TypeLua::Number(a), TypeLua::Number(b)) => { 
                TypeLua::Number(a.powf(b))
            },
            _ => TypeLua::Nil, 
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum glb_func {
    print,
    nil,
}

#[derive(Debug, Clone)]
pub struct Registre {
    pub liste: Vec<TypeLua>,
}
impl Registre {
    fn get(&self,index :usize) -> TypeLua{
        match self.liste.get(index) {
            Some(value) => value.clone(),
            None => TypeLua::Nil,
            
        }
    }
}
#[derive(Debug, Clone)]
pub struct const_type { // représente les constantes qu'on pourras lire du parsing 
    pub types :i32, // 0 si le type représente un booléen, 1 un entier, 2 un string
    pub booléen:u8, 
    pub entier :f64,
    pub chaîne : String,
}

#[derive(Debug,Clone)]
pub struct Const_list {
    pub liste: Vec<const_type>,
} 
impl Const_list {
    pub fn get(&self,index :usize) -> const_type{
        match self.liste.get(index) {
            Some(value) => const_type { types: (value.types), booléen: (value.booléen), entier: (value.entier), chaîne: (value.chaîne.clone()) },
            None => const_type { types: (-1), booléen: (0), entier: (0.0), chaîne: String::new() },
            
        }
    }
}
pub fn init_stack(taille : usize){
    unsafe {
    let mut i = 0;
    while i < taille{
        STACK.liste.push(TypeLua::Nil);
        i=i+1;
    }
    }
}
pub fn init_Global(){
    unsafe {
        GLOBAL_Key.push("print".to_string());
        GLOBAL_Value.push(TypeLua::Primitive(glb_func::print));
    }
}
pub fn str_to_glb(var : String) -> glb_func {
    if var.eq("print") {glb_func::print}else{glb_func::nil}
}
pub fn const_to_luaType(var : const_type,isPrimitive : bool) -> TypeLua {
    match var.types {
        0 => TypeLua::Boolean(if var.booléen==0{true}else{false}),
        1 => TypeLua::Number((var.entier)),
        2 => if isPrimitive {TypeLua::Primitive((str_to_glb(var.chaîne)))}else{TypeLua::String(var.chaîne)}
        _ => TypeLua::Nil,
        
    }
}
pub fn primitive_print(var: &TypeLua) {
    match var {
        TypeLua::Nil => print!("Nil"),
        TypeLua::Boolean(val) => print!("{}", val),
        TypeLua::Number(val) => print!("{}", val),
        TypeLua::String(val) => print!("{}", val),
        TypeLua::Primitive(_) => print!("<Primitive Function>"),
        TypeLua::Closure(_) => print!("<Closure Function>"),
    }
}
pub fn simule_hash(var :String)->i32{
    unsafe{
    let mut i=0;
    for elm in &GLOBAL_Key {
        if *elm==var{
            return i;
        }
        i=i+1;
    }
    i
    }
}
pub fn vm() -> Vec<TypeLua> { // fonction qui va agir de VM pour le bytecode lua 
    unsafe {
    while(PC<INSTRUCTION.len() as i32){
        match INSTRUCTION.get(PC as usize) {
            Some(&(opcode,a,b,c)) =>{
                //println!("PC = {} , \n  Opcode = {} ,\nConstant list = {:?} .\n",PC,opcode,CONSTANTES);
                //println!("PC = {} , Opcode = {} ,a = {}, b = {} , c = {} ",PC,opcode,a,b,c);
                match opcode {
                    0 => {
                        STACK.liste[a as usize] = STACK.liste[b as usize].clone();
                        STACK.liste[b as usize] = TypeLua::Nil ;
                        PC=PC+1;
                    }
                    1 => {
                        STACK.liste[a as usize] = const_to_luaType(CONSTANTES.get(b as usize),false);
                        //println!(" loadk STACK = {:?} ",STACK.liste[0..10].to_vec());
                        CONST_POINTER=CONST_POINTER+1;
                        PC=PC+1;
                    }
                    2 => {
                        STACK.liste[a as usize] = TypeLua::Boolean(b != 0);
                            if c != 0 {
                                PC=PC+1; 
                            }
                            PC=PC+1;
                    }
                    3 => {
                        for i in a..b {
                            STACK.liste[i as usize] = TypeLua::Nil;
                        }
                        PC=PC+1;
                    }
                    5 => {
                        //println!(" getglobal CONST = {:?} ",CONSTANTES.liste[0..CONSTANTES.liste.len()].to_vec());
                        let indice = simule_hash(CONSTANTES.get(b as usize).chaîne);
                        if indice == GLOBAL_Key.len() as i32 {
                            GLOBAL_Key.push(CONSTANTES.get(b as usize).chaîne);
                            GLOBAL_Value.push(TypeLua::Closure(Closure { prototype: CONSTANTES.get(b as usize).chaîne, upvalues : {
                                let mut ls:Vec<TypeLua> = Vec::new(); 
                                let mut i = a ;
                                while true {
                                    match STACK.liste[i as usize].clone() {
                                        TypeLua::Nil => break,
                                        _=> ls.push(STACK.liste[i as usize].clone()),
                                    }
                                    i=i+1;
                                }
                                ls
                            } }));
                            STACK.liste[a as usize] = GLOBAL_Value[indice as usize].clone();
                        }else{
                            STACK.liste[a as usize] = GLOBAL_Value[indice as usize].clone();
                        }
                        //println!(" getglobal STACK = {:?} ",STACK.liste[0..10].to_vec());
                        CONST_POINTER=CONST_POINTER+1;
                        PC=PC+1;
                    }
                    7 => {
                        //println!("constante = {:?} ",CONSTANTES.get(b as usize).chaîne);
                        GLOBAL_Key.push(CONSTANTES.get(b as usize).chaîne.clone());
                        GLOBAL_Value.push(STACK.liste[a as usize].clone());
                        PC=PC+1;
                    }
                    12 => {
                        if b < 256 {
                            if c < 256 {
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone() + STACK.liste[c as usize].clone() ;
                            }else{
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone() + (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }else{
                            if c < 256 {
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) + STACK.liste[c as usize].clone() ;
                            }else{
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) + (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }
                        PC=PC+1;
                    }
                    13 => {
                        if b < 256 {
                            if c < 256 {
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone() - STACK.liste[c as usize].clone() ;
                            }else{
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone() - (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }else{
                            if c < 256 {
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) - STACK.liste[c as usize].clone() ;
                            }else{
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) - (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }
                        PC=PC+1;
                    }
                    14 => {
                        if b < 256 {
                            if c < 256 {
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone() * STACK.liste[c as usize].clone() ;
                            }else{
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone() * (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }else{
                            if c < 256 {
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) * STACK.liste[c as usize].clone() ;
                            }else{
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) * (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }
                        PC=PC+1;
                    }
                    15 => {
                        if b < 256 {
                            if c < 256 {
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone() / STACK.liste[c as usize].clone() ;
                            }else{
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone() / (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }else{
                            if c < 256 {
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) / STACK.liste[c as usize].clone() ;
                            }else{
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) / (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }STACK.liste[a as usize] = STACK.liste[b as usize].clone() / STACK.liste[c as usize].clone() ;
                        PC=PC+1;
                    }
                    16 => {
                        if b < 256 {
                            if c < 256 {
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone() % STACK.liste[c as usize].clone() ;
                            }else{
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone() % (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }else{
                            if c < 256 {
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) % STACK.liste[c as usize].clone() ;
                            }else{
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) % (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }
                        PC=PC+1;
                    }
                    17 => {
                        if b < 256 {
                            if c < 256 {
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone().pow(STACK.liste[c as usize].clone()) ;
                            }else{
                                STACK.liste[a as usize] = STACK.liste[b as usize].clone().pow(const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }else{
                            if c < 256 {
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false).pow(STACK.liste[c as usize].clone()) ;
                            }else{
                                STACK.liste[a as usize] = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false).pow(const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }

                        PC=PC+1;
                    }
                    18 => {
                        STACK.liste[a as usize] = { match STACK.liste[b as usize] {
                            TypeLua::Number(val) => TypeLua::Number(-val),
                            _ => TypeLua::Nil,
                        }};
                        PC=PC+1;
                    }
                    19 => {
                        STACK.liste[a as usize] = { match STACK.liste[b as usize] {
                            TypeLua::Boolean(val) => TypeLua::Boolean(!val),
                            _ => TypeLua::Nil,
                        }};
                        PC=PC+1;
                    }
                    21 => {
                        let mut chaine: String =String::new();
                        //println!(" concat STACK = {:?} ",STACK.liste[0..10].to_vec());
                        for i in b..=c {
                            match &STACK.liste[i as usize] {
                                TypeLua::String(val) => {
                                    chaine.push_str(&val);
                                }
                                _=>{}
                            }
                        }
                        STACK.liste[a as usize] = TypeLua::String((chaine));
                        PC=PC+1;

                    }
                    22 => {
                        PC=PC+1+b as i32;
                    }
                    23 => {
                        let verif:bool;
                        if b < 256 {
                            if c < 256 {
                                verif = STACK.liste[b as usize].clone() == (STACK.liste[c as usize].clone()) ;
                            }else{
                                verif = STACK.liste[b as usize].clone() == (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }else{
                            if c < 256 {
                                verif = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) == (STACK.liste[c as usize].clone()) ;
                            }else{
                                verif = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) == (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }
                        if verif {
                            if a==1 {
                                PC=PC+1;
                            }
                        }else{
                            if a==0 {
                                PC=PC+1;
                            }
                        }
                        
                        PC=PC+1;
                    }
                    24 => {
                        let verif:bool;
                        if b < 256 {
                            if c < 256 {
                                verif = STACK.liste[b as usize].clone() < (STACK.liste[c as usize].clone()) ;
                            }else{
                                verif = STACK.liste[b as usize].clone() < (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }else{
                            if c < 256 {
                                verif = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) < (STACK.liste[c as usize].clone()) ;
                            }else{
                                verif = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) < (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }
                        if verif {
                            if a==1 {
                                PC=PC+1;
                            }
                        }else{
                            if a==0 {
                                PC=PC+1;
                            }
                        }
                        PC=PC+1;
                    }
                    25 => {
                        let verif:bool;
                        if b < 256 {
                            if c < 256 {
                                verif = STACK.liste[b as usize].clone() <= (STACK.liste[c as usize].clone()) ;
                            }else{
                                verif = STACK.liste[b as usize].clone() <= (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }else{
                            if c < 256 {
                                verif = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) <= (STACK.liste[c as usize].clone()) ;
                            }else{
                                verif = const_to_luaType(CONSTANTES.liste[b as usize %256].clone(),false) <= (const_to_luaType(CONSTANTES.liste[c as usize %256].clone(),false)) ;
                            }
                        }
                        if verif {
                            if a==1 {
                                PC=PC+1;
                            }
                        }else{
                            if a==0 {
                                PC=PC+1;
                            }
                        }
                        PC=PC+1;
                    }
                    28 => { //R(A), ... ,R(A+C-2) := R(A)(R(A+1), ... ,R(A+B-1))
                        let nb_arg: i32 = b as i32 - 1;
                        let nb_return: i32 = c as i32 -1;
                        let get_fct = STACK.liste[a as usize].clone();
                        //println!(" get_fct = {:?} avec a = {} ",get_fct,a);
                        match get_fct {
                            TypeLua::Primitive(primitive) => {
                                match primitive {
                                    glb_func::print => {
                                        //println!("entre dans le print avec nb_arg = {} \n et STACK = {:?} ",nb_arg,STACK.liste[0..10].to_vec());
                                        let mut j = 0;
                                        if nb_arg > 0 {
                                            while j < nb_arg{
                                                primitive_print(&STACK.liste[(a as i32 +(j+1)) as usize]); // fct qui doit prendre le paramètre et la primitive pour l'executer et avoir sa valeur de retour (s'il y a )
                                                j=j+1;
                                            }
                                        }else {
                                            primitive_print(&STACK.liste[(a as i32 +1) as usize]);
                                        }
                                    }
                                    _ => { println!("pas de primitive prévu pour cet instruction {:?}",get_fct )}
                                }
                                print!("\n");
                            }
                            TypeLua::Closure(closure) => {
                                //println!("Closure : {:?} ",closure);
                                let indice = FUNC_BODY[FB_POINTER as usize];
                                let tmp_pc = PC;
                                PC=indice.0 as i32;
                                let tmp_const = CONSTANTES.clone();
                                CONSTANTES.liste = CONSTANTES.liste[CONST_POINTER as usize..CONSTANTES.liste.len()].to_vec();
                                let tmp_inst = STACK.clone();
                                STACK=Registre{ liste : Vec::new()};
                                init_stack(KB);
                                let mut k = 0;
                                for i in 1..(nb_arg+1) {
                                    STACK.liste[k as usize] = tmp_inst.liste[(a as i32 +i) as usize].clone();
                                    k=k+1;
                                }
                                //println!(" stack : {:?} ",STACK.liste[0..10].to_vec());
                                let res = vm();
                                STACK=tmp_inst.clone();
                                CONSTANTES=tmp_const.clone();
                                PC=tmp_pc;
                                let mut j=a;
                                if nb_return>0 {
                                    for i in 0..nb_arg {
                                    STACK.liste[j as usize] = res[i as usize].clone();
                                    j=j+1;
                                    }
                                }else if nb_return<0 {
                                    for val in res {
                                        STACK.liste[j as usize] = val;
                                        j=j+1;
                                    }
                                }
                                FB_POINTER=FB_POINTER+1;
                            }
                            _=>{println!(" Tu n'es pas une primitive/fonction ")}
                        }
                        PC=PC+1;
                    }
                    30 => {
                        let nb_arg = b-1;
                        let mut return_value: Vec<TypeLua> = Vec::new();
                        let mut j = 0;
                        while j < nb_arg{
                            return_value.push(STACK.liste[(a+j) as usize].clone());
                            j=j+1;
                        }
                        return return_value;
                    }
                    31 => {
                        STACK.liste[a as usize] = STACK.liste[a as usize].clone() + STACK.liste[ (a+2) as usize].clone(); 
                        if STACK.liste[a as usize].clone() <= STACK.liste[(a+1) as usize].clone() {
                            PC=PC+(b as i32)+1;
                            STACK.liste[(a+3) as usize] = STACK.liste[a as usize].clone();
                        }
                    }
                    36 =>{
                        STACK.liste[a as usize] = TypeLua::Closure(Closure{
                            prototype :{
                                match CONSTANTES.liste[b as usize].clone().types {
                                    2 => CONSTANTES.liste[b as usize].clone().chaîne,
                                    _ => "Erreur".to_string(),
                                }
                            },
                            upvalues : {
                                let mut ls:Vec<TypeLua> = Vec::new(); 
                                let mut i = a ;
                                while true {
                                    match STACK.liste[i as usize].clone() {
                                        TypeLua::Nil => break,
                                        _=> ls.push(STACK.liste[i as usize].clone()),
                                    }
                                    i=i+1;
                                }
                                ls
                            }
                        });
                        PC=PC+1;
                    }
                    _ => {
                        println!("Unhandled opcode: {}", opcode);
                        PC=PC+1;
                    }

                }
            }
            None => {
                println!("Problème dans la VM");
                break;
            }
        }
    }
    return vec![TypeLua::Nil];
    }
}
pub const TYPE_OPCODE: [type_inst; 38] = [
    type_inst::IABC,   // MOVE 0
    type_inst::IABx,   // LOADK 1
    type_inst::IABC,   // LOADBOOL 2
    type_inst::IABC,   // LOADNIL 3
    type_inst::IABC,   // GETUPVAL 4
    type_inst::IABx,   // GETGLOBAL 5
    type_inst::IABC,   // GETTABLE 6
    type_inst::IABx,   // SETGLOBAL 7
    type_inst::IABC,   // SETUPVAL 8
    type_inst::IABC,   // SETTABLE 9
    type_inst::IABC,   // NEWTABLE 10
    type_inst::IABC,   // SELF 11
    type_inst::IABC,   // ADD 12
    type_inst::IABC,   // SUB 13
    type_inst::IABC,   // MUL 14
    type_inst::IABC,   // DIV 15
    type_inst::IABC,   // MOD 16
    type_inst::IABC,   // POW 17
    type_inst::IABC,   // UNM 18
    type_inst::IABC,   // NOT 19
    type_inst::IABC,   // LEN 20
    type_inst::IABC,   // CONCAT 21
    type_inst::IAsBx,  // JMP 22
    type_inst::IABC,   // EQ 23
    type_inst::IABC,   // LT 24
    type_inst::IABC,   // LE 25
    type_inst::IABC,   // TEST 26
    type_inst::IABC,   // TESTSET 27
    type_inst::IABC,   // CALL 28
    type_inst::IABC,   // TAILCALL 29
    type_inst::IABC,   // RETURN 30
    type_inst::IAsBx,  // FORLOOP 31
    type_inst::IAsBx,  // FORPREP 32
    type_inst::IABC,   // TFORLOOP 33
    type_inst::IABC,   // SETLIST 34
    type_inst::IABC,   // CLOSE 35
    type_inst::IABx,   // CLOSURE 36
    type_inst::IABC,   // VARARG 37
];
