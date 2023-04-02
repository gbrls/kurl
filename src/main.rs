pub mod print;

use anyhow::{Context, Result};
use clap::{Arg, Parser, ValueEnum};
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

    #[arg(short = 'c', long)]
    status_code: bool,

    #[arg(short, long)]
    size: bool,

    /// Validate the data as xml or json
    #[arg(long)]
    validate: bool,

    #[arg(short = 't', long)]
    content_type: bool,

    #[arg(short, long)]
    no_body: bool,

    /// Try to guess the JSON's format
    #[arg(short, long)]
    keys: bool,

    /// Display the URL
    #[arg(short = 'u', long)]
    show_url: bool,

    /// Number of parallel threads to send the requests
    #[arg(short = 'p', default_value = "4")]
    nworkers: usize,

    /// Display all status
    #[arg(long)]
    all: bool,

    #[arg(long)]
    scripts: Vec<String>,

    #[arg(short = 'X', default_value = "GET")]
    verb: Verb,

    #[arg(short, long)]
    data: Option<String>,

    #[arg(long)]
    verbose: bool,
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
        .timeout(Duration::from_secs(2))
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

/// This is a blocking function that will only return when all the requests
fn execute_requests(args: &CliArgs) {
    let params = urls(&args)
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
                    print::log_response(&args, response, &url).unwrap();
                    tx.send(1).unwrap();
                }

                // TODO: Log error when in verbose mode
                Err(_) => tx.send(0).unwrap(),
            }
        });
    }

    // TODO tif some request panics, this will halt the application
    let res = rx.into_iter().take(n).collect_vec();
    // verbose
    //eprintln!("{res:?}");
}

fn main() -> Result<()> {
    let args = CliArgs::parse();
    execute_requests(&args);

    Ok(())
}
