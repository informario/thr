use core::time;
use std::{env, io::{self, Read, Write}, net::{TcpListener, TcpStream}, sync::{Arc, Mutex}, thread, time::Duration};



fn main() {
    let args: Vec<_> = env::args().collect();
    let mut handles = vec![];
    let listener;
    if args.len() > 1 {
        println!("Busco conectarme a: {}", args[1]);
        let stream = TcpStream::connect((args[1].clone(), 8080)).unwrap();
        stream.set_nonblocking(true).unwrap();
        let handle = thread::spawn(|| {
            handle_connection(stream);
        });
        handles.push(handle);
        listener = TcpListener::bind("127.0.0.77:8080").unwrap();
        println!("Yo soy 127.0.0.77:8080")
    }
    else{
        println!("Solo recibo");
        listener = TcpListener::bind("127.0.0.76:8080").unwrap();
        println!("Yo soy 127.0.0.76:8080")
    }

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        stream.set_nonblocking(true).unwrap();
        let handle = thread::spawn(|| {
            handle_connection(stream);
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap(); // Esperamos a que cada hilo termine
    }
}


fn handle_connection(stream:TcpStream){
    let arc_stream = Arc::new(Mutex::new(stream));

    let arc_stream_clone = Arc::clone(&arc_stream);
    let send_handle = thread::spawn(move || {
        send(arc_stream_clone);
    });


    let arc_stream_clone = Arc::clone(&arc_stream);
    let receive_handle = thread::spawn(move || {
        receive(arc_stream_clone);
    });
    send_handle.join().unwrap();    
    receive_handle.join().unwrap();
}

fn send(arc_stream:Arc<Mutex<TcpStream>>){
    loop {
        println!("ss1");
        thread::sleep(time::Duration::from_millis(20));
        println!("ss2");
        let mut stream = arc_stream.lock().unwrap();
        println!("ss3");
        stream.write(b"adios").unwrap();
        println!("ss4");
    }
    }

fn receive(arc_stream:Arc<Mutex<TcpStream>>){
    loop {
        println!("rr1");
        thread::sleep(Duration::from_millis(20));
        println!("rr2");
        let mut stream = arc_stream.lock().unwrap();
        println!("rr3");
        let mut buf: [u8;6] = [0;6];
        println!("rr4");
        match stream.read(&mut buf){
            Ok(_) => (),
            Err(_) => println!("error"),
        };
        println!("rr5");
        println!("{:?}", &buf);
        println!("rr6");
    }
}