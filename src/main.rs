pub mod print;
mod tests;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use itertools::Itertools;
use reqwest::{
    self,
    blocking::{ClientBuilder, Response},
};
use std::time::Duration;
use std::{fs, sync::mpsc::channel};
use thiserror::Error;
use threadpool::ThreadPool;

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq)]
#[clap(rename_all = "UPPER")]
pub enum Verb {
    POST,
    GET,
    HEAD,
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("invalid hostname (`{0}`)")]
    InvalidHostname(String),
    #[error("unkown error(`{0}`)")]
    Unkown(reqwest::Error),
}

#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// URL or file with URLs to send the request
    url_or_file: String,

    /// Number of parallel threads to send the requests
    #[arg(short = 'p', default_value = "4")]
    nworkers: usize,

    #[arg(short = 'X', default_value = "GET")]
    verb: Verb,

    #[arg(short = 'b', long = "body")]
    show_response_body: bool,

    /// Data to be sent in the request body
    #[arg(short, long)]
    data: Option<String>,

    #[arg(long, default_value = "0")]
    verbosity_level: usize,

    /// File to write the results
    #[arg(short)]
    output: Option<String>,

    /// Extensions to be ignored
    #[arg(
        long = "fext",
        default_value = "jpeg,png,jpg,gif,wof,ttf,otf,eot,swf,ico,svg,css,woff,woff2"
    )]
    filter_extensions: String,

    /// Status codes to be ignored
    #[arg(long = "fstatus", default_value = "404")]
    filter_status: String,
}

impl CliArgs {
    pub fn filter_extensions(&self) -> Vec<String> {
        self.filter_extensions
            .clone()
            .split(",")
            .map(|s| format!(".{}", s))
            .collect_vec()
    }

    pub fn filter_status(&self) -> Vec<u16> {
        self.filter_status
            .clone()
            .split(",")
            .map(|s| s.parse().unwrap())
            .collect_vec()
    }
}

#[derive(Debug, Clone)]
struct RequestParam {
    http_verb: Verb,
    // refactor this to be more strongly typed
    url: String,
    // refactor this to &[u8]
    body: String,
}

//TODO: Run any script from bash here
fn run_scripts(_scripts: &[String]) -> Result<()> {
    todo!("Not implemented")
}

fn to_url(s: &str) -> String {
    if !(s.starts_with("http://") || s.starts_with("https://")) {
        format!("http://{}", &s)
    } else {
        s.to_string()
    }
}

fn request(
    RequestParam {
        http_verb,
        url,
        body,
    }: RequestParam,
) -> Result<reqwest::blocking::Response> {
    let client = ClientBuilder::new()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(30))
        .build()
        .context("Error building http client")
        .unwrap();

    match http_verb {
        Verb::GET => client.get(&url),
        Verb::POST => client.post(&url),
        Verb::HEAD => client.head(&url),
    }
    .body(body)
    .send()
    .map_err(|e| {
        let s = format!("{:#?}", e);
        if s.contains("dns error") {
            RequestError::InvalidHostname(url)
        } else {
            // Turn this on in verbose mode
            //eprintln!("{:#?}", e);
            RequestError::Unkown(e)
        }
    })
    .context("While sending request")
}

fn urls(args: &CliArgs) -> Vec<String> {
    let file_or_url = &args.url_or_file;
    // check if file exits
    match fs::read_to_string(&file_or_url) {
        Ok(urls) => urls.lines().into_iter().map(|s| to_url(s)).collect_vec(),
        Err(_) => vec![to_url(&file_or_url.clone())],
    }
}

fn filter_by_extension(args: &CliArgs, urls: Vec<String>) -> Vec<String> {
    let exts = args.filter_extensions();

    urls.into_iter()
        .filter(|url| exts.iter().all(|ext| !url.ends_with(ext)))
        .collect_vec()
}

/// This is a blocking function that will only return when all the requests
fn execute_requests(args: &CliArgs) -> Result<()> {
    let params = filter_by_extension(&args, urls(&args))
        .into_iter()
        .map(|url| RequestParam {
            http_verb: args.verb,
            url: url,
            body: args.data.clone().unwrap_or("".into()),
        })
        .collect_vec();

    let pool = ThreadPool::new(args.nworkers);
    let (tx, rx) = channel();

    let n = params.len();

    for rp in params.into_iter() {
        let tx = tx.clone();

        let args = args.clone();

        pool.execute(move || {
            let url = rp.url.clone();
            // verbose mode
            //eprintln!("started {}", rp.url);
            match request(rp) {
                Ok(response) => {
                    let res = print::log_response(&args, response, &url);
                    tx.send(res.unwrap_or("".to_owned())).unwrap();
                }

                // TODO: Log error when in verbose mode
                Err(_) => tx.send("".to_owned()).unwrap(),
            }
        });
    }

    // TODO: if some request panics, this will halt the application
    let res = rx.into_iter().take(n).collect_vec();
    // verbose
    //eprintln!("{res:?}");

    print::write_results(args, res)
}

fn banner() {
    let main = r#"
██ ▄█▀ █    ██  ██▀███   ██▓    
██▄█▒  ██  ▓██▒▓██ ▒ ██▒▓██▒    
▓███▄░ ▓██  ▒██░▓██ ░▄█ ▒▒██░    
▓██ █▄ ▓▓█  ░██░▒██▀▀█▄  ▒██░    
▒██▒ █▄▒▒█████▓ ░██▓ ▒██▒░██████▒
▒ ▒▒ ▓▒░▒▓▒ ▒ ▒ ░ ▒▓ ░▒▓░░ ▒░▓  ░
░ ░▒ ▒░░░▒░ ░ ░   ░▒ ░ ▒░░ ░ ▒  ░
░ ░░ ░  ░░░ ░ ░   ░░   ░   ░ ░   
░  ░      ░        ░         ░  ░
    "#
    .bright_yellow();

    let version: &str = env!("CARGO_PKG_VERSION");

    eprint!("{}", main);

    eprintln!("v{} - By: gbrls\n", version);
}

fn main() -> Result<()> {
    let args = CliArgs::parse();
    banner();
    execute_requests(&args)?;

    Ok(())
}
