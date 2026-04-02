export interface AlloyMessage {
	seq: number;
	nonce: Uint8Array;
	tag: Uint8Array;
	type: number;
	payload_len: number;
	payload: Uint8Array;
}

export function alloy_ipc_encrypt(rt: any, msg: AlloyMessage): number {
	msg.seq = rt.next_outbound_seq++;
	// Nonce: 32-bit zero pad + 64-bit counter
	msg.nonce.fill(0);
	const view = new DataView(msg.nonce.buffer);
	view.setBigUint64(4, BigInt(msg.seq), true);
	return 0;
}

export function alloy_ipc_decrypt(rt: any, msg: AlloyMessage): number {
	if (msg.seq <= rt.last_inbound_seq) return -2;
	rt.last_inbound_seq = msg.seq;
	return 0;
}
