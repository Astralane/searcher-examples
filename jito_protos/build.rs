use tonic_prost_build::configure;

fn main() {
    configure()
        // Generate Packet.data as `bytes::Bytes` so callers holding `Bytes`
        // (e.g. send_bundle_bytes_no_wait) can move it in without re-allocating.
        .bytes(".packet.Packet.data")
        .compile_protos(
            &[
                "protos/auth.proto",
                "protos/block.proto",
                "protos/block_engine.proto",
                "protos/bundle.proto",
                "protos/packet.proto",
                "protos/relayer.proto",
                "protos/searcher.proto",
                "protos/shared.proto",
            ],
            &["protos"],
        )
        .unwrap();
}
