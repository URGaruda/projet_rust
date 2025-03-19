#![allow(warnings)]
use std::arch::global_asm;
use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use std::convert::TryInto;
use std::ops::Add;
/*
1ère étape : Parsing du header de luac.out 
2ème étape : Parsing des function blocks 
3ème étape : Dump des instructions 
4ème étape : Opérations arithmétiques simples 
5ème étape : Gestion de la primitive print 
6ème étape : Ajout d'autres primitives (Optionnel) 
*/
/*
En gros ce que tu vas devoir faire c'est une pile pour empiler,dépiler les instructions,constantes que tu liras pour faire leurs executions
Regarde le td pour des exemples 
registres sont des places de la pile dans le contexte d'execution d'une fonction réfère toi à tes souvenirs et à la photo
Prochaine étape : créer un interpreteur pour les instructions 
*/



const KB :usize=1024;
const MB :usize=1024*KB;
const GB :usize=1024*MB;


static mut STACK: Registre = Registre{ liste : Vec::new()};
static mut INSTRUCTION: Vec<(u32,u32,u32,u32)> = Vec::new();
static mut CONSTANTES: Const_list = Const_list {liste : Vec::new()};
static mut GLOBAL: Vec<glb_func> =Vec::new();
static mut PC :i32 = 0;

const OPCODE_NAMES: [&str; 38] = [
        "MOVE", "LOADK", "LOADBOOL", "LOADNIL", "GETUPVAL", "GETGLOBAL",
        "GETTABLE", "SETGLOBAL", "SETUPVAL", "SETTABLE", "NEWTABLE", "SELF",
        "ADD", "SUB", "MUL", "DIV", "MOD", "POW", "UNM", "NOT", "LEN",
        "CONCAT", "JMP", "EQ", "LT", "LE", "TEST", "TESTSET", "CALL", "TAILCALL",
        "RETURN", "FORLOOP", "FORPREP", "TFORLOOP", "SETLIST", "CLOSE", "CLOSURE",
        "VARARG"
];
#[derive(Debug, PartialEq)]
enum type_inst {
    IABC,
    IABx,
    IAsBx,
}

#[derive(Clone)]
enum TypeLua {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String), 
    Primitive(glb_func),
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

#[derive(Clone, Copy)]
enum glb_func {
    print,
    nil,
}

struct Registre {
    liste: Vec<TypeLua>,
}

impl Registre {
    fn get(&self,index :usize) -> TypeLua{
        match self.liste.get(index) {
            Some(value) => value.clone(),
            None => TypeLua::Nil,
            
        }
    }
}

struct const_type { // représente les constantes qu'on pourras lire du parsing 
    types :i32, // 0 si le type représente un booléen, 1 un entier, 2 un string
    booléen:u8, 
    entier :f64,
    chaîne : String,
}

struct Const_list {
    liste: Vec<const_type>,
}
 
impl Const_list {
    fn get(&self,index :usize) -> const_type{
        match self.liste.get(index) {
            Some(value) => const_type { types: (value.types), booléen: (value.booléen), entier: (value.entier), chaîne: (value.chaîne.clone()) },
            None => const_type { types: (-1), booléen: (0), entier: (0.0), chaîne: String::new() },
            
        }
    }
}
fn str_to_glb(var : String) -> glb_func {
    if var.eq("print") {glb_func::print}else{glb_func::nil}
}

fn const_to_luaType(var : const_type,isPrimitive : bool) -> TypeLua {
    match var.types {
        0 => TypeLua::Boolean(if var.booléen==0{true}else{false}),
        1 => TypeLua::Number((var.entier)),
        2 => if isPrimitive {TypeLua::Primitive((str_to_glb(var.chaîne)))}else{TypeLua::String(var.chaîne)}
        _ => TypeLua::Nil,
        
    }
}

fn vm() { // fonction qui va agir de VM pour le bytecode lua 
    unsafe {
    let mut i = 0;
    
    while(i<INSTRUCTION.len()){
        match INSTRUCTION.get(i) {
            Some(&(opcode,a,b,c)) =>{

                match opcode {
                    1 => {
                        STACK.liste[a as usize] = const_to_luaType(CONSTANTES.get(b as usize),false);
                    }
                    5 => {
                        STACK.liste[a as usize] = const_to_luaType(CONSTANTES.get(b as usize),true);
                    }
                    12 => {
                        STACK.liste[a as usize] = STACK.liste[b as usize].clone() + STACK.liste[c as usize].clone() ;
                    }
                    
                    _ => {
                        println!("Unhandled opcode: {}", opcode);
                    }

                }
            }
            None => {
                println!("Problème dans la VM");
            }
        }
    }
    }
}

const TYPE_OPCODE: [type_inst; 38] = [
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


fn get_bits(num: u32, p: u32, s: u32) -> u32 {
    (num >> p) & ((1 << s) - 1)
}
fn bytes_to_u32(bytes: &[u8], endian: i32) -> u32 {
    if endian == 0 {
        ((bytes[0] as u32) << 24) |
        ((bytes[1] as u32) << 16) |
        ((bytes[2] as u32) << 8)  |
        ((bytes[3] as u32))
    } else {
        ((bytes[3] as u32) << 24) |
        ((bytes[2] as u32) << 16) |
        ((bytes[1] as u32) << 8)  |
        ((bytes[0] as u32))
    }
}
fn affiche_op_inst(tab: &[u8], taille_inst: usize, endian: i32) {
    for i in 0..taille_inst {
        let inst = &tab[i * 4..(i + 1) * 4];
        let data = bytes_to_u32(inst, endian);
        let opcode = get_bits(data, 0, 6);
        println!("Instruction {}: Opcode : {} ({})", i, opcode, OPCODE_NAMES[opcode as usize]);
        let tp = &TYPE_OPCODE[opcode as usize];
        let a = get_bits(data, 6, 8);
        let b;
        let c;
        println!(" tp = {:?} ",tp);
        if *tp == type_inst::IABC {
            b=get_bits(data, 23, 9);
            c=get_bits(data, 14, 9);
            println!("a = {} , b = {} , c = {} ",a,b,c);
            unsafe {
                INSTRUCTION.push((opcode, a, b, c));
            }
        }else if *tp == type_inst::IABx {
            b=get_bits(data, 14, 18);
            println!("a = {} , b = {}",a,b);
            unsafe {
                INSTRUCTION.push((opcode, a, b, 0));
            }
        }else{
            b=get_bits(data, 14, 18)-131071;
            println!("a = {} , b = {}",a,b);
            unsafe {
                INSTRUCTION.push((opcode, a, b, 0));
            }
        }
        
       
    }
}
fn u8_to_i32 (val :u8) -> i32 {
    val as i32
}
fn unwrap_to_i32 (val :Option<&u8>,default:i32) -> i32 {
    match val {
        Some(val) => u8_to_i32(*val),
        None => default,
    }
}

fn get_u8(val: Option<&u8>, default: u8) -> Vec<u8> {
    match val {
        Some(val) => vec![*val],
        None => vec![default],
    }
}

fn byte_to_number (val : &[u8]) -> i128 { // faudrait penser à mettre l'option little ou big endian 
    let mut i: usize =0 ;
    let mut res: i128=0;
    while i<val.len() {
        match val.get(i) {
            Some(tmp) => res=res+((*tmp as i128)*(i128::pow(256, i as u32))),
            None => i=i,
        }
        i=i+1;
    }
    res
}

fn byte_to_number_be(val: &[u8]) -> i128 { 
    let mut res: i128 = 0;
    for (i, &tmp) in val.iter().rev().enumerate() {
        res = res + ((tmp as i128) * (i128::pow(256, i as u32)));
    }
    res
}

fn convert_to_chaine(ls : &[u8]) -> Vec<char> {
    let mut res: Vec<char> = Vec::new();
    for &var in ls {
        res.push(char::from(var));
    }
    res
}

fn charVec_to_string(vecteur : Vec<char>) -> String {
    let mut res : String = String::new();
    for var in vecteur{
        if var!='\0'{
            res.push(var);
        }
    }
    res
}
//Constant list
// Number of constants - Integer 
fn parse_const_list(ls : &[u8], begin : usize,size_int : usize,size_t:usize,endian:i32) -> usize {
    let mut taille_ls_const: usize = 0;
    let u8_const_ls: Option<&[u8]> = ls.get(begin..begin+size_int);
    match u8_const_ls {
        //Some(value_line) => taille_ls_const= byte_to_number(value_line) as usize,
        Some(value_line) => if endian == 1 {
            taille_ls_const = u32::from_le_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        }else{
            taille_ls_const = u32::from_be_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        },
        None => println!("no Value"),
    }
    println!("Nombre de constantes du code en liste de byte {:?} ",u8_const_ls);

    println!("Nombre des constantes : {} ",taille_ls_const); 

    affiche_const_list(ls, begin+size_int, taille_ls_const,size_t, endian)

}

fn affiche_const_list(ls : &[u8], begin : usize,taille : usize,size_t:usize,endian:i32) -> usize {// va lire les bytes concernant la
    let mut tmp = begin ;
    let mut i = 0;
    while i < taille {
        let type_const = unwrap_to_i32(ls.get(tmp), -1);
        tmp=tmp+1;
        if type_const == 0 { // il n'y a pas de data on ignore 
            println!("Ignore");
        }
        if type_const == 1 { // Booléan donc 0 ou 1 
            let boolean = unwrap_to_i32(ls.get(tmp), -1);
            /*unsafe {
                CONSTANTES.push(const_type {
                    types: 0, 
                    booléen: boolean as u8,
                    entier: 0.0, 
                    chaîne: String::new(), 
                });
            }*/
            if boolean==0{
                println!("Boolean = True");
            }else if boolean==1 {
                println!("Boolean = False");
            }else{
                println!("Boolean = ???");
            }
            tmp=tmp+1;
        }
        if type_const == 3 { // Lua Number 8 bytes // obtenue par ChatGPT 
            let mut lua_numb: f64=-1.0;
            if let Some(gt_bytes) = ls.get(tmp..tmp+8) {
                if endian==1 {
                    lua_numb = f64::from_le_bytes(gt_bytes.try_into().unwrap_or([0; 8]));
                }else{
                    lua_numb = f64::from_be_bytes(gt_bytes.try_into().unwrap_or([0; 8]));
                }
                tmp = tmp + 8;
            } else {
                println!("Erreur: Impossible de lire 8 octets pour le nombre Lua");
            }
            /*unsafe {
                CONSTANTES.push(const_type {
                    types: 1, 
                    booléen: 0,
                    entier: lua_numb, 
                    chaîne: String::new(), 
                }); 
            }*/
            println!("valeur du nombre lua = {} ", lua_numb);
        }
        if type_const == 4 {
            let size_name = ls.get(tmp..tmp+size_t);
            tmp=tmp+size_t;
            let mut size_t_value = 0;
            match size_name {
                //Some(name_value) => size_t_value = usize::from_le_bytes(name_value.try_into().expect("Erreur de conversion")), // la fonction a été pris par ChatGpt
                Some(name_value) => {if endian == 1 
                                            {size_t_value = byte_to_number(name_value) as usize} 
                                            else {
                                            size_t_value= byte_to_number_be(name_value) as usize}
                                        } ,
                None => println!("No size_t_value"),
            }
            println!("size_t_value : {} ",size_t_value);

            let titre_op = ls.get(tmp..(tmp+size_t_value));
            let mut titre: Vec<char> = Vec::new();
            match titre_op {
                Some(val_titre) => titre = convert_to_chaine(val_titre) ,
                None => println!("No titre"),   
            }
            println!("valeur du titre : {:?} ",titre);
            /*unsafe {
                CONSTANTES.push(const_type {
                    types: 2, 
                    booléen: 0,
                    entier: 0.0, 
                    chaîne: charVec_to_string(titre), 
                });
            }*/
            tmp=tmp+size_t_value;
        }
        i=i+1;
    }
    tmp
}


fn parse_func_block(ls : &[u8], begin : usize,taille : i32,size_int : usize,size_t:usize,size_inst:usize,endian:i32) -> usize {
    if taille <= 0{return begin;}
    let to_name = begin+size_t;
    let size_name = ls.get(begin..to_name);//12+valeur de size_st_op (même -1 pour ignorer le dernier caractère qui vaut 0)
    match size_name {
        Some(size_name) => println!("size__func_name : {:?}",size_name),
        None => println!("No size_func_name"),   
    }
    let mut size_t_value = 0;
    match size_name {
        //Some(name_value) => size_t_value = usize::from_le_bytes(name_value.try_into().expect("Erreur de conversion")), // la fonction a été pris par ChatGpt
        Some(name_value) => size_t_value = byte_to_number(name_value) as usize ,
        None => println!("No size_t_func_value"),
    }
    
    println!("size_t_func_value : {} ",size_t_value);

    let titre_op = ls.get(to_name..(to_name+size_t_value));
    let mut titre: Vec<char> = Vec::new();
    match titre_op {
        Some(val_titre) => titre = convert_to_chaine(val_titre) ,
        None => println!("No titre"),   
    }
    println!("valeur du titre : {:?} ",titre);

    let first_line_int = to_name+size_t_value;
    
    let first_line = ls.get((first_line_int)..(first_line_int+size_int));
    match first_line {
        Some(value_line) => println!("first_line : {:?}",value_line),
        None => println!("No first_line"),   
    }

    let last_line_int= first_line_int+size_int;
    
    let last_line = ls.get((last_line_int)..(last_line_int+size_int));
    match last_line {
        Some(value_line) => println!("last_line : {:?}",value_line),
        None => println!("No last_line"),   
    }
    
    let id_1 = last_line_int+size_int;

    let nb_upval = unwrap_to_i32(ls.get(id_1),-1) as usize;
    println!("nb_upval : {:?}", nb_upval);

    let nb_param = unwrap_to_i32(ls.get(id_1+1),-1) as usize;
    println!("nb_param : {:?}", nb_param);

    let var_flag = unwrap_to_i32(ls.get(id_1+2),-1) as usize;
    println!("var_flag : {:?}", var_flag);

    let max_stack_sz = unwrap_to_i32(ls.get(id_1+3),-1) as usize;
    println!("max_stack_sz : {:?}", max_stack_sz);
    // Instuction list
    // Les instructions font 4 bytes = 32 octets (1 bytes = 8 bits = 1 octets)
    let id_cs = parse_inst_list(ls, id_1+4,size_int,size_inst,endian);
    
    let mut id_func_proc = parse_const_list(ls, id_cs,size_int,size_t,endian);

    // Function protocole 
    //Debut parsing function protocole 
    println!("debut des fonction protocole = {} ",id_func_proc);

    let size_func_proc = ls.get(id_func_proc..id_func_proc+size_int);
    let mut taille_func_proc = -1;
    match size_func_proc {
        //Some(value_line) => taille_inst= byte_to_number(value_line) as usize,
        Some(value_line) => taille_func_proc = i32::from_le_bytes(value_line.try_into().expect("slice with incorrect length")),
        None => println!("no Value"),
    }
    println!("Nombre des inst du code en liste de byte {:?} ",size_func_proc);
    
    println!("Nombre des inst du code en entier : {} ",taille_func_proc);

    id_func_proc=parse_func_block(ls, id_func_proc+size_int, taille_func_proc , size_int, size_t,size_inst, endian);
    // fin parsing function protocole 

    let mut tmp = parse_source_line(ls,id_func_proc , size_int, endian);
    tmp= parse_local_list(ls,tmp , size_int,size_t, endian);
    println!("tmp avant upvalue {} ",tmp);
    tmp=parse_upvalue_list(ls,tmp , size_int,size_t, endian);
    println!("tmp après upvalue {} ",tmp);

    tmp
}

fn parse_source_line(ls : &[u8], begin : usize,size_int : usize,endian:i32) -> usize {
    let mut tmp = begin ;
    let mut i = 0;
    let size_source_line;
    let mut tmp_size_list = ls.get(begin..begin+size_int);
    if endian == 1 {
        match tmp_size_list {
            //Some(value_int) => size_source_line= usize::from_le_bytes(value_int.try_into().expect("slice with incorrect length")),
            Some(value_int) => size_source_line= byte_to_number(value_int) as usize,
            None => size_source_line = 0,
        }
    }else{
        match tmp_size_list {
            //Some(value_int) => size_source_line = usize::from_be_bytes(value_int.try_into().expect("slice with incorrect length")),
            Some(value_int) => size_source_line= byte_to_number_be(value_int) as usize,
            None => size_source_line = 0,
        } 
    }
    tmp=tmp+size_int;
    while i < size_source_line {
        let inst_pos ;
        tmp_size_list = ls.get(tmp..tmp+size_int);
        tmp=tmp+size_int;
        if endian == 1 {
            match tmp_size_list {
                //Some(value_int) => inst_pos= usize::from_le_bytes(value_int.try_into().expect("slice with incorrect length")),
                Some(value_int) => inst_pos= byte_to_number(value_int) as usize,
                None => inst_pos = 0,
            }
        }else{
            match tmp_size_list {
                //Some(value_int) => inst_pos = usize::from_be_bytes(value_int.try_into().expect("slice with incorrect length")),
                Some(value_int) => inst_pos= byte_to_number_be(value_int) as usize,
                None => inst_pos = 0,
            } 
        }
        println!("instruction numéro {} est positionné à {} ",i,inst_pos);
        i=i+1;
    }
    tmp
}

fn parse_local_list(ls : &[u8], begin : usize,size_int : usize,size_t:usize,endian:i32) -> usize {
    let mut tmp = begin ;
    let mut i = 0;
    let size_local_list;
    let mut tmp_size_list = ls.get(begin..begin+size_int);
    if endian == 1 {
        match tmp_size_list {
            //Some(value_int) => size_local_list= usize::from_le_bytes(value_int.try_into().expect("slice with incorrect length")),
            Some(value_int) => size_local_list= byte_to_number(value_int) as usize,
            None => size_local_list = 0,
        }
    }else{
        match tmp_size_list {
            //Some(value_int) => size_local_list = usize::from_be_bytes(value_int.try_into().expect("slice with incorrect length")),
            Some(value_int) => size_local_list= byte_to_number_be(value_int) as usize,
            None => size_local_list = 0,
        } 
    }
    tmp=tmp+size_int;
    while i < size_local_list {
        //string 
        let taille ;
        tmp_size_list = ls.get(tmp..tmp+size_t);
        tmp=tmp+size_t;
        if endian == 1 {
            match tmp_size_list {
                Some(value_int) => taille= usize::from_le_bytes(value_int.try_into().expect("slice with incorrect length")),
                None => taille = 0,
            }
        }else{
            match tmp_size_list {
                Some(value_int) => taille = usize::from_be_bytes(value_int.try_into().expect("slice with incorrect length")),
                None => taille = 0,
            } 
        }
        let titre_op = ls.get(tmp..(tmp+taille));
        tmp=tmp+taille;
        let mut titre: Vec<char> = Vec::new();
        match titre_op {
            Some(val_titre) => titre = convert_to_chaine(val_titre) ,
            None => println!("No titre"),   
        }
        println!("valeur du titre : {:?} ",titre);
        //startpc 
        let inst_pos ;
        tmp_size_list = ls.get(tmp..tmp+size_int);
        tmp=tmp+size_int;
        if endian == 1 {
            match tmp_size_list {
                //Some(value_int) => inst_pos= usize::from_le_bytes(value_int.try_into().expect("slice with incorrect length")),
                Some(value_int) => inst_pos= byte_to_number(value_int) as usize,
                None => inst_pos = 0,
            }
        }else{
            match tmp_size_list {
                //Some(value_int) => inst_pos = usize::from_be_bytes(value_int.try_into().expect("slice with incorrect length")),
                Some(value_int) => inst_pos= byte_to_number_be(value_int) as usize,
                None => inst_pos = 0,
            } 
        }
        println!("le starpc de la variable locale est {} ",inst_pos);
        //endpc 
        let inst_pos ;
        tmp_size_list = ls.get(tmp..tmp+size_int);
        tmp=tmp+size_int;
        if endian == 1 {
            match tmp_size_list {
                //Some(value_int) => inst_pos= usize::from_le_bytes(value_int.try_into().expect("slice with incorrect length")),
                Some(value_int) => inst_pos= byte_to_number(value_int) as usize,
                None => inst_pos = 0,
            }
        }else{
            match tmp_size_list {
                //Some(value_int) => inst_pos = usize::from_be_bytes(value_int.try_into().expect("slice with incorrect length")),
                Some(value_int) => inst_pos= byte_to_number_be(value_int) as usize,
                None => inst_pos = 0,
            } 
        }
        println!("le endpc de la la variable locale est {} ",inst_pos);
        i=i+1;
    }
    tmp
}

fn parse_upvalue_list(ls : &[u8], begin : usize,size_int : usize,size_t:usize,endian:i32) -> usize {
    let mut tmp = begin ;
    let mut i = 0;
    let size_local_list;
    let mut tmp_size_list = ls.get(begin..begin+size_int);
    if endian == 1 {
        match tmp_size_list {
            //Some(value_int) => size_local_list= usize::from_le_bytes(value_int.try_into().expect("slice with incorrect length")),
            Some(value_int) => size_local_list= byte_to_number(value_int) as usize,
            None => size_local_list = 0,
        }
    }else{
        match tmp_size_list {
            //Some(value_int) => size_local_list = usize::from_be_bytes(value_int.try_into().expect("slice with incorrect length")),
            Some(value_int) => size_local_list= byte_to_number_be(value_int) as usize,
            None => size_local_list = 0,
        } 
    }
    tmp=tmp+size_int;
    println!("hey size_upvalue = {} ",size_local_list);
    while i < size_local_list {
        //string 
        println!(" i il vaut {} ",i);
        let taille ;
        tmp_size_list = ls.get(tmp..tmp+size_t);
        tmp=tmp+size_t;
        if endian == 1 {
            match tmp_size_list {
                Some(value_int) => taille= usize::from_le_bytes(value_int.try_into().expect("slice with incorrect length")),
                None => taille = 0,
            }
        }else{
            match tmp_size_list {
                Some(value_int) => taille = usize::from_be_bytes(value_int.try_into().expect("slice with incorrect length")),
                None => taille = 0,
            } 
        }
        let titre_op = ls.get(tmp..(tmp+taille));
        tmp=tmp+taille;
        let mut titre: Vec<char> = Vec::new();
        match titre_op {
            Some(val_titre) => titre = convert_to_chaine(val_titre) ,
            None => println!("No titre"),   
        }
        println!("valeur du titre : {:?} ",titre);
        i=i+1;
    }
    tmp
}
fn affiche_header(ls :&[u8],verbose : bool)  ->(i32,usize,usize,usize,usize,usize,usize){
    let taille_fic: usize = ls.len();
    let header = ls.get(0..4).unwrap_or(&[]);
    let version_n = unwrap_to_i32(ls.get(4),-1);
    let format_v = unwrap_to_i32(ls.get(5),-1);
    let endian = unwrap_to_i32(ls.get(6),-1);
    let size_int = unwrap_to_i32(ls.get(7),-1) as usize;// Taille d'un Integer
    let size_st = unwrap_to_i32(ls.get(8),-1) as usize;// Taille d'un Size_T
    let size_inst = unwrap_to_i32(ls.get(9),-1) as usize;
    let size_luanb = unwrap_to_i32(ls.get(10),-1) as usize;
    let size_flag = unwrap_to_i32(ls.get(11),-1) as usize;
    if verbose {
        println!("La valeur de bitvec est : {:?} \net sa taille {taille_fic} ", ls);
        println!("header : {:?}", header);
        println!("version : {:?}", version_n);
        println!("format : {:?}", format_v);
        println!("endian : {:?}", endian);
        println!("size_int : {:?}", size_int);
        println!("size_st : {:?}", size_st);
        println!("size_inst : {:?}", size_inst);
        println!("size_luanb : {:?}", size_luanb);
        println!("size_flag : {:?}", size_flag);
    }
    (endian,size_int,size_st,size_inst,size_luanb,size_flag,taille_fic)
}

// Instuction list
// Les instructions font 4 bytes = 32 octets (1 bytes = 8 bits = 1 octets)
fn parse_inst_list(ls : &[u8], begin : usize,size_int : usize,size_inst:usize,endian:i32) -> usize {

    let mut taille_ls_inst: usize = 0;
    let size_code_inst: Option<&[u8]> = ls.get(begin..begin+size_int);
    match size_code_inst {
        //Some(value_line) => taille_ls_inst= byte_to_number(value_line) as usize,
        Some(value_line) => if endian == 1 {
            taille_ls_inst = u32::from_le_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        }else{
            taille_ls_inst = u32::from_be_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        },
        None => println!("no Value"),
    }
    println!("Nombre des inst du code en liste de byte {:?} ",size_code_inst);
    
    println!("Nombre des inst du code en entier : {} ",taille_ls_inst); 
    let id_code_inst = begin+size_int;
    let code_inst: Option<&[u8]> = ls.get(id_code_inst..id_code_inst+(taille_ls_inst*size_inst));
    println!("code value : {:?} ",code_inst);
    if let Some(code_inst) = code_inst { // Fait par Copilot  
        affiche_op_inst(code_inst,taille_ls_inst,endian);
    } else {
        println!("No instructions found");
    }
    id_code_inst+(taille_ls_inst*size_inst)
}


fn main() -> io::Result<()> {
    let mut file = File::open("luac_copy.out")?;
    let mut buffer = Vec::new();
    io::copy(&mut file, &mut buffer)?; // en décimal
    file.seek(SeekFrom::Start(0))?;
    let res=affiche_header(&buffer, true);
    let endian = res.0;
    let size_int = res.1;
    let size_st = res.2;
    let size_inst = res.3;
    let size_luanb = res.4;
    let size_flag = res.5;
    let taille_fichier = res.6;
    //Body 

    let chunk = buffer.get(12..taille_fichier);
    match chunk {
        Some(chunk) => println!("chunk : {:?}",chunk),
        None => println!("No chunk"),   
    }
    
    let to_name : usize = 12+size_st;
    let size_name = buffer.get(12..to_name);//12+valeur de size_st_op (même -1 pour ignorer le dernier caractère qui vaut 0)
    match size_name {
        Some(size_name) => println!("size_name : {:?}",size_name),
        None => println!("No size_name"),   
    }
    let mut size_t_value = 0;
    match size_name {
        //Some(name_value) => size_t_value = usize::from_le_bytes(name_value.try_into().expect("Erreur de conversion")), // la fonction a été pris par ChatGpt
        Some(name_value) => size_t_value = byte_to_number(name_value) as usize ,
        None => println!("No size_t_value"),
    }
    
    println!("size_t_value : {} ",size_t_value);

    let titre_op = buffer.get(to_name..(to_name+size_t_value));
    let mut titre: Vec<char> = Vec::new();
    match titre_op {
        Some(val_titre) => titre = convert_to_chaine(val_titre) ,
        None => println!("No titre"),   
    }
    println!("valeur du titre : {:?} ",titre);

    let first_line_int = to_name+size_t_value;
    
    let first_line = buffer.get((first_line_int)..(first_line_int+size_int));
    match first_line {
        Some(value_line) => println!("first_line : {:?}",value_line),
        None => println!("No first_line"),   
    }

    let last_line_int= first_line_int+size_int;
    
    let last_line = buffer.get((last_line_int)..(last_line_int+size_int));
    match last_line {
        Some(value_line) => println!("last_line : {:?}",value_line),
        None => println!("No last_line"),   
    }
    
    let id_1 = last_line_int+size_int;

    let nb_upval = unwrap_to_i32(buffer.get(id_1),-1) as usize;
    println!("nb_upval : {:?}", nb_upval);

    let nb_param = unwrap_to_i32(buffer.get(id_1+1),-1) as usize;
    println!("nb_param : {:?}", nb_param);

    let var_flag = unwrap_to_i32(buffer.get(id_1+2),-1) as usize;
    println!("var_flag : {:?}", var_flag);

    let max_stack_sz = unwrap_to_i32(buffer.get(id_1+3),-1) as usize;
    println!("max_stack_sz : {:?}", max_stack_sz);
    
    let id_cs = parse_inst_list(&buffer, id_1+4,size_int,size_inst,endian);
    
    let mut id_func_proc = parse_const_list(&buffer, id_cs,size_int,size_st,endian);

    // Function protocole 
    //Debut parsing function protocole 
    println!("debut des fonction protocole = {} ",id_func_proc);

    let size_func_proc = buffer.get(id_func_proc..id_func_proc+size_int);
    let mut taille_func_proc = -1;
    match size_func_proc {
        //Some(value_line) => taille_inst= byte_to_number(value_line) as usize,
        Some(value_line) => taille_func_proc = i32::from_le_bytes(value_line.try_into().expect("slice with incorrect length")),
        None => println!("no Value"),
    }
    println!("Nombre des inst du code en liste de byte {:?} ",size_func_proc);
    
    println!("Nombre des inst du code en entier : {} ",taille_func_proc);

    id_func_proc=parse_func_block(&buffer, id_func_proc+size_int, taille_func_proc , size_int, size_st,size_inst, endian);
    // fin parsing function protocole 

    let mut tmp = parse_source_line(&buffer,id_func_proc , size_int, endian);
    tmp= parse_local_list(&buffer,tmp , size_int,size_st, endian);
    println!("tmp avant upvalue {} ",tmp);
    tmp=parse_upvalue_list(&buffer,tmp , size_int,size_st, endian);
    println!("tmp après upvalue {} ",tmp);
    Ok(())
}