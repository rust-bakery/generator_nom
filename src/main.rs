#[macro_use]
extern crate nom;
extern crate flavors;

use std::env;
use std::cmp::min;
use std::fs::File;
use std::io::{BufRead,BufReader,Write};
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
  let buf = reader.fill_buf()?;
  println!("data after consume and fill_buff ({} bytes):\n{}", buf.len(), (&buf[..min(buf.len(), 128)]).to_hex(16));
  Ok(())
}
