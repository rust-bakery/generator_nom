#![feature(generators, generator_trait)]

#[macro_use] extern crate nom;
extern crate flavors;
extern crate circular;

use std::env;
use std::cmp::min;
use std::fs::File;
use std::io::Read;
use std::ops::{Generator, GeneratorState};

use flavors::parser::header;
use nom::{HexDisplay,IResult,Offset};
use circular::Buffer;

mod types;

use types::*;

fn main() {
  let mut args = env::args();
  let path = args.next().expect("first arg is program path");
  let filename = args.next().expect("please pass a file path as first argument");

  println!("filename: {}", path);
  println!("filename: {}", filename);
  run(&filename).expect("should parse file correctly");
}

fn run(filename: &str) -> std::io::Result<()> {
  let mut file = File::open(filename)?;

  // circular::Buffer is a ring buffer abstraction that separates reading and consuming data
  // it can grow its internal buffer and move data around if we reached the end of that buffer
  let mut capacity = 1000;
  let mut b = Buffer::with_capacity(capacity);

  // we write into the `&mut[u8]` returned by `space()`
  let sz = file.read(b.space()).expect("should write");
  b.fill(sz);
  println!("write {:#?}", sz);

  let length = {
    // `available_data()` returns how many bytes can be read from the buffer
    // `data()` returns a `&[u8]` of the current data
    // `to_hex(_)` is a helper method of `nom::HexDisplay` to print a hexdump of a byte slice
    println!("data({} bytes):\n{}", b.available_data(), (&b.data()[..min(b.available_data(), 128)]).to_hex(16));

    // we parse the beginning of the file with `flavors::parser::header`
    // a FLV file is made of a header, then a serie of tags, suffixed by a 4 byte integer (size of previous tag)
    // the file header is also followed by a 4 byte integer size
    let res = header(b.data());
    if let IResult::Done(remaining, h) = res {
      println!("parsed header: {:#?}", h);

      // `offset()` is a helper method of `nom::Offset` that can compare two slices and indicate
      // how far they are from each other. The parameter of `offset()` must be a subset of the
      // original slice
      b.data().offset(remaining)
    } else {
      panic!("couldn't parse header");
    }
  };

  // 4 more bytes for the size of previous tag just after the header
  println!("consumed {} bytes", length+4);
  b.consume(length+4);


  let mut generator = move || {
    // we will count the number of tag and use that and return value for the generator
    let mut tag_count = 0usize;
    let mut consumed = length;

    // this is the data reading loop. On each iteration we will read more data, then try to parse
    // it in the inner loop
    loop {
      // refill the buffer
      let sz = file.read(b.space()).expect("should write");
      b.fill(sz);
      println!("refill: {} more bytes, available data: {} bytes, consumed: {} bytes",
        sz, b.available_data(), consumed);

      // if there's no more available data in the buffer after a write, that means we reached
      // the end of the file
      if b.available_data() == 0 {
        println!("no more data to read or parse, stopping the reading loop");
        break;
      }

      // this is the parsing loop. After we read some data, we will try to parse from it until
      // we get an error or the parser returns `Incomplete`, indicating it needs more data
      loop {
        let (length,tag) = {
          //println!("[{}] data({} bytes, consumed {}):\n{}", tag_count,
          //  b.available_data(), consumed, (&b.data()[..min(b.available_data(), 128)]).to_hex(16));

          // try to parse a tag
          // the `types::flv_tag` parser combines the tag parsing and consuming the 4 byte integer size
          // following it
          match flv_tag(b.data()) {

            // `Incomplete` means the nom parser does not have enough data to decide,
            // so we wait for the next refill and then retry parsing
            IResult::Incomplete(needed) => {
              println!("not enough data, needs a refill: {:?}", needed);
              break;
            },

            // stop on an error. Maybe something else than a panic would be nice
            IResult::Error(e) => {
              panic!("parse error: {:#?}", e);
            },

            // we produced a correct tag
            IResult::Done(remaining, tag) => {
              tag_count += 1;

              // tags parsed with flavors contain a slice of the original data. We cannot
              // return that from the generator, since it is borrowed from the Buffer's internal
              // data. Instead, we use the `types::Tag` defined in `src/types.rs` to clone
              // the data
              let t = Tag::new(tag);
              (b.data().offset(remaining), t)
            },
          }
        };

        println!("consuming {} of {} bytes", length, b.available_data());
        b.consume(length);
        consumed += length;

        // give the tag to the calling code. On the next call to the generator's `resume()`,
        // we will continue from the parsing loop, and go on the reading loop's next iteration
        // if necessary
        yield tag;
      }

      // if the buffer has no more space to write too, it might be time to grow the internal buffer
      if b.available_space() == 0 {
        println!("growing buffer capacity from {} bytes to {} bytes", capacity, capacity*2);

        capacity *= 2;
        b.grow(capacity);
      }
    }

    // we finished looping over the data, return how many tag we parsed
    return tag_count;
  };

  loop {
    match generator.resume() {
      GeneratorState::Yielded(tag) => {
        println!("next tag: type={:?}, timestamp={}, size={}",
          tag.header.tag_type, tag.header.timestamp, tag.header.data_size);
      },
      GeneratorState::Complete(tag_count) => {
        println!("parsed {} FLV tags", tag_count);
        break;
      }
    }
  }

  Ok(())
}
