# Password history removal review: root acceptance

Project `891e7443-8dac-4c1b-897f-9e53cb59c7ee`; 2026-10-02.

Accept source `1bfd1631c779aeb301ddf53b6581ccd8ce72befc` and its
[separate evidence](local-wave30-password-history-ci-report.md). Only the named
`history_retention_import_preview_and_concurrent_writes` body changes.

The Linux failure at main `2dea9f5df63583caee5cfa99794a5be2c796b9e4`,
run `36977687053`, was an unconfirmed password removal. The shared helper
planned then applied without the exact removal-review confirmation. The fixture
now inspects the normal public plan, proves an unconfirmed 409 leaves the entire
store unchanged, and confirms that same plan ID through the public apply service.
All original history, import, preview, rehash and concurrent-write checks remain.

Root reconstructed the whole original file by replacing just the new block with
its former helper call. Every other byte is identical; shared helpers, production
and dependencies are unchanged. Root inspected the diff and bound-plan checks,
and checked source scope, formatting, documentation and whitespace. No root Rust
execution occurred. The worker reproduced the exact failure, then passed the
exact corrected filter once locally on macOS/redb. No full target, Linux success
or overall green CI is inferred. Publication triggers the next CI run.

W02 and W05 remain done; this support fix does not change task acceptance.
