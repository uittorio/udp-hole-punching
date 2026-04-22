use std::{
    env,
    net::{Ipv4Addr, UdpSocket},
};

fn main() {
    let mut args = env::args();

    args.next();

    let mode = args.next();

    if let Some(m) = mode
        && m == "--client"
    {
        let address = args.next().expect("provide address");

        println!("Connecting to address {}", address);
        let udp_socket =
            UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).expect("couldn't bind to address client");

        println!("Udp socket bound");

        udp_socket
            .connect(address)
            .expect("cannot connet to server");

        println!("Connected to server");

        udp_socket
            .send(b"ping")
            .expect("Could not send the message");

        println!("Client: Message sent");

        let mut buf = [0; 1024];

        let number_of_bytes = udp_socket.recv(&mut buf).expect("Cannot receive messages");

        println!("Client: Message received");

        let message = &buf[..number_of_bytes];
        let message_str = String::from_utf8_lossy(message);
        println!("client received {}", message_str);

        return;
    }

    let udp_socket = UdpSocket::bind("127.0.0.1:3400").expect("couldn't bind to address");

    let mut buf = [0; 1024];
    println!("Starting");

    loop {
        let (number_of_bytes, src_addr) = udp_socket.recv_from(&mut buf).expect("error here");

        let message = &buf[..number_of_bytes];

        let message_str = String::from_utf8_lossy(message);

        println!("value: {}", &message_str);

        if message_str == "ping" {
            udp_socket
                .send_to(b"pong", src_addr)
                .expect("tutto okay, non funziona");
        }
    }
}
