use std::{
  fs::File,
  io::{BufRead, BufReader, Read, Write},
  net::{TcpListener, TcpStream},
  path::Path,
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

  stream.write_all(&response).unwrap();
  stream.flush().unwrap();
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

fn prepare_response(response: Response) -> Vec<u8> {
  let start_line = format!(concat!("HTTP/1.1 {} {}\n"), response.code, response.status);

  if let Some(ReadFile {
    buffer,
    length,
    extension,
  }) = response.file
  {
    return format!(
      concat!("{}", "Content-Type: {}\n", "Content-Length: {}\n\n"),
      start_line,
      parse_mime(&extension),
      length,
    )
    .into_bytes()
    .into_iter()
    .chain(buffer)
    .collect::<Vec<u8>>();
  }

  return start_line.into_bytes();
}

fn read_file(path: &str) -> Option<ReadFile> {
  let path = Path::new(&path);

  let file = File::open(path).ok()?;

  let mut reader = BufReader::new(file);

  let mut buffer: Vec<u8> = vec![];

  let extension = path.extension()?.to_str()?;

  if let Ok(length) = reader.read_to_end(&mut buffer) {
    let result = ReadFile {
      length,
      buffer,
      extension: String::from(extension),
    };

    return Some(result);
  } else {
    return None;
  };
}
