<div align="center">
<h3>MPEG Parser</h3>
</div>
MPEG transport stream parser written in Rust
This is strictly for educational purposes (learning rust).

About
------------

Reads a MPEG transport stream (currently from a file), parses the Packet information (PATs, PMTs, etc..)
into objects and prints out useful information about the stream

Based on MPEG-ts spec (iso/iec 13818): https://ecee.colorado.edu/~ecen5653/ecen5653/papers/iso13818-1.pdf

**Goals**:
- Practically apply rust to a real life problem
- Implement the main parts of the TS, don't bother with the details
- Create a library and a binary that calls the library
- Carefully organize the code into a way that makes sense using Rust's module, packages and crates system
- (Optionally) Implement tests to learn about Rust's testing system
