use std::{
  fs::File,
  io::{BufRead, BufReader, Error, ErrorKind, Read, Result, Write},
  net::{TcpListener, TcpStream},
  path::Path,
};

fn main() {
  let listener = TcpListener::bind("0.0.0.0:8080").expect("cannot create a server");

  for stream in listener.incoming() {
    if let Ok(mut stream) = stream {
      let response_buffer = handle_connection(&stream);

      if let Ok(buffer) = response_buffer {
        stream.write_all(&buffer).unwrap();
        stream.flush().unwrap();
      }
    }
  }
}

fn handle_connection(stream: &TcpStream) -> Result<Vec<u8>> {
  let request = get_request(&stream);

  let Ok(request) = request else {
    return Err(Error::new(ErrorKind::Other, "error"));
  };

  let file = read_file(&format!("static{}", request.path));

  return Ok(match file {
    Some(file) => prepare_response_buffer(Response {
      file: Some(file),
      code: 200,
      status: String::from("OK"),
    }),
    None => prepare_response_buffer(Response {
      file: read_file("static/404.html"),
      code: 404,
      status: String::from("NOT FOUND"),
    }),
  });
}

struct Request {
  path: String,
}

fn get_request(mut stream: &TcpStream) -> Result<Request> {
  let reader = BufReader::new(&mut stream);

  let request: Result<Vec<_>> = reader
    .lines()
    .map(|line| line.map_err(|e| Error::new(ErrorKind::Other, e)))
    .take_while(|line| match line {
      Ok(line) => !line.is_empty(),
      Err(_) => true,
    })
    .collect();

  let request = request?;
  let path = request
    .first()
    .ok_or(Error::new(ErrorKind::Other, "error"))?
    .split(" ")
    .nth(1)
    .ok_or(Error::new(ErrorKind::Other, "error"))?;

  return Ok(Request {
    path: path.to_string(),
  });
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
    "png" => "image/png",
    _ => todo!(),
  };
}

fn prepare_response_buffer(response: Response) -> Vec<u8> {
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
