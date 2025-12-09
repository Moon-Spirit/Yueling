mod api;
mod storage;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use axum::{Router, routing::post};
use std::sync::Arc;
use tokio::net::TcpListener as OtherTcpListener;

fn main(){      
     let port = 2025;
     println!("Starting chat server on port {}", port);

    // 启动TCP服务器
     thread::spawn(move || {
         println!("TCP server listening on port {}", port);
         if let Err(e) = run_tcp_server(port) {
             eprintln!("TCP server error: {}", e);
        }
    });

    // 保持主线程运行
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// 运行TCP服务器
fn run_tcp_server(port: u16) -> std::io::Result<()> {
    let listener = TcpListener::bind(&format!("0.0.0.0:{}", port))?;
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("TCP client connected: {}", stream.peer_addr()?);
                thread::spawn(move || {
                    if let Err(e) = handle_tcp_client(stream) {
                        eprintln!("TCP client error: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("TCP connection error: {}", e);
            }
        }
    }
    
    Ok(())
}

// 处理TCP客户端连接
fn handle_tcp_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0u8; 1024];
    
    loop {
        let n = stream.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        
        let message = String::from_utf8_lossy(&buffer[..n]);
        println!("Received TCP message: {}", message);
        
        // 简单回显
        stream.write_all(&buffer[..n])?;
    }
    
    println!("TCP client disconnected: {}", stream.peer_addr()?);
    Ok(())
}
