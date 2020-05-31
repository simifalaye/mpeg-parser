use std::collections::HashMap;

pub mod mpeg32_crc;
pub mod packet;
pub mod psi;

#[derive(Copy, Clone, Debug, Default)]
pub struct PidErrors {
    pub cc_errors: u32,
    pub crc_errors: u32,
}

#[derive(Debug, Default)]
pub struct PidState {
    pub count: u32,
    pub duplicate_count: u32,
    pub prev_packet: packet::Packet,
    pub errors: PidErrors,
}

impl PidState {
    /// Print out all the Pids and there states
    pub fn display_states(states: &HashMap<u16, PidState>) {
        let total_count: u32 = states.iter().map(|(_,s)| s.count).sum();
        println!("");
        println!("PIDs:");
        println!("-----");
        for (pid, state) in states.iter() {
            println!("[PID] {}, Count: {} ({:2.2}%)", pid, state.count,
                (state.count as f64 / total_count as f64) * 100f64);
            println!("\t=> Continuity errors: {}, Crc errors: {}",
                state.errors.cc_errors, state.errors.crc_errors);
        }
    }
}
