# SPDX-License-Identifier: Apache-2.0
"""Finite, exact-address TCP peer for the controlled route matrix.

Run on a disposable peer with --ipv4 and --ipv6 set to that host's actual
non-loopback addresses. The script creates no files or background service.
"""

import argparse
import socket
import threading
import time


def receive_exact(connection, length):
    result = bytearray()
    while len(result) < length:
        chunk = connection.recv(length - len(result))
        if not chunk:
            raise RuntimeError(
                f"connection ended after {len(result)} of {length} expected bytes: {bytes(result)!r}"
            )
        result.extend(chunk)
    return bytes(result)


def serve(family, address, port, runs, result):
    expected = {"selected": runs, "sentinel": runs * 3}
    counts = {"selected": 0, "sentinel": 0}
    errors = []
    workers = []
    lock = threading.Lock()

    def handle(connection):
        try:
            with connection:
                connection.settimeout(5)
                first = connection.recv(16)
                if not first:
                    return
                first += receive_exact(connection, 16 - len(first))
                if first == b"sentinel-request":
                    if connection.recv(1) != b"":
                        raise RuntimeError("sentinel sent unexpected trailing bytes")
                    connection.sendall(b"sentinel-reply")
                    kind = "sentinel"
                else:
                    first += receive_exact(connection, len(b"runtime-client-request") - 16)
                    if first != b"runtime-client-request":
                        raise RuntimeError(f"unexpected request {first!r}")
                    connection.sendall(b"runtime-origin-reply")
                    finish = receive_exact(connection, len(b"runtime-client-finish"))
                    if finish != b"runtime-client-finish" or connection.recv(1) != b"":
                        raise RuntimeError("selected client did not finish the round trip")
                    kind = "selected"
                with lock:
                    counts[kind] += 1
        except Exception as error:
            with lock:
                errors.append(str(error))

    with socket.socket(family, socket.SOCK_STREAM) as listener:
        listener.bind((address, port))
        listener.listen(16)
        listener.settimeout(1)
        print(f"READY {address}:{port}", flush=True)
        deadline = time.monotonic() + 180
        while time.monotonic() < deadline:
            with lock:
                if counts == expected or errors:
                    break
            try:
                connection, _ = listener.accept()
            except socket.timeout:
                continue
            worker = threading.Thread(target=handle, args=(connection,), daemon=True)
            worker.start()
            workers.append(worker)
        for worker in workers:
            worker.join(timeout=5)
    result[address] = (counts, errors)
    print(f"DONE {address} {counts} errors={errors}", flush=True)
    if counts != expected or errors:
        raise RuntimeError(f"incomplete controlled peer {address}: {result[address]}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--ipv4", required=True)
    parser.add_argument("--ipv6", required=True)
    parser.add_argument("--ipv4-port", type=int, required=True)
    parser.add_argument("--ipv6-port", type=int, required=True)
    parser.add_argument("--runs", type=int, default=2)
    args = parser.parse_args()
    if args.runs < 1:
        parser.error("--runs must be positive")
    result = {}
    errors = []

    def run(*parameters):
        try:
            serve(*parameters)
        except Exception as error:
            errors.append(str(error))

    threads = [
        threading.Thread(
            target=run,
            args=(family, address, port, args.runs, result),
            daemon=True,
        )
        for family, address, port in [
            (socket.AF_INET, args.ipv4, args.ipv4_port),
            (socket.AF_INET6, args.ipv6, args.ipv6_port),
        ]
    ]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join(timeout=195)
    if errors or any(thread.is_alive() for thread in threads):
        raise RuntimeError(f"controlled peer failed: {errors}")


if __name__ == "__main__":
    main()
