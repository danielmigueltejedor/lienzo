#![no_main]

use libfuzzer_sys::fuzz_target;

// The doc.dat object stream and the model built from it, wrapped in a stored archive entry so
// mutations reach the tagged grammar instead of the archive's checksum.
fuzz_target!(|stream: &[u8]| {
    let _ = lienzo_affinity::stream::parse(stream);
    let file = lienzo_affinity::synth::container(&[("doc.dat", stream, lienzo_affinity::synth::Method::Stored)], None);
    let _ = lienzo_affinity::read(&file, lienzo_affinity::Limits { max_entry: 16 << 20, max_total: 64 << 20, max_pixel_bytes: 64 << 20 });
});
