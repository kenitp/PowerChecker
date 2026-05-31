extern crate serialport;

use serialport::SerialPort;
use std::time::Duration;
use std::error::Error;
use std::io::prelude::*;

const CMD_DELAY: u64 = 100;
const MAX_RETRY_ATTEMPTS: u32 = 3;
const RESET_DELAY: u64 = 1000;

pub fn init_serial_io(device_path: &str) -> Result<Box<dyn SerialPort>, Box<dyn Error>> {
    let port = serialport::new(device_path, 115200)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(30000))
        .open()?;
    Ok(port)
}

/// シリアルポートをリセットする関数
pub fn reset_serial_port(port: &mut Box<dyn SerialPort>) -> Result<(), Box<dyn Error>> {
    println!("INFO: Resetting serial port...");
    
    // ポートをクリア
    if let Err(e) = port.clear(serialport::ClearBuffer::All) {
        println!("WARNING: Failed to clear serial port buffers: {:?}", e);
    }
    
    // リセット待機
    std::thread::sleep(Duration::from_millis(RESET_DELAY));
    
    // ポートの状態を確認
    if let Err(e) = port.try_clone() {
        println!("ERROR: Serial port is not accessible after reset: {:?}", e);
        return Err(Box::new(e))
    }
    
    println!("INFO: Serial port reset completed successfully");
    Ok(())
}

pub fn tx_command_str(port: &mut Box<dyn SerialPort>, cmd: &str) -> Result<(), Box<dyn Error>>{
    println!("SND: {:?}", cmd);
    let str = String::from(cmd) + "\r\n";
    match port.write(str.as_bytes()) {
        Ok(_) => std::io::stdout().flush()?,
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
            println!("WARNING: Write timeout for command: {:?}", cmd);
        },
        Err(e) => {
            println!("ERROR: Write error for command {:?}: {:?}", cmd, e);
            return Err(Box::new(e))
        },
    }
    std::thread::sleep(Duration::from_millis(CMD_DELAY));
    Ok(())
}

pub fn tx_command_bytes(port: &mut Box<dyn SerialPort>, cmd: &[u8]) -> Result<(), Box<dyn Error>>{
    match port.write(cmd) {
        Ok(_) => std::io::stdout().flush()?,
        Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
            println!("WARNING: Write timeout for binary command");
        },
        Err(e) => {
            println!("ERROR: Write error for binary command: {:?}", e);
            return Err(Box::new(e))
        },
    }
    std::thread::sleep(Duration::from_millis(CMD_DELAY));
    Ok(())
}

pub fn rx_command(port: &mut Box<dyn SerialPort>) -> Result<Vec<String>, Box<dyn Error>>{
    rx_command_with_retry(port, 0)
}

/// リトライ機能付きの受信関数
fn rx_command_with_retry(port: &mut Box<dyn SerialPort>, retry_count: u32) -> Result<Vec<String>, Box<dyn Error>> {
    let mut buf: Vec<u8> = Vec::new();
    let mut read_attempts = 0;
    
    loop {
        let mut buf_tmp: Vec<u8> = vec![0; 1000];
        match port.read(buf_tmp.as_mut_slice()) {
            Ok(t) => {
                if t > 0 {
                    for index in 0..t {
                        buf.push(buf_tmp[index]);
                    }
                    
                    // CRLF終端チェック
                    if 2 <= buf.len() {
                        if (buf[buf.len()-2] == 0x0d) && (buf[buf.len()-1] ==0x0a) {
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::TimedOut => {
                        println!("ERROR: Serial port read timeout after {} attempts (buffer size: {} bytes, retry: {}/{})", 
                                read_attempts + 1, buf.len(), retry_count + 1, MAX_RETRY_ATTEMPTS);
                        
                        // リトライ可能な場合
                        if retry_count < MAX_RETRY_ATTEMPTS {
                            println!("INFO: Attempting to recover from timeout (attempt {}/{})", retry_count + 1, MAX_RETRY_ATTEMPTS);
                            
                            // シリアルポートをリセット
                            if let Err(reset_err) = reset_serial_port(port) {
                                println!("ERROR: Failed to reset serial port: {:?}", reset_err);
                                return Err(reset_err)
                            }
                            
                            // バッファが空でない場合は部分的なデータを返す
                            if buf.len() > 0 {
                                println!("WARNING: Returning partial data due to timeout");
                                return parse_received_data(&buf)
                            }
                            
                            // リトライ
                            return rx_command_with_retry(port, retry_count + 1)
                        } else {
                            // 最大リトライ回数に達した場合
                            if buf.len() == 0 {
                                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::TimedOut, 
                                    format!("Serial port read timeout after {} retries with empty buffer", MAX_RETRY_ATTEMPTS))))
                            } else {
                                println!("WARNING: Max retries reached, returning partial data");
                                return parse_received_data(&buf)
                            }
                        }
                    }
                    _ => {
                        println!("ERROR: Serial port read error: {:?}", e);
                        return Err(Box::new(e))
                    }
                }
            }
        };
        
        read_attempts += 1;
        if read_attempts > 100 {
            println!("ERROR: Too many read attempts ({}) without finding CRLF termination", read_attempts);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Too many read attempts")))
        }
    }

    if buf.len() == 0 {
        println!("WARNING: Empty buffer received from serial port");
        return Ok(Vec::new())
    }

    parse_received_data(&buf)
}

/// 受信データをパースする関数
fn parse_received_data(buf: &Vec<u8>) -> Result<Vec<String>, Box<dyn Error>> {
    let mut cmds_byte: Vec<Vec<u8>> = Vec::new();
    let mut cmds_str: Vec<String> = Vec::new();
    let bytes = &buf[..buf.len()];
    let mut start_idx = 0;
    
    for index in 1..buf.len() {  // index=1からスタート（index-1を使うため）
        if (bytes[index] == 0x0a) && (bytes[index-1] == 0x0d) {
            let cmd: Vec<u8> = bytes[start_idx..index-1].to_vec();
            if 0 < cmd.len(){
                cmds_byte.push(cmd);
            }
            start_idx = index+1;
        }
    }
    
    for (i, cmd_byte) in cmds_byte.iter().enumerate() {
        let string = get_one_line_from_cmdbytes(&cmd_byte);
        println!("RCV[{}]: {:?}", i, string);
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

/// シリアルポートの状態をチェックする関数
#[allow(dead_code)]
pub fn check_serial_port_health(port: &mut Box<dyn SerialPort>) -> Result<bool, Box<dyn Error>> {
    match port.try_clone() {
        Ok(_) => {
            println!("INFO: Serial port is healthy");
            Ok(true)
        },
        Err(e) => {
            println!("ERROR: Serial port health check failed: {:?}", e);
            Ok(false)
        }
    }
}

/// シリアルポートの接続を再確立する関数
#[allow(dead_code)]
pub fn reconnect_serial_port(device_path: &str) -> Result<Box<dyn SerialPort>, Box<dyn Error>> {
    println!("INFO: Attempting to reconnect to serial port: {}", device_path);
    
    // 既存の接続を閉じるための待機時間
    std::thread::sleep(Duration::from_millis(2000));
    
    // 新しい接続を確立
    match init_serial_io(device_path) {
        Ok(port) => {
            println!("INFO: Successfully reconnected to serial port");
            Ok(port)
        },
        Err(e) => {
            println!("ERROR: Failed to reconnect to serial port: {:?}", e);
            Err(e)
        }
    }
}
