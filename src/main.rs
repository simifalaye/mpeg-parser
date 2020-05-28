use std::{
    fs::File,
    io::prelude::*,
    collections::HashMap,
};
use mpeg_parser::*;
use mpeg_parser::psi::*;

// Usage:
// mpeg-parser <filename>
//
// Arguments:
//     - filename
//         Path to TS stream file
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

    // Move the file pointer to the sync byte
    if !packet::advance_file_to_sync_byte(&mut file) {
        eprintln!("Unable to find sync byte in file: {}", filename);
        std::process::exit(1);
    }

    let mut buffer = [0u8; packet::PACKET_SIZE * 1024];
    let mut pmt_pids: Vec<Pid> = vec![];
    let mut pat_crc = 0u32;
    let mut pmt_crcs: HashMap<u16, u32> = HashMap::new();
    println!("PSIs:");
    println!("-----");
    'file: loop {
        // Read file in chunks (more efficient to read in larger chunks)
        match file.read(&mut buffer[..]).expect("read failed") {
            // EOF; quit
            0 => break,
            _n => {
                // iterate through buffer by the packet size
                let mut itr = buffer.chunks_exact(packet::PACKET_SIZE);
                let mut pk = match itr.next() {
                    None => break 'file,
                    p => p.unwrap(),
                };

                'packet: loop {
                    // try to parse the buffer as a packet
                    if let Some(packet) = packet::Packet::new(pk, &mut pmt_pids) {
                        // if the packet is a psi packet, display it (only prints each unique psi)
                        if let Some(psi) = packet.psi {
                            match psi {
                                Psi::Pat(pat) => pat.display(&mut pat_crc),
                                Psi::Pmt(pmt) => pmt.display(&packet.pid, &mut pmt_crcs),
                            }
                        }
                        // iterate to next packet
                        pk = match itr.next() {
                            None => break 'packet,
                            p => p.unwrap(),
                        };
                    }
                }
            },
        }
    }

    // Return
    std::process::exit(0);
}
