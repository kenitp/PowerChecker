extern crate alloc;

use alloc::{boxed::Box, format, string::String, string::ToString};
use core::sync::atomic::Ordering;

use esp_hal::time::Instant;
use esp_wifi::wifi::{WifiController, WifiDevice};
use log::{debug, info, warn};
use smoltcp::{
    iface::{Config as IfaceCfg, Interface, SocketHandle, SocketSet, SocketStorage},
    socket::{
        dhcpv4,
        tcp::{Socket as TcpSocket, SocketBuffer as TcpBuf},
        udp::{PacketBuffer as UdpBuf, PacketMetadata, Socket as UdpSocket},
    },
    time::Instant as SmoltcpInstant,
    wire::{EthernetAddress, IpCidr, IpEndpoint, Ipv4Address},
};

use crate::{
    WIFI_CONNECTED, WIFI_PASS, WIFI_SSID,
    POWER_CHECKER_URL, NTP_HOST_IP, NTP_PORT, JST_OFFSET_SECS,
};

#[derive(Debug)]
pub enum NetworkError {
    Config,
    Connect,
    Send,
    Timeout,
    NoData,
    Parse,
    Dns,
    BadUrl,
}

pub struct NetworkState {
    iface: Interface,
    sockets: SocketSet<'static>,
    dhcp_handle: SocketHandle,
    tcp_handle: SocketHandle,
    udp_handle: SocketHandle,
    device: WifiDevice<'static>,
    _ap_device: WifiDevice<'static>,
    controller: WifiController<'static>,
    pub ip_assigned: bool,
    local_ip: Ipv4Address,
}

impl NetworkState {
    fn st(ms: u64) -> SmoltcpInstant {
        SmoltcpInstant::from_millis(ms as i64)
    }

    pub fn connect(
        mut device: WifiDevice<'static>,
        ap: WifiDevice<'static>,
        mut controller: WifiController<'static>,
    ) -> Result<Self, NetworkError> {
        use embedded_hal::delay::DelayNs;
        use esp_wifi::wifi::{AuthMethod, ClientConfiguration, Configuration};

        let ssid = String::from(WIFI_SSID);
        let password = String::from(WIFI_PASS);
        info!("[WiFi] Connecting to SSID: \"{}\"", WIFI_SSID);
        if ssid.len() > 32 || password.len() > 64 {
            warn!("[WiFi] SSID or password too long");
            return Err(NetworkError::Config);
        }

        controller
            .set_configuration(&Configuration::Client(ClientConfiguration {
                ssid,
                password,
                auth_method: AuthMethod::WPAWPA2Personal,
                ..Default::default()
            }))
            .map_err(|e| {
                warn!("[WiFi] set_configuration: {:?}", e);
                NetworkError::Config
            })?;
        controller.start().map_err(|e| {
            warn!("[WiFi] controller.start: {:?}", e);
            NetworkError::Connect
        })?;
        info!("[WiFi] Controller started");

        let mut delay = esp_hal::delay::Delay::new();
        let mut attempt: u32 = 0;
        const WIFI_ASSOC_TIMEOUT_MS: u64 = 8_000;
        const WIFI_RETRY_DELAY_MS: u32 = 1_000;
        loop {
            attempt += 1;
            info!("[WiFi] Connect attempt #{}", attempt);
            if let Err(e) = controller.connect() {
                warn!("[WiFi] connect() error: {:?}", e);
                delay.delay_ms(WIFI_RETRY_DELAY_MS);
                continue;
            }

            let deadline =
                Instant::now().duration_since_epoch().as_millis() + WIFI_ASSOC_TIMEOUT_MS;
            let mut connected = false;
            loop {
                match controller.is_connected() {
                    Ok(true) => {
                        connected = true;
                        break;
                    }
                    Ok(false) | Err(_) => {
                        delay.delay_ms(100u32);
                    }
                }
                if Instant::now().duration_since_epoch().as_millis() > deadline {
                    warn!("[WiFi] Attempt #{} timed out", attempt);
                    break;
                }
            }
            if connected {
                break;
            }
            let _ = controller.disconnect();
            delay.delay_ms(WIFI_RETRY_DELAY_MS);
        }
        WIFI_CONNECTED.store(true, Ordering::Relaxed);
        info!("[WiFi] Associated to \"{}\"", WIFI_SSID);

        let mac = device.mac_address();
        info!("[WiFi] MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
        let cfg = IfaceCfg::new(EthernetAddress(mac).into());
        let now = Instant::now().duration_since_epoch().as_millis();
        let mut iface = Interface::new(cfg, &mut device, Self::st(now));
        iface.update_ip_addrs(|a| {
            a.push(IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0)).ok();
        });

        let tcp_rx: &'static mut [u8] = Box::leak(Box::new([0u8; 4096]));
        let tcp_tx: &'static mut [u8] = Box::leak(Box::new([0u8; 512]));
        let udp_rx_data: &'static mut [u8] = Box::leak(Box::new([0u8; 256]));
        let udp_tx_data: &'static mut [u8] = Box::leak(Box::new([0u8; 256]));
        let udp_rx_meta: &'static mut [PacketMetadata; 4] =
            Box::leak(Box::new([PacketMetadata::EMPTY; 4]));
        let udp_tx_meta: &'static mut [PacketMetadata; 4] =
            Box::leak(Box::new([PacketMetadata::EMPTY; 4]));

        let socket_storage: &'static mut [SocketStorage<'static>; 3] =
            Box::leak(Box::new([SocketStorage::EMPTY; 3]));
        let mut sockets = SocketSet::new(socket_storage.as_mut_slice());

        let dhcp_handle = sockets.add(dhcpv4::Socket::new());
        let tcp_handle = sockets.add(TcpSocket::new(TcpBuf::new(tcp_rx), TcpBuf::new(tcp_tx)));
        let udp_handle = sockets.add(UdpSocket::new(
            UdpBuf::new(udp_rx_meta.as_mut_slice(), udp_rx_data),
            UdpBuf::new(udp_tx_meta.as_mut_slice(), udp_tx_data),
        ));
        info!("[Net] Socket set created (dhcp/tcp/udp)");

        let mut ns = NetworkState {
            iface,
            sockets,
            dhcp_handle,
            tcp_handle,
            udp_handle,
            device,
            _ap_device: ap,
            controller,
            ip_assigned: false,
            local_ip: Ipv4Address::UNSPECIFIED,
        };

        info!("[DHCP] Starting discovery...");
        ns.poll(Instant::now().duration_since_epoch().as_millis());

        Ok(ns)
    }

    pub fn poll(&mut self, now_ms: u64) {
        let t = Self::st(now_ms);
        self.iface.poll(t, &mut self.device, &mut self.sockets);

        if !self.ip_assigned {
            let ev = self.sockets.get_mut::<dhcpv4::Socket>(self.dhcp_handle).poll();
            if let Some(dhcpv4::Event::Configured(cfg)) = ev {
                self.iface.update_ip_addrs(|addrs| {
                    addrs.clear();
                    addrs
                        .push(IpCidr::new(
                            cfg.address.address().into(),
                            cfg.address.prefix_len(),
                        ))
                        .ok();
                });
                if let Some(gw) = cfg.router {
                    info!("[DHCP] Gateway: {}", gw);
                    self.iface.routes_mut().add_default_ipv4_route(gw).ok();
                }
                self.local_ip = cfg.address.address();
                self.ip_assigned = true;
                WIFI_CONNECTED.store(true, Ordering::Relaxed);
                info!("[DHCP] Configured: {}/{}", cfg.address.address(), cfg.address.prefix_len());
            }
        }
    }

    pub fn try_reconnect(&mut self) {
        let connected = matches!(self.controller.is_connected(), Ok(true));
        if connected {
            return;
        }

        if WIFI_CONNECTED.load(Ordering::Relaxed) {
            warn!("[WiFi] Disconnected — attempting reconnect");
        }
        WIFI_CONNECTED.store(false, Ordering::Relaxed);
        self.ip_assigned = false;
        self.local_ip = Ipv4Address::UNSPECIFIED;

        let dhcp = self.sockets.get_mut::<dhcpv4::Socket>(self.dhcp_handle);
        dhcp.reset();

        self.iface.update_ip_addrs(|addrs| {
            addrs.clear();
            addrs
                .push(IpCidr::new(Ipv4Address::UNSPECIFIED.into(), 0))
                .ok();
        });

        match self.controller.connect() {
            Ok(()) => info!("[WiFi] Reconnect initiated"),
            Err(e) => warn!("[WiFi] Reconnect failed: {:?}", e),
        }
    }

    pub fn http_get_power(&mut self, now_ms: u64) -> Result<(u32, u32), NetworkError> {
        let (host, port, path) = parse_url(POWER_CHECKER_URL)?;
        info!("[HTTP] GET http://{}:{}{}", host, port, path);

        // ── Ensure the TCP socket is fully closed before connecting ──────────
        {
            let s = self.sockets.get_mut::<TcpSocket>(self.tcp_handle);
            if s.is_open() {
                debug!("[HTTP] Aborting existing TCP connection (state: {:?})", s.state());
                s.abort();
            }
        }
        // Poll once so smoltcp flushes the RST packet
        self.poll(Instant::now().duration_since_epoch().as_millis());

        let server_ip: Ipv4Address = host.parse().map_err(|_| {
            warn!("[HTTP] Failed to parse IP: {}", host);
            NetworkError::Dns
        })?;
        let remote = IpEndpoint::new(server_ip.into(), port);

        {
            let local_port = 49152u16 + (now_ms % 16384) as u16;
            info!("[HTTP] local_ip={}, remote={}, local_port={}", self.local_ip, remote, local_port);
            let cx = self.iface.context();
            let s = self.sockets.get_mut::<TcpSocket>(self.tcp_handle);
            s.connect(cx, remote, (self.local_ip, local_port))
                .map_err(|e| {
                    warn!("[HTTP] TCP connect error: {:?}", e);
                    NetworkError::Connect
                })?;
        }

        let deadline = now_ms + 5_000;
        {
            use embedded_hal::delay::DelayNs;
            let mut delay = esp_hal::delay::Delay::new();
            loop {
                let cur = Instant::now().duration_since_epoch().as_millis();
                self.poll(cur);
                if self.sockets.get::<TcpSocket>(self.tcp_handle).may_send() {
                    info!("[HTTP] TCP connected");
                    break;
                }
                if cur > deadline {
                    warn!("[HTTP] TCP connect timeout (5 s)");
                    return Err(NetworkError::Timeout);
                }
                delay.delay_ms(5u32);
            }
        }

        let req = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nAccept: application/json\r\n\r\n",
            path, host
        );
        debug!("[HTTP] Request:\n{}", req);
        {
            let s = self.sockets.get_mut::<TcpSocket>(self.tcp_handle);
            s.send_slice(req.as_bytes()).map_err(|e| {
                warn!("[HTTP] send_slice error: {:?}", e);
                NetworkError::Send
            })?;
        }

        let mut body_buf = [0u8; 512];
        let mut body_len = 0usize;
        let recv_deadline = Instant::now().duration_since_epoch().as_millis() + 4_000;

        {
            use embedded_hal::delay::DelayNs;
            let mut delay = esp_hal::delay::Delay::new();
            loop {
                let cur = Instant::now().duration_since_epoch().as_millis();
                self.poll(cur);
                {
                    let s = self.sockets.get_mut::<TcpSocket>(self.tcp_handle);
                    if s.recv_queue() > 0 {
                        let n = s.recv_slice(&mut body_buf[body_len..]).unwrap_or(0);
                        if n > 0 {
                            debug!("[HTTP] Received {} bytes (total {})", n, body_len + n);
                            body_len += n;
                        }
                        if body_len >= body_buf.len() {
                            break;
                        }
                    } else if !s.may_recv() {
                        debug!("[HTTP] Connection closed by server");
                        break;
                    }
                }
                if cur > recv_deadline {
                    warn!("[HTTP] Receive timeout (4 s), got {} bytes", body_len);
                    break;
                }
                delay.delay_ms(5u32);
            }
        }

        {
            let s = self.sockets.get_mut::<TcpSocket>(self.tcp_handle);
            s.close();
        }

        if body_len == 0 {
            warn!("[HTTP] No data received");
            return Err(NetworkError::NoData);
        }

        let resp = core::str::from_utf8(&body_buf[..body_len]).map_err(|_| {
            warn!("[HTTP] Response is not valid UTF-8");
            NetworkError::Parse
        })?;
        info!("[HTTP] Response ({} bytes): {:.200}", body_len, resp);

        // Find the JSON object — search for '{' to skip HTTP headers and any
        // chunk-size lines (handles both regular and chunked transfer encoding).
        let json_start = resp.find('{').ok_or_else(|| {
            warn!("[HTTP] No JSON object '{{' found in response");
            NetworkError::Parse
        })?;
        let json_end = resp[json_start..]
            .rfind('}')
            .map(|i| json_start + i + 1)
            .unwrap_or(body_len);
        let json_slice = &resp[json_start..json_end];
        debug!("[HTTP] JSON slice: {}", json_slice);

        parse_power_json(json_slice.as_bytes())
    }

    pub fn sntp_sync(&mut self, now_ms: u64) -> Result<i32, NetworkError> {
        use embedded_hal::delay::DelayNs;
        info!("[NTP] Sync start, server {}:{}", NTP_HOST_IP, NTP_PORT);

        let remote = IpEndpoint::new(NTP_HOST_IP.into(), NTP_PORT);
        let local = IpEndpoint::new(self.local_ip.into(), 12345u16);

        // Ensure the UDP socket is closed from a previous call, then rebind.
        {
            let s = self.sockets.get_mut::<UdpSocket>(self.udp_handle);
            if s.is_open() {
                debug!("[NTP] Closing existing UDP socket");
                s.close();
            }
            s.bind(local).map_err(|e| {
                warn!("[NTP] UDP bind error: {:?}", e);
                NetworkError::Connect
            })?;
            info!("[NTP] UDP bound to port 12345");
        }

        // Need a poll so the bind is registered.
        self.poll(Instant::now().duration_since_epoch().as_millis());

        let mut pkt = [0u8; 48];
        pkt[0] = 0b00_100_011; // LI=0, VN=4, Mode=3 (client)
        {
            let s = self.sockets.get_mut::<UdpSocket>(self.udp_handle);
            s.send_slice(&pkt, remote).map_err(|e| {
                warn!("[NTP] UDP send error: {:?}", e);
                NetworkError::Send
            })?;
            info!("[NTP] Request sent to {}", remote);
        }

        // Poll to actually transmit the packet.
        self.poll(Instant::now().duration_since_epoch().as_millis());

        let deadline = now_ms + 5_000;
        let mut resp = [0u8; 48];
        let mut got = false;
        let mut delay = esp_hal::delay::Delay::new();
        let mut poll_count = 0u32;

        loop {
            let cur = Instant::now().duration_since_epoch().as_millis();
            self.poll(cur);
            poll_count += 1;
            {
                let s = self.sockets.get_mut::<UdpSocket>(self.udp_handle);
                match s.recv_slice(&mut resp) {
                    Ok((len, src)) => {
                        info!("[NTP] Received {} bytes from {}", len, src);
                        if len >= 48 {
                            got = true;
                            break;
                        }
                        warn!("[NTP] Packet too short: {} bytes", len);
                    }
                    Err(_) => {} // no data yet
                }
            }
            if cur > deadline {
                warn!("[NTP] Timeout after {} polls ({} ms)", poll_count, 5000);
                break;
            }
            delay.delay_ms(20u32);
        }

        {
            let s = self.sockets.get_mut::<UdpSocket>(self.udp_handle);
            s.close();
        }

        if !got {
            return Err(NetworkError::Timeout);
        }

        // Bytes [40..44] = Transmit Timestamp (seconds since NTP epoch 1900-01-01)
        let ntp_s = u32::from_be_bytes([resp[40], resp[41], resp[42], resp[43]]);
        let unix_s = ntp_s.saturating_sub(2_208_988_800) as i32;
        let jst_s = unix_s + JST_OFFSET_SECS;
        info!("[NTP] ntp_sec={} unix_sec={} jst_sec={}", ntp_s, unix_s, jst_s);
        Ok(jst_s)
    }
}

// ── URL parsing ───────────────────────────────────────────────────────────────

fn parse_url(url: &str) -> Result<(&str, u16, &'static str), NetworkError> {
    let rest = url.strip_prefix("http://").ok_or_else(|| {
        warn!("[URL] Not an http:// URL: {}", url);
        NetworkError::BadUrl
    })?;
    let slash = rest.find('/').unwrap_or(rest.len());
    let authority = &rest[..slash];
    let path_str = if slash < rest.len() {
        alloc::format!("/{}", &rest[slash + 1..])
    } else {
        "/".to_string()
    };
    let path: &'static str = Box::leak(path_str.into_boxed_str());

    let (host, port) = if let Some((h, p)) = authority.split_once(':') {
        (h, p.parse::<u16>().map_err(|_| NetworkError::BadUrl)?)
    } else {
        (authority, 80u16)
    };
    Ok((host, port, path))
}

// ── JSON parsing ──────────────────────────────────────────────────────────────
//
// The server may return either numeric or string JSON values:
//   {"power_w": 1500, "power_a": 6.50}   ← numbers (most common)
//   {"power_w": "1500", "power_a": "6.50"} ← strings (ArduinoJson default)
// We try numeric first, then fall back to string.

fn parse_power_json(json: &[u8]) -> Result<(u32, u32), NetworkError> {
    // ── Attempt 1: numeric fields ────────────────────────────────────────────
    #[derive(serde::Deserialize)]
    struct RespNum {
        power_w: f32,
        power_a: f32,
    }
    if let Ok((r, _)) = serde_json_core::from_slice::<RespNum>(json) {
        let watts = r.power_w as u32;
        let centi = (r.power_a * 100.0 + 0.5) as u32;
        info!("[JSON] Parsed (numeric) power_w={} power_a={:.2}", watts, r.power_a);
        return Ok((watts, centi));
    }

    // ── Attempt 2: string fields ─────────────────────────────────────────────
    #[derive(serde::Deserialize)]
    struct RespStr<'a> {
        power_w: &'a str,
        power_a: &'a str,
    }
    match serde_json_core::from_slice::<RespStr>(json) {
        Ok((r, _)) => {
            let watts: u32 = r.power_w.trim().parse().map_err(|_| {
                warn!("[JSON] Cannot parse power_w as u32: \"{}\"", r.power_w);
                NetworkError::Parse
            })?;
            let centi = parse_decimal2(r.power_a.trim())?;
            info!("[JSON] Parsed (string) power_w={} power_a_centi={}", watts, centi);
            Ok((watts, centi))
        }
        Err(e) => {
            warn!("[JSON] Both numeric and string parse failed: {:?}", e);
            Err(NetworkError::Parse)
        }
    }
}

/// Parse "3.24" → 324,  "5" → 500.
fn parse_decimal2(s: &str) -> Result<u32, NetworkError> {
    if let Some((int_s, frac_s)) = s.split_once('.') {
        let iv: u32 = int_s.parse().map_err(|_| NetworkError::Parse)?;
        let fv: u32 = match frac_s.len() {
            0 => 0,
            1 => frac_s.parse::<u32>().map_err(|_| NetworkError::Parse)? * 10,
            _ => frac_s[..2].parse().map_err(|_| NetworkError::Parse)?,
        };
        Ok(iv * 100 + fv)
    } else {
        let iv: u32 = s.parse().map_err(|_| NetworkError::Parse)?;
        Ok(iv * 100)
    }
}
