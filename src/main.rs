use core::time;
use std::{env, error::Error, io::{self, BufRead, BufReader, Read, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex, mpsc, mpsc::{Sender, Receiver}}, thread, time::Duration};



fn main() {
    let handles: Vec<std::thread::JoinHandle<()>>  = vec![];
    match bind_and_connect(&env::args().collect(), handles){
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}


fn bind_and_connect(args:&Vec<String>, mut handles:Vec<std::thread::JoinHandle<()>>) -> Result<(), Box<dyn Error>> {
    let listener;
    let mut my_addr = "127.0.0.76:8080";
    if args.len() > 1 {
        my_addr = "127.0.0.77:8080";
        println!("Busco conectarme a: {}", args[1]);
        let stream = TcpStream::connect((args[1].clone(), 8080))?;
        let handle = thread::spawn(|| {
            handle_connection(stream).expect("un HandleConnection se rompió");
        });
        handles.push(handle);
    }
    else{
        println!("Solo recibo");
    }   
    listener = TcpListener::bind(my_addr)?;
    println!("Yo soy {}", my_addr);
    

    for stream in listener.incoming() {
        let stream = stream?;
        let handle = thread::spawn(|| {
            handle_connection(stream).expect("un HandleConnection se rompió");
        });
        handles.push(handle);
    }


    for handle in handles {
        handle.join().expect("Este hilo se rompió"); // Esperamos a que cada hilo termine
    }
    return Ok(());
}


fn handle_connection(s:TcpStream)-> Result<(), Box<dyn Error>>{
    let r = s.try_clone()?;

    let (tx,rx):(Sender<i32>, Receiver<i32>) = mpsc::channel();

    let send_handle = thread::spawn(move || {
        match send(s, rx){
            Ok(_) => {},
            Err(e) => println!("{}", e),
        }
    });

    let receive_handle = thread::spawn(move || {
        match receive(r, tx){
            Ok(_) => {},
            Err(e) => println!("{}", e),
        }
    });

    send_handle.join().expect("Este hilo se rompió");  
    receive_handle.join().expect("Este hilo se rompió");
    return Ok(());
}

fn send(mut stream:TcpStream, rx:Receiver<i32>) -> Result<(), Box<dyn Error>> {
    loop {
        thread::sleep(time::Duration::from_millis(20));
        if rx.try_recv().is_ok() {
            println!("Hilo C: recibió la señal de terminar.");
            break;
        }
        let stdin = io::stdin();
        let mut stdin = stdin.lock();
        let received: Vec<u8> = stdin.fill_buf()?.to_vec();
        stdin.consume(received.len());

        let sent = stream.write(&received)?;
        let s = match String::from_utf8(received){
            Ok(s) => {
                //println!("Sent: {}",s);
            },
            Err(_) => println!("Error: string from utf8"),
        };
    }
    return Ok(());
}

fn receive(mut stream:TcpStream, tx:Sender<i32>) -> Result<(), Box<dyn Error>> {
    let mut reader = BufReader::new(stream);
    loop { 
        thread::sleep(Duration::from_millis(20)); 
        let received: Vec<u8> = reader.fill_buf()?.to_vec();
        reader.consume(received.len());
        if received.len()==0{
            println!("connection closed");
            tx.send(0);
            break
        }


        let s = match String::from_utf8(received){
            Ok(s) => {
                println!("Received: {}",s);
            },
            Err(_) => println!("Error: string from utf8"),
        };
    }
    return Ok(());
}