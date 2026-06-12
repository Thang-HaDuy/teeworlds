#!/usr/bin/env python3
"""
Teeworlds minimal bot - connects to server, joins game, logs messages.
Implements the full 0.7 connection handshake.
"""

import socket
import random
import time
import math

# ===== HUFFMAN =====
FREQ_TABLE = [
    1<<30,4545,2657,431,1950,919,444,482,2244,617,838,542,715,1814,304,240,754,212,647,186,
    283,131,146,166,543,164,167,136,179,859,363,113,157,154,204,108,137,180,202,176,
    872,404,168,134,151,111,113,109,120,126,129,100,41,20,16,22,18,18,17,19,
    16,37,13,21,362,166,99,78,95,88,81,70,83,284,91,187,77,68,52,68,
    59,66,61,638,71,157,50,46,69,43,11,24,13,19,10,12,12,20,14,9,
    20,20,10,10,15,15,12,12,7,19,15,14,13,18,35,19,17,14,8,5,
    15,17,9,15,14,18,8,10,2173,134,157,68,188,60,170,60,194,62,175,71,
    148,67,167,78,211,67,156,69,1674,90,174,53,147,89,181,51,174,63,163,80,
    167,94,128,122,223,153,218,77,200,110,190,73,174,69,145,66,277,143,141,60,
    136,53,180,57,142,57,158,61,166,112,152,92,26,22,21,28,20,26,30,21,
    32,27,20,17,23,21,30,22,22,21,27,25,17,27,23,18,39,26,15,21,
    12,18,18,27,20,18,15,19,11,17,33,12,18,15,19,18,16,26,17,18,
    9,10,25,22,22,17,20,16,6,16,15,20,14,18,24,335,1517
]

HUFFMAN_EOF = 256
HUFFMAN_MAX_SYM = 257
HUFFMAN_MAX_NODES = HUFFMAN_MAX_SYM * 2 - 1
HUFFMAN_LUTBITS = 10
HUFFMAN_LUTSIZE = 1 << HUFFMAN_LUTBITS
HUFFMAN_LUTMASK = HUFFMAN_LUTSIZE - 1

class _HNode:
    __slots__ = ['bits','num_bits','leafs','symbol']
    def __init__(self):
        self.bits = 0; self.num_bits = 0
        self.leafs = [0xffff, 0xffff]; self.symbol = 0

class Huffman:
    def __init__(self):
        self._nodes = [_HNode() for _ in range(HUFFMAN_MAX_NODES)]
        self._lut = [None] * HUFFMAN_LUTSIZE
        self._start = 0
        self._build()

    def _setbits(self, idx, bits, depth):
        n = self._nodes[idx]
        if n.leafs[1] != 0xffff: self._setbits(n.leafs[1], bits|(1<<depth), depth+1)
        if n.leafs[0] != 0xffff: self._setbits(n.leafs[0], bits, depth+1)
        if n.num_bits:
            n.bits = bits; n.num_bits = depth

    def _build(self):
        nodes = self._nodes
        for i in range(HUFFMAN_MAX_SYM):
            nodes[i].num_bits = 0xFFFFFFFF
            nodes[i].symbol = i
            nodes[i].leafs = [0xffff, 0xffff]

        freq_list = [[1 if i == HUFFMAN_EOF else FREQ_TABLE[i], i] for i in range(HUFFMAN_MAX_SYM)]
        num = HUFFMAN_MAX_SYM

        while len(freq_list) > 1:
            freq_list.sort(key=lambda x: x[0], reverse=True)
            a_id, b_id = freq_list[-1][1], freq_list[-2][1]
            nodes[num].num_bits = 0
            nodes[num].leafs[0] = a_id
            nodes[num].leafs[1] = b_id
            freq_list[-2][0] = freq_list[-1][0] + freq_list[-2][0]
            freq_list[-2][1] = num
            freq_list.pop(); num += 1

        self._start = freq_list[0][1]
        self._setbits(self._start, 0, 0)

        for i in range(HUFFMAN_LUTSIZE):
            bits = i; node = nodes[self._start]; k = 0
            while k < HUFFMAN_LUTBITS:
                li = node.leafs[bits & 1]
                if li == 0xffff: break
                node = nodes[li]; bits >>= 1; k += 1
                if node.num_bits: self._lut[i] = node; break
            if k == HUFFMAN_LUTBITS: self._lut[i] = node

    def compress(self, data):
        nodes = self._nodes
        bits = 0; bitcount = 0; out = bytearray()
        for b in data:
            n = nodes[b]; bits |= n.bits << bitcount; bitcount += n.num_bits
            while bitcount >= 8:
                out.append(bits & 0xff); bits >>= 8; bitcount -= 8
        n = nodes[HUFFMAN_EOF]; bits |= n.bits << bitcount; bitcount += n.num_bits
        while bitcount > 0:
            out.append(bits & 0xff); bits >>= 8; bitcount -= 8
        return bytes(out)

    def decompress(self, data):
        nodes = self._nodes; lut = self._lut; eof = nodes[HUFFMAN_EOF]
        out = bytearray(); bits = 0; bitcount = 0; i = 0; src = bytes(data)
        while True:
            node = lut[bits & HUFFMAN_LUTMASK] if bitcount >= HUFFMAN_LUTBITS else None
            while bitcount < 24 and i < len(src):
                bits |= src[i] << bitcount; bitcount += 8; i += 1
            if node is None: node = lut[bits & HUFFMAN_LUTMASK]
            if node is None: break
            if node.num_bits:
                bits >>= node.num_bits; bitcount -= node.num_bits
            else:
                bits >>= HUFFMAN_LUTBITS; bitcount -= HUFFMAN_LUTBITS
                while True:
                    node = nodes[node.leafs[bits & 1]]; bitcount -= 1; bits >>= 1
                    if node.num_bits: break
                    if bitcount == 0: return bytes(out)
            if node is eof: break
            out.append(node.symbol)
        return bytes(out)

_huff = Huffman()

# ===== VARIABLE INT =====
def pack_int(v):
    out = bytearray(); b = 0
    if v < 0: b |= 0x40; v = ~v
    b |= v & 0x3F; v >>= 6
    while v:
        b |= 0x80; out.append(b); b = v & 0x7F; v >>= 7
    out.append(b); return bytes(out)

def unpack_int(data, pos=0):
    if pos >= len(data): return 0, pos
    sign = (data[pos] >> 6) & 1; val = data[pos] & 0x3F
    masks = [0x7F, 0x7F, 0x7F, 0x0F]; shifts = [6, 13, 20, 27]
    for i in range(4):
        if not (data[pos] & 0x80): break
        pos += 1
        if pos >= len(data): break
        val |= (data[pos] & masks[i]) << shifts[i]
    pos += 1
    return (~val if sign else val), pos

def pack_str(s): return s.encode() + b'\x00'

def unpack_str(data, pos=0):
    end = data.index(b'\x00', pos)
    return data[pos:end].decode(errors='replace'), end + 1

# ===== CONSTANTS =====
NET_PACKETFLAG_CONTROL    = 1
NET_PACKETFLAG_RESEND     = 2
NET_PACKETFLAG_COMPRESSION = 4
NET_PACKETFLAG_CONNLESS   = 8
NET_CHUNKFLAG_VITAL       = 1
NET_CTRLMSG_KEEPALIVE     = 0
NET_CTRLMSG_CONNECT       = 1
NET_CTRLMSG_ACCEPT        = 2
NET_CTRLMSG_CLOSE         = 4
NET_CTRLMSG_TOKEN         = 5
NET_MAX_SEQUENCE          = 1024

# Engine message IDs (system) — values from src/engine/shared/protocol.h
NETMSG_INFO            = 1
NETMSG_MAP_CHANGE      = 2
NETMSG_MAP_DATA        = 3
NETMSG_SERVERINFO      = 4
NETMSG_CON_READY       = 5
NETMSG_SNAP            = 6
NETMSG_SNAPEMPTY       = 7
NETMSG_SNAPSINGLE      = 8
NETMSG_INPUTTIMING     = 10
NETMSG_RCON_LINE       = 13
NETMSG_READY           = 18
NETMSG_ENTERGAME       = 19
NETMSG_INPUT           = 20
NETMSG_REQUEST_MAP_DATA= 23
NETMSG_PING            = 26
NETMSG_PING_REPLY      = 27

# Game message IDs (not system)
SV_MOTD            = 1
SV_TUNEPARAMS      = 6
SV_READYTOENTER    = 8
SV_SERVERSETTINGS  = 17
SV_CLIENTINFO      = 18
SV_GAMEINFO        = 19
SV_CLIENTDROP      = 20
SV_GAMEMSG         = 21
CL_STARTINFO       = 27

NET_VERSION = "0.7 802f1be60a05665f"

SYS_MSG_NAMES = {
    1:'INFO', 2:'MAP_CHANGE', 3:'MAP_DATA', 4:'SERVERINFO', 5:'CON_READY',
    6:'SNAP', 7:'SNAPEMPTY', 8:'SNAPSINGLE', 10:'INPUTTIMING', 13:'RCON_LINE',
    18:'READY', 19:'ENTERGAME', 20:'INPUT', 23:'REQUEST_MAP_DATA',
    26:'PING', 27:'PING_REPLY',
}
GAME_MSG_NAMES = {
    1:'SV_MOTD', 2:'SV_BROADCAST', 3:'SV_CHAT', 4:'SV_TEAM',
    5:'SV_KILLMSG', 6:'SV_TUNEPARAMS', 7:'SV_EXTRAPROJECTILE',
    8:'SV_READYTOENTER', 9:'SV_WEAPONPICKUP', 10:'SV_EMOTICON',
    11:'SV_VOTECLEAROPTIONS', 17:'SV_SERVERSETTINGS', 18:'SV_CLIENTINFO',
    19:'SV_GAMEINFO', 20:'SV_CLIENTDROP', 21:'SV_GAMEMSG',
    27:'CL_STARTINFO',
}


class TeeBot:
    def __init__(self, host='127.0.0.1', port=8303, name='TeeBot'):
        self.addr = (host, port)
        self.name = name
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.sock.settimeout(3.0)
        self.token_cl = random.randrange(0x100000000)
        self.token_srv = 0xFFFFFFFF
        self.seq = 0    # our outgoing vital sequence
        self.ack = 0    # last server vital seq we saw (sent back as ack)

    # --- Packet header ---
    def _pack_hdr(self, flags, num_chunks):
        t = self.token_srv
        return bytes([
            ((flags & 0x3F) << 2) | ((self.ack >> 8) & 0x03),
            self.ack & 0xFF,
            num_chunks & 0xFF,
            (t >> 24) & 0xFF, (t >> 16) & 0xFF, (t >> 8) & 0xFF, t & 0xFF
        ])

    def _unpack_hdr(self, raw):
        flags = (raw[0] >> 2) & 0x3F
        ack   = ((raw[0] & 0x03) << 8) | raw[1]
        nch   = raw[2]
        token = (raw[3] << 24) | (raw[4] << 16) | (raw[5] << 8) | raw[6]
        return flags, ack, nch, token

    # --- Chunk header ---
    def _pack_chunk(self, data, vital):
        size = len(data)
        if vital:
            self.seq = (self.seq + 1) % NET_MAX_SEQUENCE
            s = self.seq
            hdr = bytes([
                (NET_CHUNKFLAG_VITAL << 6) | ((size >> 6) & 0x3F),
                (size & 0x3F) | ((s >> 2) & 0xC0),
                s & 0xFF
            ])
        else:
            hdr = bytes([(size >> 6) & 0x3F, size & 0x3F])
        return hdr + data

    def _unpack_chunk_hdr(self, payload, off):
        flags = (payload[off] >> 6) & 0x03
        size  = ((payload[off] & 0x3F) << 6) | (payload[off+1] & 0x3F)
        if flags & NET_CHUNKFLAG_VITAL:
            seq = ((payload[off+1] & 0xC0) << 2) | payload[off+2]
            return flags, size, seq, off + 3
        return flags, size, -1, off + 2

    # --- Send helpers ---
    def _send_raw(self, data): self.sock.sendto(data, self.addr)

    def _send_ctrl(self, msg_type, extra=b''):
        self._send_raw(self._pack_hdr(NET_PACKETFLAG_CONTROL, 0) + bytes([msg_type]) + extra)

    def _send_msg(self, msg_data, vital=True):
        chunk = self._pack_chunk(msg_data, vital)
        compressed = _huff.compress(chunk)
        if len(compressed) < len(chunk):
            hdr = self._pack_hdr(NET_PACKETFLAG_COMPRESSION, 1)
            self._send_raw(hdr + compressed)
        else:
            hdr = self._pack_hdr(0, 1)
            self._send_raw(hdr + chunk)

    def _sys(self, msg_id, data=b''):
        return pack_int((msg_id << 1) | 1) + data

    def _game(self, msg_id, data=b''):
        return pack_int((msg_id << 1) | 0) + data

    # --- Receive ---
    def _recv(self, timeout=3.0):
        self.sock.settimeout(timeout)
        try:
            raw, _ = self.sock.recvfrom(4096)
        except socket.timeout:
            return None, []
        if len(raw) < 7:
            return None, []

        flags, ack, nch, token = self._unpack_hdr(raw)

        if flags & NET_PACKETFLAG_CONNLESS:
            return 'connless', []

        if flags & NET_PACKETFLAG_CONTROL:
            ctrl = raw[7] if len(raw) > 7 else -1
            rest = raw[8:] if len(raw) > 8 else b''
            return 'ctrl', [(ctrl, rest)]

        payload = raw[7:]
        if flags & NET_PACKETFLAG_COMPRESSION:
            payload = _huff.decompress(payload)

        chunks = []; off = 0
        for _ in range(nch):
            if off + 2 > len(payload): break
            ch_flags, ch_size, ch_seq, off = self._unpack_chunk_hdr(payload, off)
            if off + ch_size > len(payload): break
            ch_data = payload[off:off+ch_size]; off += ch_size
            if ch_seq >= 0:
                self.ack = ch_seq
            chunks.append((ch_flags, ch_size, ch_seq, ch_data))
        return 'data', chunks

    def _parse_msg(self, data):
        if not data: return None, None, b''
        v, pos = unpack_int(data, 0)
        return v >> 1, bool(v & 1), data[pos:]

    # --- Connection steps ---
    def step_token(self):
        print("[bot] Token exchange...")
        pad = bytes(512)
        pkt = bytearray(7 + 1 + 4 + len(pad))
        pkt[0] = (NET_PACKETFLAG_CONTROL << 2) & 0xFC
        pkt[3:7] = b'\xff\xff\xff\xff'
        pkt[7] = NET_CTRLMSG_TOKEN
        pkt[8]  = (self.token_cl >> 24) & 0xFF
        pkt[9]  = (self.token_cl >> 16) & 0xFF
        pkt[10] = (self.token_cl >>  8) & 0xFF
        pkt[11] =  self.token_cl        & 0xFF
        self._send_raw(bytes(pkt))

        raw, _ = self.sock.recvfrom(4096)
        self.token_srv = (raw[8] << 24) | (raw[9] << 16) | (raw[10] << 8) | raw[11]
        print(f"[bot]   token_srv={self.token_srv:#010x}")

    def step_connect(self):
        print("[bot] Connecting...")
        self._send_ctrl(NET_CTRLMSG_CONNECT, bytes(512))
        for _ in range(10):
            ptype, chunks = self._recv(2.0)
            if ptype == 'ctrl' and chunks and chunks[0][0] == NET_CTRLMSG_ACCEPT:
                print("[bot]   ACCEPT received")
                return
        raise RuntimeError("No ACCEPT")

    def step_info(self):
        print("[bot] Sending INFO...")
        self._send_msg(self._sys(NETMSG_INFO,
                                 pack_str(NET_VERSION) + pack_str("") + pack_int(0)))

    def step_wait_map(self):
        print("[bot] Waiting for MAP_CHANGE...")
        for _ in range(20):
            ptype, chunks = self._recv()
            if ptype == 'ctrl' and chunks and chunks[0][0] == NET_CTRLMSG_CLOSE:
                reason = chunks[0][1].rstrip(b'\x00').decode(errors='replace')
                raise RuntimeError(f"Server closed: {reason}")
            if ptype != 'data': continue
            for _, _, _, data in chunks:
                mid, sys, payload = self._parse_msg(data)
                if sys and mid == NETMSG_MAP_CHANGE:
                    name, pos = unpack_str(payload)
                    crc,  pos = unpack_int(payload, pos)
                    size, pos = unpack_int(payload, pos)
                    cpr,  pos = unpack_int(payload, pos)   # chunks_per_request
                    csz,  pos = unpack_int(payload, pos)   # chunk_size
                    self._map_size = size
                    self._map_chunk_size = csz
                    self._map_total_chunks = math.ceil(size / csz) if csz > 0 else 1
                    print(f"[bot]   map={name!r} size={size} chunk_size={csz} total_chunks={self._map_total_chunks}")
                    return
                elif sys and mid == NETMSG_PING:
                    self._send_msg(self._sys(NETMSG_PING_REPLY))
        raise RuntimeError("No MAP_CHANGE")

    def step_download_map(self):
        received = 0
        total = self._map_total_chunks
        print(f"[bot] Downloading map ({total} chunks)...")
        # Server sends all chunks in one batch (sv_map_download_speed=8 by default).
        # Send one request, then drain all MAP_DATA until done.
        # Only re-request if no data arrives for a while.
        self._send_msg(self._sys(NETMSG_REQUEST_MAP_DATA, pack_int(0)))
        timeouts = 0
        while received < total:
            ptype, chunks = self._recv(2.0)
            if ptype is None:
                timeouts += 1
                if timeouts >= 2:
                    self._send_msg(self._sys(NETMSG_REQUEST_MAP_DATA, pack_int(received)))
                    timeouts = 0
                continue
            timeouts = 0
            if ptype != 'data': continue
            for _, _, _, data in chunks:
                mid, sys, _ = self._parse_msg(data)
                if sys and mid == NETMSG_MAP_DATA:
                    received += 1
                elif sys and mid == NETMSG_PING:
                    self._send_msg(self._sys(NETMSG_PING_REPLY))
        print(f"[bot]   map download complete ({received} chunks)")

    def step_ready(self):
        print("[bot] Sending READY...")
        self._send_msg(self._sys(NETMSG_READY))

    def step_wait_con_ready(self):
        print("[bot] Waiting for CON_READY...")
        for _ in range(20):
            ptype, chunks = self._recv()
            if ptype != 'data': continue
            for _, _, _, data in chunks:
                mid, sys, _ = self._parse_msg(data)
                if sys and mid == NETMSG_CON_READY:
                    print("[bot]   CON_READY received")
                    return
                elif sys and mid == NETMSG_PING:
                    self._send_msg(self._sys(NETMSG_PING_REPLY))
        raise RuntimeError("No CON_READY")

    def step_startinfo(self):
        print("[bot] Sending STARTINFO...")
        skin_names  = b''.join(pack_str("default") for _ in range(6))
        use_colors  = b''.join(pack_int(0) for _ in range(6))
        colors      = b''.join(pack_int(0) for _ in range(6))
        payload = (pack_str(self.name) + pack_str("") + pack_int(-1)
                   + skin_names + use_colors + colors)
        self._send_msg(self._game(CL_STARTINFO, payload))

    def step_wait_ready_to_enter(self):
        print("[bot] Waiting for SV_READYTOENTER...")
        for _ in range(30):
            ptype, chunks = self._recv()
            if ptype == 'ctrl' and chunks and chunks[0][0] == NET_CTRLMSG_CLOSE:
                raise RuntimeError("Server closed during wait")
            if ptype != 'data': continue
            for _, _, _, data in chunks:
                mid, sys, _ = self._parse_msg(data)
                if not sys and mid == SV_READYTOENTER:
                    print("[bot]   SV_READYTOENTER received")
                    return
                elif sys and mid == NETMSG_PING:
                    self._send_msg(self._sys(NETMSG_PING_REPLY))
        raise RuntimeError("No SV_READYTOENTER")

    def step_entergame(self):
        print("[bot] Sending ENTERGAME...")
        self._send_msg(self._sys(NETMSG_ENTERGAME))

    # --- Input / movement ---
    def _send_input(self, direction=0, jump=0, hook=0, fire=0,
                    target_x=1, target_y=0, wanted_weapon=0):
        """Send NETMSG_INPUT as a non-vital chunk (matches client behaviour)."""
        # AckGameTick=-1 until we receive a snapshot
        ack = self._ack_game_tick
        pred = self._pred_tick

        # CNetObj_PlayerInput has 10 int fields
        input_size = 10 * 4   # 40 bytes
        data = (self._sys(NETMSG_INPUT) +
                pack_int(ack) +
                pack_int(pred) +
                pack_int(input_size) +
                pack_int(direction) +
                pack_int(target_x) +
                pack_int(target_y) +
                pack_int(jump) +
                pack_int(fire) +
                pack_int(hook) +
                pack_int(0) +          # PlayerFlags
                pack_int(wanted_weapon) +
                pack_int(0) +          # NextWeapon
                pack_int(0) +          # PrevWeapon
                pack_int(0))           # PingCorrection
        # Input is sent non-vital (no ack needed), one chunk per packet
        chunk = self._pack_chunk(data, vital=False)
        c = _huff.compress(chunk)
        if len(c) < len(chunk):
            self._send_raw(self._pack_hdr(NET_PACKETFLAG_COMPRESSION, 1) + c)
        else:
            self._send_raw(self._pack_hdr(0, 1) + chunk)

    def _parse_snap_tick(self, mid, payload):
        """Extract GameTick from SNAP / SNAPSINGLE / SNAPEMPTY."""
        pos = 0
        game_tick, pos = unpack_int(payload, pos)
        _delta_base, pos = unpack_int(payload, pos)  # GameTick - DeltaTick
        return game_tick

    # --- Main loop ---
    def run(self, duration=20):
        self._ack_game_tick = -1
        self._pred_tick     = 0

        try:
            self.step_token()
            self.step_connect()
            self.step_info()
            self.step_wait_map()
            self.step_download_map()
            self.step_ready()
            self.step_wait_con_ready()
            self.step_startinfo()
            self.step_wait_ready_to_enter()
            self.step_entergame()

            print(f"\n[bot] *** JOINED SERVER AS '{self.name}' ***")
            print(f"[bot] Starting movement test...\n")

            deadline     = time.time() + duration
            last_ka      = time.time()
            last_input   = time.time()
            snaps        = 0

            # Simple movement script: walk right 2s, jump, walk left 2s, repeat
            def _direction_at(t):
                phase = t % 8.0
                if phase < 3.0:   return  1   # walk right
                elif phase < 3.2: return  0   # pause before jump
                elif phase < 5.0: return -1   # walk left
                elif phase < 5.2: return  0
                else:             return  1

            def _jump_at(t):
                phase = t % 8.0
                return 1 if 3.0 <= phase < 3.5 else 0

            start = time.time()
            while time.time() < deadline:
                now = time.time()

                # Keepalive every 0.5s
                if now - last_ka > 0.5:
                    self._send_ctrl(NET_CTRLMSG_KEEPALIVE)
                    last_ka = now

                # Send input at ~25 Hz
                if now - last_input > 0.04:
                    elapsed  = now - start
                    direction = _direction_at(elapsed)
                    jump      = _jump_at(elapsed)
                    target_x  = direction * 100 if direction else 1
                    self._send_input(direction=direction, jump=jump,
                                     target_x=target_x, target_y=0)
                    last_input = now

                ptype, chunks = self._recv(0.02)

                if ptype == 'ctrl' and chunks and chunks[0][0] == NET_CTRLMSG_CLOSE:
                    print("[bot] Server closed connection")
                    break

                if ptype != 'data':
                    continue

                for _, _, _, data in chunks:
                    mid, sys, payload = self._parse_msg(data)
                    if not sys:
                        name = GAME_MSG_NAMES.get(mid, f'GAME_{mid}')
                        if mid == SV_MOTD:
                            msg, _ = unpack_str(payload)
                            if msg: print(f"[bot] MOTD: {msg!r}")
                        continue

                    if mid in (NETMSG_SNAP, NETMSG_SNAPSINGLE, NETMSG_SNAPEMPTY):
                        game_tick = self._parse_snap_tick(mid, payload)
                        self._ack_game_tick = game_tick
                        self._pred_tick     = game_tick + 2
                        snaps += 1
                        if snaps % 50 == 0:
                            elapsed = time.time() - start
                            print(f"[bot] t={elapsed:.1f}s  snap#{snaps}  tick={game_tick}"
                                  f"  dir={_direction_at(elapsed):+d}"
                                  f"  jump={_jump_at(elapsed)}")
                    elif mid == NETMSG_PING:
                        self._send_msg(self._sys(NETMSG_PING_REPLY))

            print(f"\n[bot] Done. Received {snaps} snapshots in {duration}s.")

        except Exception as e:
            print(f"[bot] ERROR: {e}")
            import traceback; traceback.print_exc()
        finally:
            self._send_ctrl(NET_CTRLMSG_CLOSE)
            self.sock.close()
            print("[bot] Disconnected.")


if __name__ == '__main__':
    import sys
    host = sys.argv[1] if len(sys.argv) > 1 else '127.0.0.1'
    port = int(sys.argv[2]) if len(sys.argv) > 2 else 8303
    TeeBot(host, port, name='TeeBot').run(duration=15)
