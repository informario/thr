use std::error::Error;
use std::io::Write;
use bitcoin_hashes::sha256;
use bitcoin_hashes::Hash;

pub enum Messages{
    Nice2MeetU(MessageNice2MeetU),
    Text(MessageText),
    Unknown(),
}


pub const MAGIC: [u8;4]= [121,104,119,104];
pub const MAGIC_CHAR : [char;4]= ['y','h','w','h'];



pub struct Header {
    pub magic: [char;4],
    pub cmd_name: [char;12],
    pub size: u32,
    pub checksum: [u8;4],
}

pub struct MessageNice2MeetU {
    pub username: [char;64],
}

pub struct MessageText{
    pub content:Vec<char>
}


pub fn check_checksum(body:&Vec<u8>, checksum1:[u8;4])-> Result<(), Box<dyn Error>>{
    let checksum2 = gen_checksum(body)?;
    //println!("check1: {:?}", checksum1);
    //println!("check2: {:?}", checksum2);
    match checksum1 == checksum2{
        true => return Ok(()),
        false => return Err("checksum not the same".into()),
    }
}

pub fn gen_checksum(body:&Vec<u8>)->Result<[u8; 4], Box<dyn Error>>{
    //println!("gen_checksum:{:?}", body);
    let mut engine = sha256::HashEngine::default();
    engine.write_all(body)?;
    let hash = sha256::Hash::from_engine(engine);
    let first_four:[u8;4] = hash.as_byte_array()[0..4].try_into()?;
    return Ok(first_four);
}


pub fn get_message_deserialized(command_u8:[u8;12], body:Vec<u8>)->Result<Messages, Box<dyn Error>>{
    let command_string = String::from_utf8(command_u8.to_vec())?;
    let command = command_string.as_str();
    //println!("received command: {}", command);
    match command{
        "nice2meetu\0\0"=> {
            let mut username = [0u8; 64];
            let len_to_copy = body.len().min(64);
            username[..len_to_copy].copy_from_slice(&body[..len_to_copy]);
            let mut username_as_char:[char;64] = ['\0';64];
            for i in 0..64{
                username_as_char[i] = username[i].try_into()?;
            }
            let new_message = MessageNice2MeetU{username:username_as_char};
            return Ok(Messages::Nice2MeetU(new_message));
        },
        "text\0\0\0\0\0\0\0\0"=> {
            let mut char_body: Vec<char> = vec![];
            for i in 0..body.len(){
                char_body.push(body[i].try_into()?);
            }
            let new_message = MessageText{content:char_body};
            return Ok(Messages::Text(new_message));
        },
        _=>{return Err("unknown message type".into())},
    }
}

pub fn serialize_header(header:Header)->Result<[u8;24], Box<dyn Error>>{
    let mut array:[u8;24] = [0;24];
    for i in 0..4{
        array[i] = header.magic[i].try_into()?;
    }
    for i in 4..16{
        array[i] = header.cmd_name[i-4].try_into()?;
    }
    let size_into_array:[u8;4]= header.size.to_le_bytes();
    for i in 16..20{
        array[i] = size_into_array[i-16].try_into()?;
    }
    for i in 20..24{
        array[i] = header.checksum[i-20].try_into()?;
    }
    return Ok(array);
}



/*
pub trait Message {
    fn serialize(&self) -> Result<Vec<u8>, Box<dyn Error>>;
}


pub fn deserialize_header(data:Vec<u8>) -> Result<Messages, Box<dyn Error>>{
    let n2mu = "nice2meetu".to_string();
    let txt = "text".to_string();

    if data.len()<24{
        return Err("Longitud header inválida".into());
    }
    let string: String = magic.iter().collect();
    if data[0 .. 4] != string.into_bytes(){
        return Err("Magic inválido".into());
    }
    
    check_checksum(body, checksum)

    let cmd_name:String = String::from_utf8_lossy(&data[4..16]).to_string();
    match cmd_name{
        n2mu=> {return Ok(Messages::Nice2MeetU)},
        txt=> {return Ok(Messages::Nice2MeetU)},
        _=>{return Err("unknown message type".into())},
    }
}

 */