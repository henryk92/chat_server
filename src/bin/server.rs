use std::{collections::HashMap, fs, io::{Read, Write}, net::{SocketAddr, TcpListener, TcpStream}};
use chat_server::ThreadPool;
use urlencoding::decode;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(12);

    loop {
        let (stream, address) = listener.accept().unwrap();
        pool.execute(move || http_handle_connection(stream, address));
    }
}

// POST 요청의 body 추출 함수
fn request_body(buffer: &[u8]) -> String {
    // 버퍼의 내용을 utf8 -> string으로 변환
    let request = String::from_utf8_lossy(buffer);
    // \r\n\r\n 기준으로 헤더와 바디가 나뉘므로 2개로 나눠서 바디부분만 추출
    request.split("\r\n\r\n").nth(1).unwrap_or("").to_string()
}

// HTML 형식 파싱 함수
fn parse_html(body: &str) -> HashMap<String, String> {
    // & 기준으로 잘라서 key=value 형식으로 pair에 저장
    body.split('&').filter_map(|pair|{
        // = 기준으로 key, value 분할
        let mut split = pair.splitn(2, '=');
        let key = split.next()?;
        let value = split.next()?;
        // Some((key, value)) 값으로 변환
        Some((
            // key 값을 &str -> String 타입으로 변환
            key.to_string(),
            // URL 인코딩 값을 실제 문자로 변환 후, Cow<str>값에서 소유권을 가져와서 String으로 변환
            decode(value).unwrap_or_else(|_| value.into()).into_owned(),
        ))
    })
    // 모든 튜플값을 HashMap 타입으로 변환
    .collect()
}

// http 연결 함수
fn http_handle_connection(mut stream: TcpStream, mut address: SocketAddr) {
    println!("- Connected from {}", address);
    
    //0으로 채운 1024바이트 버퍼 생성
    let mut buffer = [0; 1024];
    //TcpStream에서 데이터를 읽어와 버퍼에 저장
    stream.read(&mut buffer).unwrap();
    
    // 헤더 정보 저장공간 확보 (배열길이:64 / EMPTY_HEADER로 초기화)
    let mut headers = [httparse::EMPTY_HEADER; 64];
    // httparse::Request 구조체 생성 헤더를 채워넣을 배열 전달
    let mut req = httparse::Request::new(&mut headers);

    // 버퍼 데이터를 파싱해서 req에 저장 (반환값 필요없음)
    let _ = req.parse(&buffer).unwrap();
    // 파싱한 데이터에서 HTTP 메서드 가져오기
    let method = req.method.unwrap_or("");
    // 파싱한 데이터에서 경로 가져오기
    let path = req.path.unwrap_or("");
    // 파싱한 결과 출력
    println!("Request: {} {}", method, path);

    // HTTP 라우팅 정보
    let (status_line, contents) = match (method, path) {
        // 메인 화면
        ("GET", "/") => (
            "HTTP/1.1 200 OK",
            fs::read_to_string("main.html").unwrap_or("<h1>No Main page</h1>".to_string()),
        ),
        // 회원가입 화면
        ("GET", "/signup") => (
            "HTTP/1.1 200 OK",
            fs::read_to_string("signup.html").unwrap_or("<h1>No Signup page</h1>".to_string()),
        ),
        // 회원가입 정보 수신
        ("POST", "/signup") => {
            // POST 바디 부분만 추출
            let body = request_body(&buffer);
            // 추출한 내용 파싱
            let parse = parse_html(&body);

            let response_body = format!(
                "<h1>회원가입 완료!</h1>\
                <p>{}</p>\
                <a href=\"/\"><button>메인화면으로</button></a>",
                body
            );
            ("HTTP/1.1 200 OK", response_body)
        }
        // 그 외
        _ => (
            "HTTP/1.1 404 NOT FOUND",
            fs::read_to_string("404.html").unwrap_or("<h1>404 Not Found</h1>".to_string()),
        ),
    };
    // HTTP 응답값 생성
    let response = format!(
        "{}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );
    // 생성된 응답값을 바이트로 변환하여 TCP 스트림에 전송
    stream.write_all(response.as_bytes()).unwrap();
    // 남아있는 데이터까지 강제로 플러시
    stream.flush().unwrap();
}

