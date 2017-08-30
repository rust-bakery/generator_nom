# example code for using nom parsers with the new Rust generator feature

[generators are now available in Rust nightly.](https://internals.rust-lang.org/t/help-test-async-await-generators-coroutines/5835)

They provide a good way to wrap a nom parser and its backing file or socket,
to produce parsed data.
