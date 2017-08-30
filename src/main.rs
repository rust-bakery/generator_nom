#![feature(generators, generator_trait)]

#[macro_use]
extern crate nom;
extern crate flavors;

use std::env;
use std::cmp::min;
use std::fs::File;
use std::io::{BufRead,BufReader};
use std::ops::{Generator, GeneratorState};

use flavors::parser::{header,complete_tag,tag_header,TagType};
use nom::{HexDisplay,IResult,Offset};

mod types;

use types::*;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct MyTagHeader {
  pub tag_type:  TagType,
  pub data_size: u32,
  pub timestamp: u32,
  pub stream_id: u32,
}

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
  let mut reader = BufReader::new(file);

  let length = {
    let buf = reader.fill_buf()?;
    println!("data({} bytes):\n{}", buf.len(), (&buf[..min(buf.len(), 128)]).to_hex(16));
    let res = header(buf);
    //println!("header: {:?}", res);
    if let IResult::Done(remaining, h) = res {
      println!("parsed header: {:#?}", h);
      buf.offset(remaining)
    } else {
      panic!("couldn't parse header");
    }
  };

  println!("consumed {} bytes", length);
  reader.consume(length);
  // 4 bytes for the size of previous tag
  reader.consume(4);
  //let buf = reader.fill_buf()?;
  //println!("data after consume and fill_buff ({} bytes):\n{}", buf.len(), (&buf[..min(buf.len(), 128)]).to_hex(16));

  let mut generator = move || {
    let mut tag_count = 0usize;
    let mut consumed = length + 4;
    loop {
      let (length,tag) = {
        let buf = reader.fill_buf().expect("should fill buf");
        println!("data({} bytes, consumed {}):\n{}", buf.len(), consumed, (&buf[..min(buf.len(), 128)]).to_hex(16));

        if buf.len() == 0 {
          break;
        }

        match complete_tag(buf) {
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
            /*let t = MyTagHeader {
              tag_type: tag.tag_type,
              data_size: tag.data_size,
              timestamp: tag.timestamp,
              stream_id: tag.stream_id,
            };*/
            (buf.offset(remaining), t)
          },
        }
      };

      reader.consume(length+4);
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
