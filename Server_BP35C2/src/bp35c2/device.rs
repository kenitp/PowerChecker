extern crate serialport;

use serialport::SerialPort;
use std::time::Duration;
use std::error::Error;
use std::io::prelude::*;

const CMD_DELAY: u64 = 100;

pub fn init_serial_io(device_path: &str) -> Result<Box<dyn SerialPort>, Box<dyn Error>> {
    let port = serialport::new(device_path, 115200)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(30000))
        .open()?;
    return Ok(port)
}

pub fn tx_command_str(port: &mut Box<dyn SerialPort>, cmd: &str) -> Result<(), Box<dyn Error>>{
    println!("SND: {:?}", cmd);
    let str = String::from(cmd) + "\r\n";
    match port.write(str.as_bytes()) {
        Ok(_) => std::io::stdout().flush()?,
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    std::thread::sleep(Duration::from_millis(CMD_DELAY));
    Ok(())
}

pub fn tx_command_bytes(port: &mut Box<dyn SerialPort>, cmd: &[u8]) -> Result<(), Box<dyn Error>>{
    match port.write(cmd) {
        Ok(_) => std::io::stdout().flush()?,
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => (),
        Err(e) => eprintln!("{:?}", e),
    }
    std::thread::sleep(Duration::from_millis(CMD_DELAY));
    Ok(())
}

pub fn rx_command(port: &mut Box<dyn SerialPort>) -> Result<Vec<String>, Box<dyn Error>>{
    let mut buf: Vec<u8> = Vec::new();
    loop {
        let mut buf_tmp: Vec<u8> = vec![0; 1000];
        match port.read(buf_tmp.as_mut_slice()) {
            Ok(t) => {
                for index in 0..t {
                    buf.push(buf_tmp[index]);
                }
                if (buf[buf.len()-2] == 0x0d) && (buf[buf.len()-1] ==0x0a) {
                    break;
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
                return Ok(Vec::new());
            }
        };
    }

    let mut cmds_byte: Vec<Vec<u8>> = Vec::new();
    let mut cmds_str: Vec<String> = Vec::new();
    let bytes = &buf[..buf.len()];
    let mut start_idx = 0;
    for index in 0..buf.len() {
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
        println!("RCV: {:?}", string);
        cmds_str.push(string);
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
    string
}
