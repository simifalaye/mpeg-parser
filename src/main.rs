use std::{
    fs::File,
    io::prelude::*,
    collections::HashMap,
    collections::HashSet,
};
use mpeg_parser::{PidState, packet::*};

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
    if !advance_file_to_sync_byte(&mut file) {
        eprintln!("Unable to find sync byte in file: {}", filename);
        std::process::exit(1);
    }

    // State variables
    let mut buffer = [0u8; PACKET_SIZE * 1024];
    let mut pmt_pids: HashSet<u16> = HashSet::new();
    let mut pid_states: HashMap<u16, PidState> = HashMap::new();

    'file: loop {
        // Read file in chunks (more efficient to read in larger chunks)
        match file.read(&mut buffer[..]).expect("read failed") {
            // EOF; quit
            0 => break,
            _n => {
                // iterate through buffer by the packet size
                let mut itr = buffer.chunks_exact(PACKET_SIZE);
                let mut pk = match itr.next() {
                    None => break 'file,
                    p => p.unwrap(),
                };

                'packet: loop {
                    if let Some(packet) = Packet::new(pk) {
                        // Update state and errors
                        packet.update_state(&mut pid_states, &mut pmt_pids);

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
    // Print pids
    PidState::display_states(&pid_states);

    // Return
    std::process::exit(0);
}
