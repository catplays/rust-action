use anyhow::{anyhow, Result};
use clap::Parser;
use reqwest::{header, Client, Response, Url};
use std::{collections::HashMap, str::FromStr};
use colored::Colorize;
use mime::Mime;

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "Tyr ")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommend,
}

#[derive(Parser, Debug)]
enum SubCommend {
    Get(Get),
    Post(Post),
}

#[derive(Parser, Debug)]
struct Get {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
}

#[derive(Parser, Debug)]
struct Post {
    /// HTTP 请求的 URL
    #[clap(parse(try_from_str = parse_url))]
    url: String,
    // 请求的url
    #[clap(parse(try_from_str = parse_kv_pair))]
    body: Vec<KvPair>,// http请求的body
}

#[allow(dead_code)]
#[derive(Debug)]
struct KvPair {
    k: String,
    v: String,
}

/// 当我们实现 FromStr trait 后，可以用 str.parse() 方法将字符串解析成 KvPair
impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('=');
        let err = || anyhow!(format!("Failed to parse s:{}", s));
        Ok(Self {
            // 从迭代器中取第一个结果作为 key，迭代器返回 Some(T)/None
            // 我们将其转换成 Ok(T)/Err(E)，然后用 ? 处理错误
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_url(url: &str) -> Result<String> {
    let _url: Url = url.parse()?;
    Ok(url.into())
}


fn parse_kv_pair(s: &str) -> Result<KvPair> {
    s.parse()
}

#[tokio::main]
pub(crate) async fn go_cli() -> Result<()> {
    let opts: Opts = Opts::parse();
    println!("{:?}", opts);
    let client = Client::new();
    let result = match opts.subcmd {
        SubCommend::Get(ref args) => get(client, args).await?,
        SubCommend::Post(ref args) => post(client, args).await?,
    };
    Ok(result)
}


async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let resp = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}

async fn print_resp(resp: Response) -> Result<()> {
    printf_status(&resp);
    printf_header(&resp);
    let mine = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mine, &body);
    Ok(())
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

fn printf_header(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value)
    }
    print!("\n")
}

fn printf_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

// 打印服务器返回的 HTTP body
fn print_body(m: Option<Mime>, body: &String) {
    match m {
        // 对于 "application/json" 我们pretty print 
        Some(v) if v == mime::APPLICATION_JSON => { println!("{}", jsonxf::pretty_print(body).unwrap().cyan()) } // 其它 mime type，我们就直接输出 
        _ => println!("{}", body),
    }
}