#[cfg(feature = "fuzzing")]
#[test]
fn retained_parser_corpus_does_not_panic() {
    let corpus = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus/parsers");
    let mut paths: Vec<_> = std::fs::read_dir(corpus)
        .unwrap()
        .map(|path| path.unwrap().path())
        .collect();
    paths.sort();
    assert!(
        paths.len() <= 64,
        "Review the bounded smoke budget when expanding the corpus"
    );
    let mut failures = Vec::new();
    let mut calls = 0;
    let mut replay = |name: &str, data: &[u8]| {
        calls += 1;
        if std::panic::catch_unwind(|| riauth::fuzzing::parsers(data)).is_err() {
            failures.push(name.to_owned());
        }
    };
    for path in paths {
        let data = std::fs::read(&path).unwrap();
        assert!(
            data.len() <= 65_536,
            "Corpus input exceeds campaign budget: {}",
            path.display()
        );
        let name = path.file_name().unwrap().to_str().unwrap();
        replay(name, &data);
        // Exhaust small prefixes and sample longer truncations. Do not make
        // CI work quadratic in the size of a newly retained input.
        for end in 0..data.len().min(256) {
            replay(name, &data[..end]);
        }
        for end in [
            512,
            1024,
            4096,
            32_768,
            49_152,
            65_536,
            data.len().saturating_sub(1),
        ] {
            if end >= 256 && end < data.len() {
                replay(name, &data[..end]);
            }
        }
        if !data.is_empty() {
            for index in [0, data.len() / 2, data.len() - 1] {
                for mask in [1, 0x20, 0x80] {
                    let mut changed = data.clone();
                    changed[index] ^= mask;
                    replay(name, &changed);
                }
            }
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
        replay("generated-boundary", &bytes);
    }
    eprintln!(
        "Bounded parser smoke: {calls} calls; {} panics",
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "Parser panics retained for triage: {failures:?}"
    );
}
