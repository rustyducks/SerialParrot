#!/usr/bin/python3
import time
from threading import Thread
import struct
from serial import Serial
import sys
import argparse


class ParrotReplay:
    def __init__(self, filename, port):
        super().__init__()
        self.filename = filename
        self.port = port
        self.serial = None
        self.stopRequested = False

    def __enter__(self):
        return self

    def stop(self):
        if self.serial is not None:
            self.serial.close()

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.stop()

    def run(self):
        with open(self.filename, 'rb') as fic:
            baudrate, = struct.unpack("<I", fic.read(4))
            print(baudrate)
            self.serial = Serial(self.port, baudrate)
            start_time = time.time()
            while True:
                chunk = fic.read(10)
                if len(chunk) < 10:
                    print("end of file !")
                    break
                timestamp, data_len = struct.unpack("<dH", chunk)
                data = fic.read(data_len)
                if len(data) < data_len:
                    print("Oh no!")
                    break
                dt = timestamp - (time.time() - start_time)
                if dt > 0:
                    time.sleep(dt)
                else:
                    print(dt)
                self.serial.write(data)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-p", "--port", help="serial port to write to", required=True)
    parser.add_argument("-f", "--file", help="source file", required=True)
    args = parser.parse_args()
    with ParrotReplay(args.file, args.port) as replay:
        replay.run()
