extern crate regex;
extern crate serialport;

use std::time::Duration;
use std::io::prelude::*;
use std::error::Error;
use serialport::SerialPort;

use regex::Regex;

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


pub fn wait_resp_command_ok(resp: &Vec<String>) -> Result<(), std::io::ErrorKind> {

    let ret = match &*resp[resp.len() - 1] {
        "OK" => (),
        _ => return Err(std::io::ErrorKind::InvalidData),
    };
    Ok(ret)
}

pub fn wait_resp_event20(resp: &Vec<String>, meter_info: &mut MeterInfo) -> MeterInfo {

    let mut event20 = meter_info.event20;
    let mut event22 = meter_info.event22;
    let mut pan_id = String::from(&meter_info.pan_id);
    let mut channel = String::from(&meter_info.channel);
    let mut meter_mac_addr = String::from(&meter_info.meter_mac_addr);
    let meter_ip6_addr = String::from(&meter_info.meter_ip6_addr);

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

pub fn wait_resp_event25(port: &mut Box<dyn SerialPort>){
    let mut event25 = false;
    loop {
        let resp = rx_command(port);
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

pub fn wait_resp_erxudp_w(port: &mut Box<dyn SerialPort>, time_ms: u64) -> Result<u32, std::io::ErrorKind> {
    let mut rcv = false;
    let mut power_w: u32 = 0;
    let mut count: u64 = 0;
    loop {
        let resp = rx_command(port);
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

pub fn wait_resp_erxudp_a(port: &mut Box<dyn SerialPort>, time_ms: u64) -> Result<f64, std::io::ErrorKind> {
    let mut rcv = false;
    let mut power_a: u32 = 0;
    let mut count: u64 = 0;
    loop {
        let resp = rx_command(port);
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

fn tx_command_str(port: &mut Box<dyn SerialPort>, cmd: &str, millis: u64) -> Result<(), Box<dyn Error>>{
    println!("SND: {:?}", cmd);
    let str = String::from(cmd) + "\r\n";
    match port.write(str.as_bytes()) {
        Ok(_) => std::io::stdout().flush()?,
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    std::thread::sleep(Duration::from_millis(millis));
    Ok(())
}

fn tx_command_bytes(port: &mut Box<dyn SerialPort>, cmd: &[u8], millis: u64) -> Result<(), Box<dyn Error>>{
    match port.write(cmd) {
        Ok(_) => std::io::stdout().flush()?,
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    std::thread::sleep(Duration::from_millis(millis));
    Ok(())
}

fn rx_command(port: &mut Box<dyn SerialPort>) -> Result<Vec<String>, Box<dyn Error>>{
    let mut buf: Vec<u8> = vec![0; 1000];
    let mut cmds_byte: Vec<Vec<u8>> = Vec::new();
    let mut cmds_str: Vec<String> = Vec::new();
    match port.read(buf.as_mut_slice()) {
        Ok(t) => {
            let bytes = &buf[..t];
            let mut start_idx = 0;
            for index in 0..t {
                if (bytes[index] == 0x0a) && (bytes[index-1] == 0x0d) {
                    let cmd: Vec<u8> = bytes[start_idx..index-1].to_vec();
                    if 0 < cmd.len(){
                        cmds_byte.push(cmd);
                    }
                    start_idx = index+1;
                }
            }
            for cmd_byte in cmds_byte {
                let string = get_one_line_from_cmdbytes(&cmd_byte);
                cmds_str.push(string);
            }
        }
        Err(e) => eprintln!("{:?}", e),
    }
    Ok(cmds_str)
}

fn get_one_line_from_cmdbytes(cmd: &Vec<u8>) -> String {

    let slice: &[u8] = cmd;
    let mut len: usize = 0;
    for i in 0..cmd.len() {
        match String::from_utf8(slice[0..(cmd.len()-i)].to_vec()) {
            Ok(_) => {
                len = cmd.len() - i;
                break;
            },
            Err(_) => {
            },
        }
    }
    let string = String::from_utf8(slice[0..(len)].to_vec()).unwrap();
    println!("RCV: {:?}", string);
    string
}

pub fn init_serial_io(device_path: &str) -> Result<Box<dyn SerialPort>, Box<dyn Error>> {
    let port = serialport::new(device_path, 115200)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(100))
        .open()?;
    return Ok(port)
}

pub fn send_command(port: &mut Box<dyn SerialPort>, cmd: &str, millis: u64, resp: bool) -> Result<Vec<String>, Box<dyn Error>>{
    match tx_command_str(port, cmd, millis){
        Ok(())=> (),
        Err(err) => return Err(err),
    }
    if resp == true {
        return rx_command(port);
    } else {
        let cmds: Vec<String> = Vec::new();
        return Ok(cmds);
    }
}

pub fn send_echonet_udp(serial: &mut Box<dyn SerialPort>, ip6addr: &str, cmd: &[u8]) -> Result<(), Box<dyn Error>>{
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
    match tx_command_bytes(serial, cmd_bytes, 1000){
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