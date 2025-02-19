use cyw43::{Control, JoinOptions, NetDriver};
use embassy_executor::Spawner;
use embassy_net::{
    udp::{PacketMetadata, UdpSocket},
    Ipv4Address, StackResources,
};
use embassy_rp::clocks::RoscRng;
use log::info;
use rand::RngCore;
use static_cell::StaticCell;

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

pub async fn setup_network(
    spawner: &Spawner,
    net_device: NetDriver<'static>,
    control: &mut Control<'static>,
    local_ip: Option<Ipv4Address>,
    local_port: u16,
) -> UdpSocket<'static> {
    let mut rng = RoscRng;

    // OPTIONAL: speed up connecting to the network once you know your ip address (via DHCP) by putting your address in LOCAL_IP.txt
    let config = match local_ip {
        Some(address) => embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: embassy_net::Ipv4Cidr::new(address, 24),
            dns_servers: heapless::Vec::new(),
            gateway: None,
        }),
        None => embassy_net::Config::dhcpv4(Default::default()),
    };

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );

    spawner.spawn(net_task(runner)).unwrap();

    // make sure these files exist in your `src` folder
    let wifi_ssid: &str = include_str!("./WIFI_SSID.txt");
    let wifi_password: &str = include_str!("./WIFI_PASSWORD.txt");

    info!("connecting to wifi network '{}'", wifi_ssid);

    loop {
        let options = JoinOptions::new(wifi_password.as_bytes());
        match control.join(wifi_ssid, options).await {
            Ok(_) => {
                info!("connected to wifi network");
                break;
            }
            Err(err) => {
                info!("join failed with status={}, retrying...", err.status);
            }
        }
    }

    info!("waiting for ip config");
    stack.wait_config_up().await;
    info!("config up with {:?}", stack.config_v4());

    static RX_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();
    static TX_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();
    static RX_META: StaticCell<[PacketMetadata; 16]> = StaticCell::new();
    static TX_META: StaticCell<[PacketMetadata; 16]> = StaticCell::new();
    let rx_buffer = RX_BUFFER.init([0u8; 4096]);
    let tx_buffer = TX_BUFFER.init([0u8; 4096]);
    let rx_meta = RX_META.init([PacketMetadata::EMPTY; 16]);
    let tx_meta = TX_META.init([PacketMetadata::EMPTY; 16]);

    let mut socket = UdpSocket::new(stack, rx_meta, rx_buffer, tx_meta, tx_buffer);
    socket.bind(local_port).unwrap();

    socket
}
