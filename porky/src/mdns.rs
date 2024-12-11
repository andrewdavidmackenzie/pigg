use crate::wifi;
use core::net::{Ipv4Addr, Ipv6Addr};
use defmt::info;
use edge_mdns::buf::VecBufAccess;
use edge_mdns::domain::base::Ttl;
use edge_mdns::host::{Service, ServiceAnswers};
use edge_mdns::io;
use edge_mdns::io::IPV4_DEFAULT_SOCKET;
use edge_mdns::{host::Host, HostAnswersMdnsHandler};
use edge_nal::UdpSplit;
use edge_nal_embassy::{Udp, UdpBuffers};
use embassy_net::Stack;
use embassy_rp::clocks::RoscRng;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use rand::RngCore;

/// mDNS responder embassy task
#[embassy_executor::task]
pub async fn mdns_responder(
    stack: Stack<'static>,
    ipv4: Ipv4Addr,
    port: u16,
    serial_number: &'static str,
    model: &'static str,
    service: &'static str,
    protocol: &'static str,
) {
    let udp_buffers: UdpBuffers<{ wifi::STACK_RESOURCES_SOCKET_COUNT }, 1500, 1500, 2> =
        UdpBuffers::new();
    let udp = Udp::new(stack, &udp_buffers);
    let mut socket = io::bind(&udp, IPV4_DEFAULT_SOCKET, Some(Ipv4Addr::UNSPECIFIED), None)
        .await
        .unwrap();

    let (recv, send) = socket.split();

    let signal = Signal::new();

    let (recv_buf, send_buf) = (
        VecBufAccess::<NoopRawMutex, 1500>::new(),
        VecBufAccess::<NoopRawMutex, 1500>::new(),
    );

    let mdns = io::Mdns::<NoopRawMutex, _, _, _, _>::new(
        Some(Ipv4Addr::UNSPECIFIED),
        None,
        recv,
        send,
        recv_buf,
        send_buf,
        |buf| RoscRng.fill_bytes(buf),
        &signal,
    );

    // Host we are announcing from - not sure how important this is
    let host = Host {
        hostname: "host1",
        ipv4,
        ipv6: Ipv6Addr::UNSPECIFIED,
        ttl: Ttl::CAP,
    };

    // The service we will be announcing over mDNS
    let service = Service {
        name: serial_number,
        priority: 1,
        weight: 5,
        service,
        protocol,
        port,
        service_subtypes: &[],
        txt_kvs: &[
            ("Serial", serial_number),
            ("Model", model),
            ("AppName", env!("CARGO_BIN_NAME")),
            ("AppVersion", env!("CARGO_PKG_VERSION")),
        ],
    };

    info!("Starting mDNS responder");

    let _ = mdns
        .run(HostAnswersMdnsHandler::new(ServiceAnswers::new(
            &host, &service,
        )))
        .await;

    info!("Exiting mDNS responder");
}
