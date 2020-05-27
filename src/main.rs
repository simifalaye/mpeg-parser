use std::{
    fs::File,
    io::prelude::*,
};
use mpeg_parser::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = if args.len() == 2 { &args[1] } else { "media/fifth-new.ts" };
    let mut file = match File::open(filename) {
        Err(e) => {
            println!("File error: {}", e);
            std::process::exit(1);
        },
        Ok(f) => f,
    };

    // Find the sync byte
    if !packet::advance_file_to_sync_byte(&mut file) {
        eprintln!("Unable to find sync byte in file: {}", filename);
        std::process::exit(1);
    }

    // Read file in chunks
    let mut buffer = [0u8; packet::PACKET_SIZE * 1024];
    let mut pids: Vec<Pid> = vec![];
    'file: loop {
        match file.read(&mut buffer[..]).expect("read failed") {
            0 => break,
            _n => {
                // let mut last_crc = 0u32;
                let mut itr = buffer.chunks_exact(packet::PACKET_SIZE);
                let mut pk = match itr.next() {
                    None => break 'file,
                    p => p.unwrap(),
                };

                'packet: loop {
                    if let Some(packet) = packet::Packet::new(pk) {
                        if packet.is_pat() && packet.psi.is_some() {
                            println!("{}", packet);
                        }
                        // display_pat(&pk, &mut last_crc);
                        let pid = Pid::get_packet_pid(&pk);
                        match pids.binary_search_by(|x| x.value.cmp(&pid)) {
                            Ok(pos) => pids[pos].count += 1,
                            Err(pos) => pids.insert(pos, Pid { value: pid, count: 1}),
                        }

                        pk = match itr.next() {
                            None => break 'packet,
                            p => p.unwrap(),
                        };
                    }
                }
            },
        }
    }
    Pid::print_pids(&mut pids);

    // Return
    std::process::exit(0);
}
