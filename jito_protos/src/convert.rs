use std::{
    cmp::min,
    net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use bincode::serialize;
use solana_perf::packet::{Packet, PACKET_DATA_SIZE};
use solana_packet::{Meta, PacketFlags};
use solana_sdk::transaction::VersionedTransaction;

use crate::{
    packet::{
        Meta as ProtoMeta, Packet as ProtoPacket, PacketBatch as ProtoPacketBatch,
    },
    shared::Socket,
};

/// converts from a protobuf packet to packet
pub fn proto_packet_to_packet(p: &ProtoPacket) -> Packet {
    let mut data = [0u8; PACKET_DATA_SIZE];
    let copy_len = min(data.len(), p.data.len());
    data[..copy_len].copy_from_slice(&p.data[..copy_len]);
    let mut packet = Packet::new(data, Meta::default());
    if let Some(meta) = &p.meta {
        packet.meta_mut().size = meta.size as usize;
        packet.meta_mut().addr = meta
            .addr
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        packet.meta_mut().port = meta.port as u16;
        if let Some(flags) = &meta.flags {
            if flags.simple_vote_tx {
                packet.meta_mut().flags.insert(PacketFlags::SIMPLE_VOTE_TX);
            }
            if flags.forwarded {
                packet.meta_mut().flags.insert(PacketFlags::FORWARDED);
            }
            if flags.tracer_packet {
                packet.meta_mut().flags.insert(PacketFlags::PERF_TRACK_PACKET);
            }
            if flags.repair {
                packet.meta_mut().flags.insert(PacketFlags::REPAIR);
            }
            if flags.discard {
                packet.meta_mut().flags.insert(PacketFlags::DISCARD);
            }
        }
    }
    packet
}

pub fn proto_packet_batch_to_packets(
    packet_batch: ProtoPacketBatch,
) -> impl Iterator<Item = Packet> {
    packet_batch
        .packets
        .into_iter()
        .map(|proto_packet| proto_packet_to_packet(&proto_packet))
}

/// Converts a protobuf packet to a VersionedTransaction
pub fn versioned_tx_from_packet(p: &ProtoPacket) -> Option<VersionedTransaction> {
    let mut data = [0; PACKET_DATA_SIZE];
    let copy_len = min(data.len(), p.data.len());
    data[..copy_len].copy_from_slice(&p.data[..copy_len]);
    let mut packet = Packet::new(data, Default::default());
    if let Some(meta) = &p.meta {
        packet.meta_mut().size = meta.size as usize;
    }
    packet.deserialize_slice(..).ok()
}

/// Coverts a VersionedTransaction to packet
pub fn packet_from_versioned_tx(tx: VersionedTransaction) -> Packet {
    let tx_data = serialize(&tx).expect("serializes");
    let mut data = [0; PACKET_DATA_SIZE];
    let copy_len = min(tx_data.len(), data.len());
    data[..copy_len].copy_from_slice(&tx_data[..copy_len]);
    let mut packet = Packet::new(data, Default::default());
    packet.meta_mut().size = copy_len;
    packet
}

/// Converts a VersionedTransaction to a protobuf packet
pub fn proto_packet_from_versioned_tx(tx: &VersionedTransaction) -> ProtoPacket {
    let data = serialize(tx).expect("serializes");
    let size = data.len() as u64;
    ProtoPacket {
        data: data.into(),
        meta: Some(ProtoMeta {
            size,
            addr: "".to_string(),
            port: 0,
            flags: None,
            sender_stake: 0,
        }),
    }
}

pub fn proto_packet_from_tx_bytes(data: bytes::Bytes) -> ProtoPacket {
    let size = data.len() as u64;
    ProtoPacket {
        data,
        meta: Some(ProtoMeta {
            size,
            addr: "".to_string(),
            port: 0,
            flags: None,
            sender_stake: 0,
        })
    }
}

/// Converts a GRPC Socket to stdlib SocketAddr
impl TryFrom<&Socket> for SocketAddr {
    type Error = AddrParseError;

    fn try_from(value: &Socket) -> Result<Self, Self::Error> {
        IpAddr::from_str(&value.ip).map(|ip| SocketAddr::new(ip, value.port as u16))
    }
}

#[cfg(test)]
mod tests {
    use solana_sdk::{
        hash::Hash,
        message::Message,
        signature::{Keypair, Signer},
        transaction::{Transaction, VersionedTransaction},
    };

    use crate::convert::{proto_packet_from_versioned_tx, versioned_tx_from_packet};

    // Build the transaction from solana-sdk's own types rather than
    // `solana_perf::test_tx`, which pulls an older `solana-transaction` version
    // whose `Transaction` doesn't convert into solana-sdk's `VersionedTransaction`.
    fn test_tx() -> VersionedTransaction {
        let keypair = Keypair::new();
        let message = Message::new(&[], Some(&keypair.pubkey()));
        VersionedTransaction::from(Transaction::new(&[&keypair], message, Hash::default()))
    }

    #[test]
    fn test_proto_to_packet() {
        let tx_before = test_tx();
        let tx_after = versioned_tx_from_packet(&proto_packet_from_versioned_tx(&tx_before))
            .expect("tx_after");

        assert_eq!(tx_before, tx_after);
    }
}
