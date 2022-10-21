extern crate regex;

use serialport::SerialPort;
use std::time::Duration;
use std::error::Error;

use regex::Regex;

use crate::bp35c2::device;

pub struct MeterInfo {
    pub channel: String,
    pub pan_id: String,
    pub meter_mac_addr: String,
    pub meter_ip6_addr: String,
    pub event20: bool,
    pub event22: bool,
}

pub const GET_POWER_W:[u8;16] = [0x10, 0x81, 0x00, 0x01, 0x05, 0xFF, 0x01, 0x02, 0x88, 0x01, 0x62, 0x01, 0xE7, 0x00, 0x0d, 0x0a];
pub const GET_POWER_A:[u8;16] = [0x10, 0x81, 0x00, 0x01, 0x05, 0xFF, 0x01, 0x02, 0x88, 0x01, 0x62, 0x01, 0xE8, 0x00, 0x0d, 0x0a];


pub(crate) fn init_bp35c2(device_path: &str, b_route_id: &str, b_route_pass: &str) -> Result<Box<dyn SerialPort>, Box<dyn Error>> {
    let mut port = device::init_serial_io(device_path)?;

    send(&mut port, "SKRESET", 1000, true).unwrap();
    let resp = send(&mut port, "SKVER", 100, true).unwrap();
    match wait_resp_ok(&resp) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    let resp = send(&mut port, "SKINFO", 100, true).unwrap();
    match wait_resp_ok(&resp) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    let resp = send(&mut port, "SKAPPVER", 100, true).unwrap();
    match wait_resp_ok(&resp) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    let resp = send(&mut port, &("SKSETRBID ".to_string() + &b_route_id.to_string()), 100, true).unwrap();
    match wait_resp_ok(&resp) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    let resp = send(&mut port, &("SKSETPWD c ".to_string() + &b_route_pass.to_string()), 100, true).unwrap();
    match wait_resp_ok(&resp) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    return Ok(port)
}

pub fn scan_meter(port: &mut Box<dyn SerialPort>) -> MeterInfo {
    let mut meter_info: MeterInfo;
    loop {
        let resp = send(port, "SKSCAN 2 FFFFFFFF 6 0", 20000, true).unwrap();
        meter_info = wait_resp_event20(&resp);
        if meter_info.event20 == true {
            break;
        }
    }
    send(port, &("SKSREG S2 ".to_string() + &meter_info.channel), 100, true).unwrap();
    send(port, &("SKSREG S3 ".to_string() + &meter_info.pan_id), 100, true).unwrap();
    let resp = send(port, &("SKLL64 ".to_string() + & meter_info.meter_mac_addr), 100, true).unwrap();
    meter_info.meter_ip6_addr = resp[1].to_string();
    send(port, &("SKJOIN ".to_string() + &meter_info.meter_ip6_addr), 300, false).unwrap();
    wait_resp_event25(port);
    MeterInfo { 
        channel: meter_info.channel,
        pan_id: meter_info.pan_id, 
        meter_mac_addr: meter_info.meter_mac_addr,
        meter_ip6_addr: meter_info.meter_ip6_addr,
        event20: meter_info.event20,
        event22: meter_info.event22, 
    }
}

pub fn connect_meter(port: &mut Box<dyn SerialPort>, meterinfo: &MeterInfo) {
    send(port, &("SKJOIN ".to_string() + &meterinfo.meter_ip6_addr), 300, false).unwrap();
    wait_resp_event25(port); 
}

pub fn read_power_w(port: &mut Box<dyn SerialPort>, meterinfo: &MeterInfo) -> Result<u32, std::io::ErrorKind>{
    let power;
    match send_echonet_udp(port, &meterinfo.meter_ip6_addr, &GET_POWER_W) {
        Ok(_) => {
            match wait_resp_erxudp_w(port, 10000) {
                Ok(v) => power = v,
                Err(_) => return Err(std::io::ErrorKind::InvalidData)
            }
        },
        Err(e) => {
            println!("err value = {}", e);
            return Err(std::io::ErrorKind::InvalidData)
        }
    }
    Ok(power)
}

pub fn read_power_a(port: &mut Box<dyn SerialPort>, meterinfo: &MeterInfo) -> Result<f64, std::io::ErrorKind>{
    let power: f64;
    match send_echonet_udp(port, &meterinfo.meter_ip6_addr, &GET_POWER_A) {
        Ok(_) => {
            match wait_resp_erxudp_a(port, 10000) {
                Ok(v) => power = v,
                Err(_) => return Err(std::io::ErrorKind::InvalidData)
            }
        },
        Err(e) => {
            println!("err value = {}", e);
            return Err(std::io::ErrorKind::InvalidData)
        }
    }
    Ok(power)
}

fn wait_resp_ok(resp: &Vec<String>) -> Result<(), std::io::ErrorKind> {

    let ret = match &*resp[resp.len() - 1] {
        "OK" => (),
        _ => return Err(std::io::ErrorKind::InvalidData),
    };
    Ok(ret)
}

fn wait_resp_event20(resp: &Vec<String>) -> MeterInfo {

    let mut event20 = false;
    let mut event22 = false;
    let mut pan_id = String::new();
    let mut channel = String::new();
    let mut meter_mac_addr = String::new();
    let meter_ip6_addr = String::new();

    for i in resp {
        if Some(0) <= i.find("EVENT 20"){
            event20 = true;
        }
        if Some(0) <= i.find("EVENT 22"){
            event22 = true;
        }
        if Some(0) <= i.find("Channel:"){
            let re = Regex::new(r"(  Channel:)([0-9]{2})").unwrap();
            let caps = re.captures(i).unwrap();
            channel = caps[2].to_string();
        }
        if Some(0) <= i.find("Pan ID:"){
            let re = Regex::new(r"(  Pan ID:)([0-9A-Z]{4})").unwrap();
            let caps = re.captures(i).unwrap();
            pan_id = caps[2].to_string();
        }
        if Some(0) <= i.find("Addr:"){
            let re = Regex::new(r"(  Addr:)([0-9A-Z]{16})").unwrap();
            let caps = re.captures(i).unwrap();
            meter_mac_addr = caps[2].to_string();
        }
    }
    MeterInfo { 
        channel: channel,
        pan_id: pan_id, 
        meter_mac_addr: meter_mac_addr,
        meter_ip6_addr: meter_ip6_addr,
        event20: event20,
        event22: event22, 
    }
}

fn wait_resp_event25(port: &mut Box<dyn SerialPort>){
    let mut event25 = false;
    loop {
        let resp = device::rx_command(port);
        let mut cmd: Vec<String> = Vec::new();
        match resp {
            Ok(v) => cmd = v,
            Err(e) => println!("ERROR: {}", e),
        }
        if 0 < cmd.len(){
            for i in cmd {
                if Some(0) <= i.find("EVENT 25"){
                    event25 = true;
                }
            }
        }
        if event25 == true {
            break;
        }

        std::thread::sleep(Duration::from_millis(2000));
    }
}

fn wait_resp_erxudp_w(port: &mut Box<dyn SerialPort>, time_ms: u64) -> Result<u32, std::io::ErrorKind> {
    let mut rcv = false;
    let mut power_w: u32 = 0;
    let mut count: u64 = 0;
    loop {
        let resp = device::rx_command(port);
        let mut cmd: Vec<String> = Vec::new();
        match resp {
            Ok(v) => cmd = v,
            Err(e) => println!("ERROR: {}", e),
        }
        let mut params: Vec<String> = Vec::new();
        if 0 < cmd.len(){
            for i in cmd {
                if Some(0) <= i.find("ERXUDP"){
                    rcv = true;

                    params = i.split(" ").fold(Vec::new(), |mut s, i| {
                        match &*i {
                            "" => (),
                            _ => s.push(i.to_string()),
                        }
                        s
                    });
                }
            }
        }
        if rcv == true {
            let str = get_usage_value_w(&(params[9]), 28, 35);
            match u32::from_str_radix(&str, 16) {
                Ok(v) => {
                    if 0 < v && v < 10000 {
                        power_w = v
                    } else {
                        return Err(std::io::ErrorKind::InvalidData);
                    }
                },
                Err(e) => println!("Error {}", e)
            };
            break;
        }
        let retry_time = 2000;
        std::thread::sleep(Duration::from_millis(retry_time));
        count = count + 1;
        if time_ms < (retry_time * count){
            break;
        }
    }
    Ok(power_w)
}

fn wait_resp_erxudp_a(port: &mut Box<dyn SerialPort>, time_ms: u64) -> Result<f64, std::io::ErrorKind> {
    let mut rcv = false;
    let mut power_a: u32 = 0;
    let mut count: u64 = 0;
    loop {
        let resp = device::rx_command(port);
        let mut cmd: Vec<String> = Vec::new();
        match resp {
            Ok(v) => cmd = v,
            Err(e) => println!("ERROR: {}", e),
        }
        let mut params: Vec<String> = Vec::new();
        if 0 < cmd.len(){
            for i in cmd {
                if Some(0) <= i.find("ERXUDP"){
                    rcv = true;

                    params = i.split(" ").fold(Vec::new(), |mut s, i| {
                        match &*i {
                            "" => (),
                            _ => s.push(i.to_string()),
                        }
                        s
                    });
                }
            }
        }
        if rcv == true {
            let str_r = get_usage_value_w(&(params[9]), 28, 31);
            let str_t = get_usage_value_w(&(params[9]), 32, 35);
            println!("POWER_A-R: {} / POWER_A-T: {}", str_r, str_t);
            match u32::from_str_radix(&str_r, 16) {
                Ok(v) => { power_a = v },
                Err(e) => println!("Error {}", e)
            };
            match u32::from_str_radix(&str_t, 16) {
                Ok(v) => { power_a = power_a + v },
                Err(e) => println!("Error {}", e)
            };
            break;
        }
        let retry_time = 2000;
        std::thread::sleep(Duration::from_millis(retry_time));
        count = count + 1;
        if time_ms < (retry_time * count){
            break;
        }
    }
    if 0 == power_a || 2000 <=  power_a {
        return Err(std::io::ErrorKind::InvalidData);
    }
    Ok(power_a as f64 / 20.0)

}

fn send(port: &mut Box<dyn SerialPort>, cmd: &str, millis: u64, resp: bool) -> Result<Vec<String>, Box<dyn Error>>{
    match device::tx_command_str(port, cmd, millis){
        Ok(())=> (),
        Err(err) => return Err(err),
    }
    if resp == true {
        return device::rx_command(port);
    } else {
        let cmds: Vec<String> = Vec::new();
        return Ok(cmds);
    }
}

fn send_echonet_udp(port: &mut Box<dyn SerialPort>, ip6addr: &str, cmd: &[u8]) -> Result<(), Box<dyn Error>>{
    let header: String = "SKSENDTO 1 ".to_string() + ip6addr + " 0E1A 1 0 000E ";
    let mut tmp_cmd_bytes : Vec<u8> = Vec::new();

    for byte in header.as_bytes(){
        tmp_cmd_bytes.push(*byte);
    }
    for byte in cmd {
        tmp_cmd_bytes.push(*byte);
    }

    let cmd_bytes : &[u8] = &tmp_cmd_bytes;
    println!("SEND(ECHONET): {}", header);
    match device::tx_command_bytes(port, cmd_bytes, 1000){
        Ok(v)=> return Ok(v),
        Err(err) => return Err(err),
    }
}

fn get_usage_value_w(param: &str, start: usize, end: usize) -> String {
    let mut s = String::new();
    for i in param.chars().enumerate() {
        match i.0 {
            n if n < start => {continue}
            n if n > end => {break}
            _ => {s.push(i.1)}
        }
    }
    s
}