extern crate regex;

use serialport::SerialPort;
use std::time::Duration;
use std::error::Error;

use regex::Regex;

use crate::bp35c2::device;

pub struct MeterInfo {
    channel: String,
    pan_id: String,
    meter_mac_addr: String,
    meter_ip6_addr: String,
    event20: bool,
    event22: bool,
}

const GET_POWER_W:[u8;16] = [0x10, 0x81, 0x00, 0x01, 0x05, 0xFF, 0x01, 0x02, 0x88, 0x01, 0x62, 0x01, 0xE7, 0x00, 0x0d, 0x0a];
const GET_POWER_A:[u8;16] = [0x10, 0x81, 0x00, 0x01, 0x05, 0xFF, 0x01, 0x02, 0x88, 0x01, 0x62, 0x01, 0xE8, 0x00, 0x0d, 0x0a];


pub fn init_bp35c2(device_path: &str, b_route_id: &str, b_route_pass: &str) -> Result<Box<dyn SerialPort>, Box<dyn Error>> {
    let mut port = device::init_serial_io(device_path)?;

    send_str_cmd(&mut port, "SKRESET", false).unwrap();
    match wait_resp_ok(&mut port) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    send_str_cmd(&mut port, "SKVER", false).unwrap();
    match wait_resp_ok(&mut port) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    send_str_cmd(&mut port, "SKINFO", false).unwrap();
    match wait_resp_ok(&mut port) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    send_str_cmd(&mut port, "SKAPPVER", false).unwrap();
    match wait_resp_ok(&mut port) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    send_str_cmd(&mut port, &("SKSETRBID ".to_string() + &b_route_id.to_string()), false).unwrap();
    match wait_resp_ok(&mut port) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    send_str_cmd(&mut port, &("SKSETPWD c ".to_string() + &b_route_pass.to_string()), false).unwrap();
    match wait_resp_ok(&mut port) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    return Ok(port)
}

pub fn scan_meter(port: &mut Box<dyn SerialPort>) -> MeterInfo {
    let mut meter_info: MeterInfo;
    loop {
        send_str_cmd(port, "SKSCAN 2 FFFFFFFF 6 0", false).unwrap();
        meter_info = wait_resp_event20(port);
        if meter_info.event20 == true {
            break;
        }
    }
    send_str_cmd(port, &("SKSREG S2 ".to_string() + &meter_info.channel), false).unwrap();
    match wait_resp_ok(port) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    send_str_cmd(port, &("SKSREG S3 ".to_string() + &meter_info.pan_id), false).unwrap();
    match wait_resp_ok(port) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    let resp = send_str_cmd(port, &("SKLL64 ".to_string() + & meter_info.meter_mac_addr), true).unwrap();
    meter_info.meter_ip6_addr = resp[1].to_string();
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
    send_str_cmd(port, &("SKJOIN ".to_string() + &meterinfo.meter_ip6_addr), false).unwrap();
    wait_resp_event25(port); 
}

pub fn read_power_w(port: &mut Box<dyn SerialPort>, meterinfo: &MeterInfo) -> Result<u32, std::io::ErrorKind>{
    let rcv_udp;
    match send_echonet_udp(port, &meterinfo.meter_ip6_addr, &GET_POWER_W) {
        Ok(_) => {
            rcv_udp = wait_resp_erxudp(port, &GET_POWER_W, 10000)?
        },
        Err(e) => {
            println!("err value = {}", e);
            return Err(std::io::ErrorKind::InvalidData)
        }
    }
    extract_power_w_from_udp(&rcv_udp)
}

pub fn read_power_a(port: &mut Box<dyn SerialPort>, meterinfo: &MeterInfo) -> Result<f64, std::io::ErrorKind>{
    let rcv_udp;
    match send_echonet_udp(port, &meterinfo.meter_ip6_addr, &GET_POWER_A) {
        Ok(_) => {
            rcv_udp = wait_resp_erxudp(port, &GET_POWER_A, 10000)?
        },
        Err(e) => {
            println!("err value = {}", e);
            return Err(std::io::ErrorKind::InvalidData)
        }
    }
    extract_power_a_from_udp(&rcv_udp)
}

fn wait_resp_ok(port: &mut Box<dyn SerialPort>) -> Result<(), std::io::ErrorKind> {

    loop {
        let resp = device::rx_command(port);
        let mut cmd: Vec<String> = Vec::new();
        match resp {
            Ok(v) => cmd = v,
            Err(e) => println!("ERROR: {}", e),
        }
        if 0 < cmd.len(){
            for i in cmd {
                if i == "OK" {
                    return Ok(());
                }
            }
        }
    }
}

fn wait_resp_event20(port: &mut Box<dyn SerialPort>) -> MeterInfo {

    let mut event20 = false;
    let mut event22 = false;
    let mut pan_id = String::new();
    let mut channel = String::new();
    let mut meter_mac_addr = String::new();
    let meter_ip6_addr = String::new();

    loop {
        let mut resp: Vec<String> = Vec::new();
        match device::rx_command(port) {
            Ok(v) => resp = v,
            Err(e) => println!("ERROR: {}", e),
        }
        if 0 < resp.len(){
            for i in resp {
                if Some(0) <= i.find("EVENT 20"){
                    event20 = true;
                }
                if Some(0) <= i.find("EVENT 22"){
                    event22 = true;
                }
                if Some(0) <= i.find("Channel:"){
                    let re = Regex::new(r"(  Channel:)([0-9]{2})").unwrap();
                    let caps = re.captures(&i).unwrap();
                    channel = caps[2].to_string();
                }
                if Some(0) <= i.find("Pan ID:"){
                    let re = Regex::new(r"(  Pan ID:)([0-9A-Z]{4})").unwrap();
                    let caps = re.captures(&i).unwrap();
                    pan_id = caps[2].to_string();
                }
                if Some(0) <= i.find("Addr:"){
                    let re = Regex::new(r"(  Addr:)([0-9A-Z]{16})").unwrap();
                    let caps = re.captures(&i).unwrap();
                    meter_mac_addr = caps[2].to_string();
                }
            }
        }
        if event22 == true {
            break;
        }
    }
    MeterInfo { 
        channel,
        pan_id, 
        meter_mac_addr,
        meter_ip6_addr,
        event20,
        event22, 
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
    }
}

fn parse_erxudp(cmd: &Vec<String>) -> Result<Vec<String>, ()> {
    let mut params: Vec<String> = Vec::new();

    for i in cmd {
        if Some(0) <= i.find("ERXUDP"){
            params = i.split(" ").fold(Vec::new(), |mut s, i| {
                match &*i {
                    "" => (),
                    _ => s.push(i.to_string()),
                }
                s
            });
        }
    }
    if params.len() == 0{
        return Err(());
    }
    Ok(params)
}

fn wait_resp_erxudp(port: &mut Box<dyn SerialPort>, send_cmd: &[u8], time_ms: u64) -> Result<String, std::io::ErrorKind> {
    let mut count: u64 = 0;
    let rcv_udp: String;
    loop {
        let resp = device::rx_command(port);
        let cmd: Vec<String>;
        match resp {
            Ok(v) => cmd = v,
            Err(e) => {
                println!("ERROR: {}", e);
                return Err(std::io::ErrorKind::TimedOut);
            }
        }
        let mut params: Vec<String> = Vec::new();
        match parse_erxudp(&cmd){
            Ok(v) => {
                params = v;
            }
            Err(_) => ()
        }
        if 0 < params.len() {
            let rcv_cmd = u8::from_str_radix(&extract_str(&(params[9]), 24, 25), 16).unwrap_or(0);
            if rcv_cmd == send_cmd[12] {
                rcv_udp = String::from(&(params[9]));
                break;
            }
        }
        let retry_time = 2000;
        std::thread::sleep(Duration::from_millis(retry_time));
        count = count + 1;
        if time_ms < (retry_time * count){
            return Err(std::io::ErrorKind::InvalidData);
        }
    }
    Ok(rcv_udp)
}

fn extract_power_w_from_udp(udp_cmd: &str) -> Result<u32, std::io::ErrorKind> {
    let power_w;
    let str = extract_str(udp_cmd, 28, 35);
    match u32::from_str_radix(&str, 16) {
        Ok(v) => {
            if 0 < v && v < 10000 {
                power_w = v
            } else {
                return Err(std::io::ErrorKind::InvalidData);
            }
        },
        Err(_) => return Err(std::io::ErrorKind::InvalidData)
    };
    Ok(power_w)
}

fn extract_power_a_from_udp(udp_cmd: &str) -> Result<f64, std::io::ErrorKind> {
    let str_r = extract_str(udp_cmd, 28, 31);
    let str_t = extract_str(udp_cmd, 32, 35);
    println!("POWER_A-R: {} / POWER_A-T: {}", str_r, str_t);
    let mut power_a: u32 = 0;
    match u32::from_str_radix(&str_r, 16) {
        Ok(v) => { power_a = v },
        Err(e) => println!("Error {}", e)
    };
    match u32::from_str_radix(&str_t, 16) {
        Ok(v) => { power_a = power_a + v },
        Err(e) => println!("Error {}", e)
    };

    if 0 == power_a || 2000 <=  power_a {
        return Err(std::io::ErrorKind::InvalidData);
    }    Ok(power_a as f64 / 20.0)
}

fn send_str_cmd(port: &mut Box<dyn SerialPort>, cmd: &str, resp: bool) -> Result<Vec<String>, Box<dyn Error>>{
    match device::tx_command_str(port, cmd){
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
    println!("SEND(ECHONET): {}{:02X?}", header, cmd);
    match device::tx_command_bytes(port, cmd_bytes){
        Ok(v)=> return Ok(v),
        Err(err) => return Err(err),
    }
}

fn extract_str(param: &str, start: usize, end: usize) -> String {
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
