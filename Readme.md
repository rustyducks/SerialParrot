# Serial Parrot

Record data from serial port, and replay it !

A _transport_ that chunk data into packets is required.
Timestamped packets are recorded to an output file.

The replay write data to serial while trying to match the timestamps.

## Data format

Data is recorded in a binary format :
The file begin by the baudrate as an uint32, followed by data records. Data records consist of a timestamp as double (f64), the lenght of the packet as uint16, then the packet itself.

Data are recorded as little endian.

file: `[baudrate: u32][data record][data record][data record][data record]`  
data record: `[timestamp: double][data lenght: u16][data..]`

## Record data

The python script had performance issues on raspberry pi, so a Rust version has been made.

### Python version
`./parrot_listener.py -p <port> -b <baudrate> -f <file>`

Example:  
`./parrot_listener.py -p /dev/ttyUSB0 -b 230400 -f /tmp/ldata`

### Rust version

`rust_listener -p <port> -b <baudrate> -f <file> [-t <transport>]`

## Replay data

The baudrate is recorded in the data file so you don't have to specify it.

`./parrot_replay.py -p <port> -f <file>`

`./parrot_replay.py -p /tmp/sink -f /tmp/ldata`

**Tip:** You can replay data to a virtual serial port. To create a pair of virtual serial ports, run:  
`socat -d -d pty,raw,echo=0,link="/tmp/sink" pty,raw,echo=0,link="/tmp/src"`

