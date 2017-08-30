#![feature(generators, generator_trait)]

#[macro_use]
extern crate nom;
extern crate flavors;
extern crate circular;

use std::env;
use std::cmp::min;
use std::fs::File;
use std::io::{Read,Write};
use std::ops::{Generator, GeneratorState};

use flavors::parser::{header,complete_tag,tag_header,TagType};
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
  let mut capacity = 1000;
  let mut b = Buffer::with_capacity(capacity);
  let sz = file.read(b.space()).expect("should write");
  b.fill(sz);
  println!("write {:#?}", sz);

  let length = {
    println!("data({} bytes):\n{}", b.available_data(), (&b.data()[..min(b.available_data(), 128)]).to_hex(16));
    let res = header(b.data());
    //println!("header: {:?}", res);
    if let IResult::Done(remaining, h) = res {
      println!("parsed header: {:#?}", h);
      b.data().offset(remaining)
    } else {
      panic!("couldn't parse header");
    }
  };

  println!("consumed {} bytes", length);
  b.consume(length);
  // 4 bytes for the size of previous tag
  b.consume(4);

  //let buf = reader.fill_buf()?;
  //println!("data after consume and fill_buff ({} bytes):\n{}", buf.len(), (&buf[..min(buf.len(), 128)]).to_hex(16));

  let mut generator = move || {
    let mut tag_count = 0usize;
    let mut consumed = length + 4;
    loop {
      if b.available_space() == 0 {
        println!("growing buffer capacity from {} bytes to {} bytes", capacity, capacity*2);

        capacity *= 2;
        b.grow(capacity);
      }

      let sz = file.read(b.space()).expect("should write");
      b.fill(sz);

      let (length,tag) = {
        println!("[{}] data({} bytes, consumed {}):\n{}", tag_count,
          b.available_data(), consumed, (&b.data()[..min(b.available_data(), 128)]).to_hex(16));

        if b.available_data() == 0 {
          break;
        }

        match complete_tag(b.data()) {
          IResult::Incomplete(needed) => {
            println!("not enough data, needs a refill: {:#?}", needed);
            continue;
          },
          IResult::Error(e) => {
            panic!("parse error: {:#?}", e);
          },
          IResult::Done(remaining, tag) => {
            tag_count += 1;
            let t = Tag::new(tag);
            (b.data().offset(remaining), t)
          },
        }
      };

      b.consume(length+4);
      consumed += length+4;
      yield tag;
    }

    return tag_count;
  };

  loop {
    match generator.resume() {
      GeneratorState::Yielded(tag) => {
        println!("next tag: {:?}", tag);
      },
      GeneratorState::Complete(tag_count) => {
        println!("parsed {} FLV tags", tag_count);
        break;
      }
    }
  }

  Ok(())
}
