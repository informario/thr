use core::time;
use std::{env, error::Error, io::{self, BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, sync::mpsc::{self, Receiver, Sender, TryRecvError}, thread};

use protocol::{check_checksum, gen_checksum, get_message_deserialized, serialize_header, MessageNice2MeetU, MessageText, Messages};
mod protocol;

fn main() {
    let handles: Vec<std::thread::JoinHandle<()>>  = vec![];
    match bind_and_connect(&env::args().collect(), handles){
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}

fn bind_and_connect(args:&Vec<String>, mut handles:Vec<std::thread::JoinHandle<()>>) -> Result<(), Box<dyn Error>> {
    let listener;
    let mut my_addr;
    let username;
    let peer_addr;
    if args.len()<3{
        println!("argumentos insuficientes");
        return Ok(())
    }
    username = args[1].clone();
    my_addr = args[2].clone();
    my_addr.push_str(":8080");
    if args.len()==4 {
        peer_addr = args[3].clone();
        println!("Busco conectarme a: {}", peer_addr);
        match TcpStream::connect((peer_addr, 8080)){
            Ok(stream) => {
                let username_clone = username.clone();
                let handle = thread::spawn(|| {
                    handle_connection(stream, username_clone).expect("un HandleConnection se rompió");
                });
                handles.push(handle);
            },
            Err(e) => println!("Error estableciendo conexión: {}", e),
        };
    }
    else{
        println!("No ha sido ingresado un peer, escuchando conexiones entrantes...")
    }

    listener = TcpListener::bind(my_addr)?;
    
    for stream in listener.incoming() {
        let stream = stream?;
        let username_clone = username.clone();
        let handle = thread::spawn(|| {
            handle_connection(stream, username_clone).expect("un HandleConnection se rompió");
        });
        handles.push(handle);
    }


    for handle in handles {
        handle.join().expect("Este hilo se rompió"); // Esperamos a que cada hilo termine
    }
    return Ok(());
}


fn handle_connection(s:TcpStream, username:String)-> Result<(), Box<dyn Error>>{
    let r = s.try_clone()?;

    let (h2s_tx,h2s_rx):(Sender<Messages>, Receiver<Messages>) = mpsc::channel();
    let (r2h_tx,r2h_rx):(Sender<Messages>, Receiver<Messages>) = mpsc::channel();
    let (r2s_tx,r2s_rx):(Sender<i32>, Receiver<i32>) = mpsc::channel();
    let (i2h_tx, i2h_rx):(Sender<String>, Receiver<String>)= mpsc::channel();

    let mut received_ntmu=false;
    let mut sent_ntmu = false;

    let receive_handle = thread::spawn(move || {
        match receive_messages(s, r2s_tx, r2h_tx){
            Ok(_) => {},
            Err(e) => println!("{}", e),
        }
    });

    let send_handle = thread::spawn(move || {
        match send_messages(r, r2s_rx, h2s_rx){
            Ok(_) => {},
            Err(e) => println!("{}", e),
        }
    });
    let read_input_handle = thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            i2h_tx.send(line.unwrap()).unwrap();
            //println!("{}", line.unwrap());
            //cambiar esto
        }
        println!("end");
    });

    //let mut inputbuf =vec![];
    loop{
        match r2h_rx.try_recv(){
            Ok(m) => match m{
                Messages::Nice2MeetU(m) => {
                    println!("Estas chateando con: ");
                    for char in m.username{
                        print!("{}", char);
                    }
                    println!{};
                    received_ntmu = true;
                },
                Messages::Text(m) => {
                    if received_ntmu{
                        print!(">");
                        for char in m.content{
                            print!("{}", char);
                        }
                        println!{};
                    }
                },
                Messages::Unknown() => {},
            },
            Err(_) => {},
        }
        if sent_ntmu==false{
            let letters: Vec<char> = username.chars().collect();
            let mut array_of_chars: [char; 64] = ['\0';64];
            for i in 0..64{
                if i<letters.len() {
                    array_of_chars[i] = letters[i];
                }
                else{
                    array_of_chars[i]='\0';
                }
            }
            let n2mu = MessageNice2MeetU{username: array_of_chars};
            match h2s_tx.send(Messages::Nice2MeetU(n2mu)){
                Ok(_)=>{
                    sent_ntmu = true;
                },
                Err(_)=>{
                    println!("error sending n2mu")
                },
            }
        }
        thread::sleep(time::Duration::from_millis(200));
        match i2h_rx.try_recv(){
            Ok(s) => {
                let chars = s.chars().collect();
                let text = MessageText{content: chars};
                match h2s_tx.send(Messages::Text(text)){
                    Ok(_)=>{
                        sent_ntmu = true;
                    },
                    Err(e)=>{
                        println!("error sending text: {}", e)
                    },
                }
            },
            Err(e) => {
                match e{
                    TryRecvError::Empty=>{
                        
                    }
                    TryRecvError::Disconnected => break,
                }
            },
        }
    }
    read_input_handle.join().expect("Este hilo se rompió");
    send_handle.join().expect("Este hilo se rompió");  
    receive_handle.join().expect("Este hilo se rompió");
    return Ok(());
}

fn send_messages(mut stream:TcpStream, rx_signal:Receiver<i32>, rx_message:Receiver<Messages>) -> Result<(), Box<dyn Error>> {
    loop {
        if rx_signal.try_recv().is_ok() {
            println!("Hilo send: recibo la señal de terminar.");
            break;
        }
        match rx_message.recv(){
            Ok(s) => match s{
                Messages::Nice2MeetU(m) => {
                    let mut payload:Vec<u8> = vec![];
                    for char in m.username{
                        payload.push(char.try_into().expect("error serializing payload"))
                    }
                    let header = protocol::Header{
                        magic:protocol::MAGIC_CHAR,
                        cmd_name:['n','i','c','e','2','m','e','e','t','u','\0','\0',],
                        size:64,
                        checksum:gen_checksum(&payload)?
                    };
                    match serialize_header(header){
                        Ok(s) => {
                            stream.write(&s)?;
                            stream.write(&payload)?;
                            //println!("written n2mu into tcp stream: {:?} {:?}", s, payload);
                        },
                        Err(e) => {
                            println!("error serializando: {}", e);
                            continue;
                        },
                    }
                },
                Messages::Text(m) => {
                    let mut payload:Vec<u8> = vec![];
                    for char in m.content{
                        payload.push(char.try_into().expect("error serializing payload"))
                    }
                    let header = protocol::Header{
                        magic:protocol::MAGIC_CHAR,
                        cmd_name:['t','e','x','t','\0','\0','\0','\0','\0','\0','\0','\0',],
                        size:payload.len().try_into().expect("error obteniendo tamaño payload text"),
                        checksum:gen_checksum(&payload)?
                    };
                    match serialize_header(header){
                        Ok(s) => {
                            stream.write(&s)?;
                            stream.write(&payload)?;
                            //println!("written text into tcp stream: {:?} {:?}", s, payload);
                        },
                        Err(e) => {
                            println!("error serializando: {}", e);
                            continue;
                        },
                    }
                },
                Messages::Unknown() => println!("unknown message, not sending"),
            },
            Err(e) => {
                println!("send_messages handle: {}",e);
                break;
            },
        }
    }
    return Ok(());
}

fn receive_messages(stream:TcpStream, tx_signal:Sender<i32>, tx_message:Sender<Messages>) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(stream);
    let mut buffer: Vec<u8> = vec![];
    let mut previous_size=0;
    loop{
        buffer.append(&mut reader.fill_buf()?.to_vec());
        reader.consume(buffer.len() - previous_size);
        if buffer.len()==0{
            println!("connection closed");
            tx_signal.send(0)?;
            break;
        }
        while buffer.len() >= 24{
            let command:[u8;12] = buffer[4..16].try_into()?;
            let checksum:[u8;4] = buffer[20..24].try_into()?;
            let size_slice:[u8;4] = buffer[16..20].try_into()?;
            let len = u32::from_le_bytes(size_slice) as usize;
            if buffer.len() < 24 + len {
                break;
            }
            else{
                let payload:Vec<u8> = buffer[24..24+len].to_vec();
                match check_checksum(&payload, checksum.try_into()?){
                    Ok(_) => {
                        match get_message_deserialized(command, payload){
                            Ok(s) => tx_message.send(s)?,
                            Err(e) => {
                                println!("{}", e);
                                tx_message.send(Messages::Unknown())?;
                            }
                        }
                    },
                    Err(e) => {
                        println!("{}", e);
                    },
                }
                buffer.clear();
                previous_size = buffer.len();
            }
        }
    }

    return Ok(());
}
