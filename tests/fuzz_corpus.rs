#[cfg(feature = "fuzzing")]
#[test]
fn retained_parser_corpus_does_not_panic() {
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/parsers");
    for path in std::fs::read_dir(corpus).unwrap() {
        let data = std::fs::read(path.unwrap().path()).unwrap();
        riauth::fuzzing::parsers(&data);
        for end in 0..data.len() {
            riauth::fuzzing::parsers(&data[..end]);
        }
    }
    let mut state = 1u64;
    for size in [0, 1, 4, 19, 20, 253, 4096, 65536, 65537] {
        let bytes = (0..size)
            .map(|_| {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                (state >> 32) as u8
            })
            .collect::<Vec<_>>();
        riauth::fuzzing::parsers(&bytes);
    }
}
