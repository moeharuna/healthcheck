// ## Задача

// Необходимо разработать CLI-утилиту совершающую HTTP Healthcheck'и по заданному URL'у.

// Проверка совершается в цикле с заданным интервалом. На каждой итерации необходимо совершить HTTP GET по заданному URL'у.
// Есть три возможных результата проверки:
// 1. `OK(200)`, в случае, если запрос вернул HTTP status code `200`.
// 2. `ERR({HTTP_CODE})`, в случае, если запрос вернул HTTP status code отличный от `200`.
// 3. `URL parsing error`, в случае, если второй аргумент не является валидным HTTP URL'ом. После чего приложение завершается.

// Утилита принимает два аргумента:
// 1. Целочисленное значение интервала в секундах.
// 2. HTTP URL который будет проверяться.

// Примеры выполнения:
// ```
// ~$./my_binary 2 http://example.com/

// Checking 'http://example.com/'. Result: OK(200)
// Checking 'http://example.com/'. Result: OK(200)
// Checking 'http://example.com/'. Result: OK(200)
// ^C
// ```
// ```
// ~$./my_binary 1 http://httpstat.us/500

// Checking 'http://httpstat.us/500'. Result: ERR(500)
// Checking 'http://httpstat.us/500'. Result: ERR(500)
// ^C
// ```
// ```
// ~$./my_binary 1 http://httpstat.us/503

// Checking 'http://httpstat.us/503'. Result: ERR(503)
// Checking 'http://httpstat.us/503'. Result: ERR(503)
// ^C
// ```
// ```
// ~$./my_binary 1 this_is_not_an_url

// URL parsing error
// ```

use std::thread;
use std::time::Duration;

use clap::Parser;

use reqwest::StatusCode;
use reqwest::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg()]
    interval: u64,
    #[arg()]
    url: String,
}

#[derive(Debug, PartialEq)]
enum HttpError {
    BadCode(StatusCode),
    RequestError,
    ParseError,
}

struct HttpClient {
    client: reqwest::blocking::Client,
}

impl HttpClient {
    fn new() -> Self {
        HttpClient {
            client: reqwest::blocking::Client::new(),
        }
    }
    fn health_check(&self, url: Url) -> Result<StatusCode, HttpError> {
        let status = self
            .client
            .get(url)
            .send()
            .map_err(|_| HttpError::RequestError)?
            .status();
        if status.is_success() {
            Ok(status)
        } else {
            Err(HttpError::BadCode(status))
        }
    }
}

fn main() -> Result<(), HttpError> {
    let args = Args::parse();
    let client = HttpClient::new();

    loop {
        let url = Url::parse(args.url.as_str()).map_err(|_| HttpError::ParseError);
        let result = match url {
            Ok(url) => client.health_check(url),
            Err(error) => Err(error),
        };
        match result {
            Err(HttpError::ParseError) => {
                println!("URL {} error", args.url);
                return Err(HttpError::ParseError);
            }
            _ => println!("Checking '{}'. Result: {:?}", args.url, result),
        }
        thread::sleep(Duration::from_secs(args.interval));
    }
}

#[test]
fn example() {
    let client = HttpClient::new();
    assert_eq!(
        client.health_check(Url::parse("http://example.com").unwrap()),
        Ok(StatusCode::from_u16(200).unwrap())
    );
}

#[test]
fn http_stat() {
    let client = HttpClient::new();
    assert_eq!(
        client.health_check(Url::parse("http://httpstat.us/500").unwrap()),
        Err(HttpError::BadCode(StatusCode::from_u16(500).unwrap()))
    );
}
