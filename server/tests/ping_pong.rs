use protocol::{ ServerPacket, network::{ setup, utils } };
use renet::DefaultChannel;
use server::server::Server;
use std::{
    net::UdpSocket,
    thread,
    time::{ Duration, Instant },
};
 
fn free_addr() -> String {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.local_addr().unwrap().to_string()
}
 
/// `Server::run` loops forever, so run it on its own thread with its own
/// runtime. The thread is killed when the test process exits.
fn spawn_server(addr: String) {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            Server::new(addr).run().await.unwrap();
        });
    });
 
    // await UDP setup
    thread::sleep(Duration::from_millis(200));
}
 
#[test]
fn client_connects_and_receives_pong() {
    let addr = free_addr();
    spawn_server(addr.clone());
 
    let (mut client, mut transport) = setup::create_client(addr).unwrap();
 
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut last = Instant::now();
    let mut connected = false;
 
    while Instant::now() < deadline {
        let now = Instant::now();
        let delta = now - last;
        last = now;
 
        client.update(delta);
        transport.update(delta, &mut client).unwrap();
 
        if client.is_connected() {
            connected = true;
 
            while let Some(bytes) = client.receive_message(DefaultChannel::ReliableOrdered) {
                let packet = utils::deserialize::<ServerPacket>(&bytes).unwrap();
                if matches!(packet, ServerPacket::Pong) {
                    return; // success
                }
            }
        }
 
        transport.send_packets(&mut client).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
 
    assert!(connected, "client never connected to the server");
    panic!("client connected, but the server never sent a Pong");
}

