use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use std::convert::TryInto;
/*
1ère étape : Parsing du header de luac.out 
2ème étape : Parsing des function blocks 
3ème étape : Dump des instructions 
4ème étape : Opérations arithmétiques simples 
5ème étape : Gestion de la primitive print 
6ème étape : Ajout d'autres primitives (Optionnel) 
*/
const OPCODE_NAMES: [&str; 38] = [
        "MOVE", "LOADK", "LOADBOOL", "LOADNIL", "GETUPVAL", "GETGLOBAL",
        "GETTABLE", "SETGLOBAL", "SETUPVAL", "SETTABLE", "NEWTABLE", "SELF",
        "ADD", "SUB", "MUL", "DIV", "MOD", "POW", "UNM", "NOT", "LEN",
        "CONCAT", "JMP", "EQ", "LT", "LE", "TEST", "TESTSET", "CALL", "TAILCALL",
        "RETURN", "FORLOOP", "FORPREP", "TFORLOOP", "SETLIST", "CLOSE", "CLOSURE",
        "VARARG"
    ];

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

fn parse_const_list(ls : &[u8], begin : usize,taille : usize,endian:i32) -> usize {// va lire les bytes concernant la
    let mut tmp = begin ;
    let mut i = 0;
    while i < taille {
        let type_const = unwrap_to_i32(ls.get(tmp), -1);
        println!("Hey");
        tmp=tmp+1;
        if type_const == 0 { // il n'y a pas de data on ignore 
            println!("Ignore");
        }
        if type_const == 1 { // Booléan donc 0 ou 1 
            let boolean = unwrap_to_i32(ls.get(tmp), -1);
            if boolean==0{
                println!("Boolean = True");
            }else if boolean==1 {
                println!("Boolean = False");
            }else{
                println!("Boolean = ???");
            }
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
            println!("valeur du nombre lua = {} ", lua_numb);
        }
        if type_const == 4 {
            let taille_str = unwrap_to_i32(ls.get(tmp), -1);
            println!("taille du string = {} ",taille_str);
            tmp=tmp+1;
            let val_str = ls.get(tmp..tmp+(taille_str as usize));
            let mut affiche_str: Vec<char> = Vec::new();
            match val_str {
                Some(val_titre) => affiche_str = convert_to_chaine(val_titre) ,
                None => println!("No string"),   
            }
            println!("valeur du titre : {:?} ",affiche_str);
            
            tmp=tmp+tmp+(taille_str as usize);

        }
        i=i+1;
    }
    tmp
}


fn parse_func_block(ls : &[u8], begin : usize,taille : i32,size_int : usize,size_t:usize,endian:i32) -> usize {
    if taille <= 0{return begin;}
    let tmp ;
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
    let id_ls = id_1+4;
    let mut taille_inst: usize = 0;
    let size_code_inst: Option<&[u8]> = ls.get(id_ls..id_ls+size_int);
    match size_code_inst {
        //Some(value_line) => taille_inst= byte_to_number(value_line) as usize,
        Some(value_line) => if endian == 1 {
            taille_inst = u32::from_le_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        }else{
            taille_inst = u32::from_be_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        },
        None => println!("no Value"),
    }
    println!("Nombre des inst du code en liste de byte {:?} ",size_code_inst);
    
    println!("Nombre des inst du code en entier : {} ",taille_inst); // c'est une liste de taille Integer 
    let id_code_inst = id_ls+size_int;
    let code_inst: Option<&[u8]> = ls.get(id_code_inst..id_code_inst+(taille_inst*4));
    println!("code value : {:?} ",code_inst);
    //Constant list
    // Number of constants - Integer 
   
    let id_cs = id_code_inst+(taille_inst*4);
    let mut taille_inst: usize = 0;
    let size_const_ls: Option<&[u8]> = ls.get(id_cs..id_cs+size_int);
    match size_const_ls {
        //Some(value_line) => taille_inst= byte_to_number(value_line) as usize,
        Some(value_line) => if endian == 1 {
            taille_inst = u32::from_le_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        }else{
            taille_inst = u32::from_be_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        },
        None => println!("no Value"),
    }
    println!("Nombre de constantes du code en liste de byte {:?} ",size_const_ls);

    println!("Nombre des constantes : {} ",taille_inst); 

    let id_cs = id_cs + size_int;
    let mut id_func_proc = parse_const_list(ls, id_cs, taille_inst,endian);

    println!("debut des fonction protocole = {} ",id_func_proc);

    let size_func_proc = ls.get(id_func_proc..id_func_proc+size_int);
    let mut taille_func_proc = -1;
    match size_func_proc {
        //Some(value_line) => taille_inst= byte_to_number(value_line) as usize,
        Some(value_line) => taille_func_proc = i32::from_le_bytes(value_line.try_into().expect("slice with incorrect length")),
        None => println!("no Value"),
    }
    println!("taille de la fonction en liste d'octet {:?} ",size_func_proc); 
    println!("taille de la fonction en entier : {} ",taille_func_proc);

    id_func_proc=parse_func_block(ls, id_func_proc+size_int, taille_func_proc , size_int, size_t, endian);

    // c'est une liste de taille Integer 
    let mut retour = parse_source_line(ls,id_func_proc+size_int+(taille_func_proc as usize) , size_int, endian);
    retour= parse_local_list(ls,retour , size_int,size_t, endian);
    println!("tmp avant upvalue {} ",retour);
    tmp=parse_upvalue_list(ls,retour , size_int,size_t, endian);
    println!("tmp après upvalue {} ",retour);

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
fn main() -> io::Result<()> {
    let mut file = File::open("luac.out")?;
    let mut buffer = Vec::new();
    
    io::copy(&mut file, &mut buffer)?; // en décimal
    
    file.seek(SeekFrom::Start(0))?;
    
    let taille_fic: usize = buffer.len();
    println!("La valeur de bitvec est : {:?} \net sa taille {taille_fic} ", buffer);

    let header = buffer.get(0..4).unwrap_or(&[]);
    println!("header : {:?}", header);

    let version_n = unwrap_to_i32(buffer.get(4),-1);
    println!("version : {:?}", version_n);

    let format_v = unwrap_to_i32(buffer.get(5),-1);
    println!("format : {:?}", format_v);

    let endian = unwrap_to_i32(buffer.get(6),-1);
    println!("endian : {:?}", endian);

    let size_int = unwrap_to_i32(buffer.get(7),-1) as usize;// Taille d'un Integer
    println!("size_int : {:?}", size_int);

    let size_st = unwrap_to_i32(buffer.get(8),-1) as usize;// Taille d'un Size_T
    println!("size_st : {:?}", size_st);

    let size_inst = unwrap_to_i32(buffer.get(9),-1) as usize;
    println!("size_inst : {:?}", size_inst);

    let size_luanb = unwrap_to_i32(buffer.get(10),-1) as usize;
    println!("size_luanb : {:?}", size_luanb);

    let size_flag = unwrap_to_i32(buffer.get(11),-1) as usize;
    println!("size_flag : {:?}", size_flag);

    //Body 

    let chunk = buffer.get(12..taille_fic);
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
    
    // Instuction list
    // Les instructions font 4 bytes = 32 octets (1 bytes = 8 bits = 1 octets)
    let id_ls = id_1+4;
    let mut taille_inst: usize = 0;
    let size_code_inst: Option<&[u8]> = buffer.get(id_ls..id_ls+size_int);
    match size_code_inst {
        //Some(value_line) => taille_inst= byte_to_number(value_line) as usize,
        Some(value_line) => if endian == 1 {
            taille_inst = u32::from_le_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        }else{
            taille_inst = u32::from_be_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        },
        None => println!("no Value"),
    }
    println!("Nombre des inst du code en liste de byte {:?} ",size_code_inst);
    
    println!("Nombre des inst du code en entier : {} ",taille_inst); // c'est une liste de taille Integer 
    let id_code_inst = id_ls+size_int;
    let code_inst: Option<&[u8]> = buffer.get(id_code_inst..id_code_inst+(taille_inst*4));
    println!("code value : {:?} ",code_inst);
    let first_inst=buffer.get(id_code_inst..id_code_inst+(taille_inst));
    if let Some(code_inst) = code_inst { // Fait par Copilot  
        for i in 0..taille_inst {
            let inst = &code_inst[i*4..(i+1)*4];
            let opcode = inst[0] >> 2; // Les 6 premiers bits
            let opcode_str = format!("{:06b}", opcode); // Convertir en chaîne de caractères binaire
            //println!("Instruction {}: Opcode : {}", i, opcode_str);
            if opcode < OPCODE_NAMES.len() as u8 {
                println!("Instruction {}: Opcode : {} ({})", i, opcode_str, OPCODE_NAMES[opcode as usize]);
            } else {
                println!("Instruction {}: Opcode : {} (Unknown Opcode)", i, opcode_str);
            }
        }
    } else {
        println!("No instructions found");
    }
        
    
    //Constant list
    // Number of constants - Integer 
   
    let id_cs = id_code_inst+(taille_inst*4);
    let mut taille_inst: usize = 0;
    let size_const_ls: Option<&[u8]> = buffer.get(id_cs..id_cs+size_int);
    match size_const_ls {
        //Some(value_line) => taille_inst= byte_to_number(value_line) as usize,
        Some(value_line) => if endian == 1 {
            taille_inst = u32::from_le_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        }else{
            taille_inst = u32::from_be_bytes(value_line.try_into().expect("slice with incorrect length")) as usize
        },
        None => println!("no Value"),
    }
    println!("Nombre de constantes du code en liste de byte {:?} ",size_const_ls);

    println!("Nombre des constantes : {} ",taille_inst); 

    let id_cs = id_cs + size_int;
    let mut id_func_proc = parse_const_list(&buffer, id_cs, taille_inst,endian);

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

    id_func_proc=parse_func_block(&buffer, id_func_proc+size_int, taille_func_proc , size_int, size_st, endian);
    // c'est une liste de taille Integer 
    let mut tmp = parse_source_line(&buffer,id_func_proc , size_int, endian);
    tmp= parse_local_list(&buffer,tmp , size_int,size_st, endian);
    println!("tmp avant upvalue {} ",tmp);
    tmp=parse_upvalue_list(&buffer,tmp , size_int,size_st, endian);
    println!("tmp après upvalue {} ",tmp);
    Ok(())
}