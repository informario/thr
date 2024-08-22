use core::time;
use std::{env, io::{self, Read, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, thread, time::Duration, error::Error};
use gtk::{cairo::Result as CairoResult, gio::JoinHandle as GioJoinHandle, prelude::*};
use gtk::{glib, Application};
//const APP_ID: &str = "org.gtk_rs.HelloWorld1";


fn main() {
    let args = env::args().collect();
    let mut handles: Vec<std::thread::JoinHandle<()>>  = vec![];
    match bind_and_connect(&args, handles){
        Ok(_) => (),
        Err(e) => println!("Error: {}", e),
    }
}


fn bind_and_connect(args:&Vec<String>, mut handles:Vec<std::thread::JoinHandle<()>>) -> Result<(), Box<dyn Error>> {
    let listener;
    if args.len() > 1 {
        println!("Busco conectarme a: {}", args[1]);
        let stream = TcpStream::connect((args[1].clone(), 8080))?;
        stream.set_nonblocking(true)?;
        let handle = thread::spawn(|| {
            handle_connection(stream).expect("un HandleConnection se rompió");
        });
        handles.push(handle);
        listener = TcpListener::bind("127.0.0.77:8080")?;
        println!("Yo soy 127.0.0.77:8080")
    }
    else{
        println!("Solo recibo");
        listener = TcpListener::bind("127.0.0.76:8080")?;
        println!("Yo soy 127.0.0.76:8080")
    }

    for stream in listener.incoming() {
        let stream = stream?;
        stream.set_nonblocking(true)?;
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


fn handle_connection(stream:TcpStream)-> Result<(), Box<dyn Error>>{
    let arc_stream = Arc::new(Mutex::new(stream));

    let arc_stream_clone = Arc::clone(&arc_stream);
    let send_handle = thread::spawn(move || {
        println!("{:?}", send(&arc_stream_clone));
    });

    let arc_stream_clone = Arc::clone(&arc_stream);
    let receive_handle = thread::spawn(move || {
        println!("{:?}", receive(&arc_stream_clone));
    });
    send_handle.join().expect("Este hilo se rompió");  
    receive_handle.join().expect("Este hilo se rompió");
    return Ok(());
}

fn send(arc_stream:&Arc<Mutex<TcpStream>>) -> Result<(), Box<dyn Error>> {
    loop {
        thread::sleep(time::Duration::from_millis(20));
        let mut stream = match arc_stream.lock(){
            Ok(s) => s,
            Err(_) => return Err("Error: obteniendo lock de stream".into()),
        };
        stream.write(b"adio")?;
    }
    }

fn receive(arc_stream:&Arc<Mutex<TcpStream>>) -> Result<(), Box<dyn Error>> {
    loop { 
        thread::sleep(Duration::from_millis(20)); 
        let mut stream = match arc_stream.lock(){
            Ok(s) => s,
            Err(_) => return Err("Error: obteniendo lock de stream".into()),
        };

        let mut buf: [u8;6] = [0;6];
        match stream.read(&mut buf){
            Ok(_) => (),
            Err(_) => println!("error"),
        };
        println!("{:?}", &buf);
    }
}