#!/usr/bin/python3
import time
import struct
from serial import Serial
from dataclasses import dataclass
from typing import List
from threading import Thread
from transport import AbstractTransport, LD06Transport
import argparse


@dataclass
class Packet:
    timestamp: float
    data: bytes


class ParrotListener(Thread):
    def __init__(self, port, baudrate, filename, transport: AbstractTransport):
        super().__init__()
        self.port = port
        self.baudrate = baudrate
        self.filename = filename
        self.transport = transport
        self.serial = Serial(port, baudrate, timeout=0.01)
        self.packets = []   # type: List[Packet]
        self.stopRequested = False

    def __enter__(self):
        self.start()
        return self

    def stop(self):
        self.stopRequested = True
        print("stopping...")
        self.join()
        self.serial.close()

    def save(self):
        with open(self.filename, 'wb') as fic:
            baud = struct.pack("<I", self.baudrate)
            fic.write(baud)
            for p in self.packets:
                header = struct.pack("<dH", p.timestamp, len(p.data))
                fic.write(header)
                fic.write(p.data)

    def run(self):
        start_time = time.time()
        while not self.stopRequested:
            b = self.serial.read()
            if b != b'':
                buffer = self.transport.put(b)
                if buffer is not None:
                    dt = time.time() - start_time
                    packet = Packet(dt, buffer)
                    self.packets.append(packet)

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()
        self.save()


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-p", "--port", help="serial port to listen to", required=True)
    parser.add_argument("-b", "--baudrate", type=int, required=True)
    parser.add_argument("-f", "--file", help="output file", required=True)
    parser.add_argument("-t", "--transport", help="LD06, XV11", default="LD06")
    args = parser.parse_args()

    if args.transport == "LD06":
        trans = LD06Transport()
    else:
        raise Exception(f"transport {args.transport} unknown!")

    with ParrotListener(args.port, args.baudrate, args.file, trans) as listener:
        while True:
            time.sleep(1)
            if len(listener.packets) > 0:
                size = len(listener.packets)*len(listener.packets[0].data)
                print(f"{size/1000}K")
