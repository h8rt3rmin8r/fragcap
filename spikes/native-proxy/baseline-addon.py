# SPDX-License-Identifier: Apache-2.0

import hashlib
import json
import os


OUTPUT = os.environ["FRAGCAP_SPIKE_EVENTS"]


def emit(kind, flow, content=b"", direction=None):
    message = flow.request if kind == "request" else flow.response
    entry = {
        "kind": kind,
        "method": getattr(flow.request, "method", None),
        "path": getattr(flow.request, "path", None),
        "http_version": getattr(message, "http_version", None),
        "byte_length": len(content),
        "digest": hashlib.sha256(content).hexdigest(),
        "direction": direction,
    }
    with open(OUTPUT, "a", encoding="utf-8", newline="\n") as handle:
        handle.write(json.dumps(entry, sort_keys=True) + "\n")


def request(flow):
    emit("request", flow, flow.request.raw_content or b"")


def response(flow):
    emit("response", flow, flow.response.raw_content or b"")


def websocket_message(flow):
    message = flow.websocket.messages[-1]
    content = message.content
    if isinstance(content, str):
        content = content.encode("utf-8")
    direction = "client-to-server" if message.from_client else "server-to-client"
    emit("websocket-message", flow, content, direction)
