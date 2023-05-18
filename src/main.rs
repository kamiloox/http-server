use std::{
  fs::File,
  io::{BufRead, BufReader, Read, Write},
  net::{TcpListener, TcpStream},
  str::from_utf8,
};

fn main() {
  let listener = TcpListener::bind("0.0.0.0:8080").expect("cannot create a server");

  for stream in listener.incoming() {
    if let Ok(stream) = stream {
      handle_connection(stream);
    }
  }
}

fn handle_connection(mut stream: TcpStream) {
  let reader = BufReader::new(&mut stream);

  let request: Vec<_> = reader
    .lines()
    .map(|line| line.unwrap())
    .take_while(|line| !line.is_empty())
    .collect();

  let asset = request.first().unwrap().split(" ").nth(1).unwrap();

  let file = read_file(&format!("static{}", asset));

  let response = match file {
    Some(file) => prepare_response(Response {
      file: Some(file),
      code: 200,
      status: String::from("OK"),
    }),
    None => prepare_response(Response {
      file: read_file("static/404.html"),
      code: 404,
      status: String::from("NOT FOUND"),
    }),
  };

  stream.write_all(response.as_bytes()).unwrap();
}

#[derive(Debug)]
struct ReadFile {
  length: usize,
  buffer: Vec<u8>,
  extension: String,
}

#[derive(Debug)]
struct Response {
  file: Option<ReadFile>,
  code: usize,
  status: String,
}

fn parse_mime(extension: &str) -> &'static str {
  return match extension {
    "html" => "text/html",
    "ico" => "image/png",
    _ => todo!(),
  };
}

fn prepare_response(response: Response) -> String {
  let mut parsed = format!(concat!("HTTP/1.1 {} {}\n"), response.code, response.status);

  if let Some(ReadFile {
    buffer,
    length,
    extension,
  }) = response.file
  {
    parsed = format!(
      concat!("{}", "Content-Type: {}\n", "Content-Length: {}\n", "\n{}"),
      parsed,
      parse_mime(&extension),
      length,
      // todo: do not parse utf_8, it might be a binary data like image
      from_utf8(&buffer).unwrap().to_string()
    );
  }

  return parsed;
}

fn read_file(path: &str) -> Option<ReadFile> {
  let file = File::open(path).ok()?;

  let mut reader = BufReader::new(file);

  let mut buffer: Vec<u8> = vec![];

  if let Ok(length) = reader.read_to_end(&mut buffer) {
    let result = ReadFile {
      length,
      buffer,
      extension: path.split(".").nth(1).unwrap().to_string(),
    };

    return Some(result);
  } else {
    return None;
  };
}
