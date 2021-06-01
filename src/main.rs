use libmodbus_rs::{Modbus, ModbusServer, ModbusRTU, ModbusMapping};
use clap::{App, Arg};
use configparser::ini::Ini;
use std::process::Command;
// use std::io::{self, Write};

const YOUR_DEVICE_ID: u8 = 1;

fn main() {
    let matches = App::new("simple-client")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Simple Modbus Server")
        .author("Erik Kallen")
        .arg(
            Arg::new("serial_interface")
                .about("Serialport")
                .long("serial_interface")
                .short('s')
                .takes_value(true)
                .required(false),
        )
        .get_matches();
    
    let mut config_file = Ini::new();

    // You can easily load a file to get a clone of the map:
    let config = config_file.load("config.ini").unwrap();
    println!("{:?}", config);

    let serial_interface = matches
        .value_of("serial_interface")
        .unwrap_or(config.get("serial","port").unwrap());

    let mut modbus = Modbus::new_rtu(&serial_interface, 115200, 'N', 8, 1).unwrap();
    modbus.set_slave(YOUR_DEVICE_ID).unwrap();
    modbus.set_debug(true).unwrap();
    modbus.connect().unwrap();

    let modbus_mapping = ModbusMapping::new(500, 500, 500, 500).unwrap();
    let mut query = vec![0; Modbus::RTU_MAX_ADU_LENGTH as usize];

    let regs = modbus_mapping.get_registers_mut();

    for (key, value) in &config["regs"] {
        println!("{} / {:?}", key, value);
        let key = key.parse::<usize>().unwrap();
        let value = value.as_ref().unwrap();

        if value.starts_with("!") {
            let cmd = &value[1..];

            let mut cmd = cmd.split_whitespace();
            let args = cmd.next().unwrap_or("");

            print!("Exec: {:?}", args);
            let output = Command::new(args).args(cmd).output().unwrap();

            let value = String::from_utf8(output.stdout).unwrap();
            print!("Output: {}", value);
            let value: u16 = value.trim().parse::<u16>().unwrap();
            regs[key] = value;
        } else {
            let value: u16 = value.parse::<u16>().unwrap();
            regs[key] = value;
        }
    }
    

    loop {
        match modbus.receive(&mut query) {
            Ok(num) => modbus.reply(&query, num, &modbus_mapping),
            Err(err) => {
                println!("ERROR while parsing: {}", err);
                break;
            }
        }
        .expect("could not receive message");
    }

}
