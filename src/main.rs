use std::{
    collections::HashMap,
    env::{self, Args},
    io::stdin,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    process::Stdio,
    str::FromStr,
    sync::mpsc::channel,
    thread,
};

fn main() {
    let mut args = env::args();

    args.next();

    let mode = args.next();

    if let Some(m) = mode
        && m == "--client"
    {
        client(args);
        return;
    }

    server()
}

fn client(mut args: Args) {
    let address = args.next().expect("provide address");

    println!("Connecting to address {}", address);
    let udp_socket =
        UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("couldn't bind to address client");

    println!("Udp socket bound");

    udp_socket
        .connect(address)
        .expect("cannot connet to server");

    println!("Connected to server");

    if let Some(key) = args.next() {
        let key = format!("k:{}", key);
        udp_socket
            .send(key.as_bytes())
            .expect("Could not send the message");

        let message_str = recv(&udp_socket);

        println!("Received address: {}", message_str);
        let address = SocketAddr::from_str(message_str.as_ref()).expect("Invalid address");

        udp_socket
            .connect(address)
            .expect("Cannot connect to other client");

        udp_socket
            .send(b"Ciao")
            .expect("Cannot send message to other client");

        let message_str = recv(&udp_socket);

        println!("Received message from other client: {}", message_str);

        udp_socket
            .send(b"Come stai")
            .expect("Cannot send message to other client");

        run_chat(udp_socket);
    } else {
        udp_socket
            .send(b"ping")
            .expect("Could not send the message");

        println!("Client: Message sent");

        let message_str = recv(&udp_socket);
        println!("client received {}", message_str);
    }
}

fn server() {
    let udp_socket = UdpSocket::bind("0.0.0.0:3400").expect("couldn't bind to address");

    let mut map: HashMap<String, SocketAddr> = HashMap::new();

    let mut buf = [0; 1024];
    println!("Starting");

    loop {
        let (number_of_bytes, src_addr) = udp_socket.recv_from(&mut buf).expect("error here");

        let message = &buf[..number_of_bytes];

        let message_str = String::from_utf8_lossy(message);

        println!("received message: {}", &message_str);

        if message_str == "ping" {
            udp_socket
                .send_to(b"pong", src_addr)
                .expect("tutto okay, non funziona");
        }

        if message_str.starts_with("k:") {
            let key = message_str;

            if let Some(address_a) = map.remove(&key.to_string()) {
                println!("address_a: {}", address_a.to_string());
                let address_b = src_addr;
                println!("address_b: {}", address_b.to_string());

                if address_a == address_b {
                    println!("same address, reinserting");
                    map.insert(key.to_string(), src_addr);
                    return;
                }

                let message = address_b.to_string();
                udp_socket
                    .send_to(message.as_bytes(), address_a)
                    .expect("cannot send the message to address a");

                let message = address_a.to_string();
                udp_socket
                    .send_to(message.as_bytes(), address_b)
                    .expect("cannot send the message to address b");
            } else {
                println!("first message, storing address {}", src_addr);
                map.insert(key.to_string(), src_addr);
            }
        }
    }
}

fn run_chat(udp_socket: UdpSocket) {
    let (tx_stdin, rx_stdin) = channel();
    let (tx_udp, rx_udp) = channel();

    thread::spawn(move || {
        loop {
            let mut buf = String::new();
            if let Ok(_) = stdin().read_line(&mut buf) {
                tx_stdin.send(buf).unwrap();
            } else {
                tx_stdin.send("exit".to_string()).unwrap();
            }
        }
    });

    thread::spawn({
        let udp_socket = udp_socket
            .try_clone()
            .expect("Should be able to clone a udp socket");
        move || {
            loop {
                let received_message = recv(&udp_socket);
                tx_udp
                    .send(received_message)
                    .expect("Cannot send on channel");
            }
        }
    });

    loop {
        for message in rx_stdin.try_iter() {
            if message == "exit" {
                udp_socket
                    .send(b"DISCONNECTED")
                    .expect("Cannot send message");
                println!("EXITING");
                return;
            }

            udp_socket
                .send(message.as_bytes())
                .expect("Cannot send message");
        }

        for message in rx_udp.try_iter() {
            println!("Message received: {}", message);
        }
    }
}

fn recv(udp_socket: &UdpSocket) -> String {
    let mut buf = [0; 1024];
    let number_of_bytes = udp_socket.recv(&mut buf).expect("Cannot receive messages");

    println!("Client: Message received");

    let message = &buf[..number_of_bytes];
    String::from_utf8_lossy(message).to_string()
}
