use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use itertools::Itertools;
use reqwest::{self, blocking::ClientBuilder};
use thiserror::Error;

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq)]
#[clap(rename_all = "UPPER")]
enum Verb {
    POST,
    GET,
    HEAD,
}

#[derive(Clone, Debug, PartialEq)]
enum DataFormat {
    Json(serde_json::Value),
    Xml(xmltree::Element),
}

#[derive(Error, Debug)]
enum RequestError {
    #[error("invalid hostname (`{0}`)")]
    InvalidHostname(String),
    #[error("unkown error(`{0}`)")]
    Unkown(reqwest::Error),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// URL to send the request
    url: String,

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

fn get_json_keys(json: &serde_json::Value) -> Vec<String> {
    if json.is_object() {
        json.as_object()
            .unwrap()
            .keys()
            .map(|x| x.to_owned())
            .collect::<Vec<String>>()
    } else if json.is_array() {
        let arr = json.as_array().unwrap().to_owned();

        if arr.len() > 0 {
            get_json_keys(&arr[0])
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

fn get_xml_keys(xml: &xmltree::Element) -> Vec<String> {
    if xml
        .children
        .iter()
        .any(|x| matches!(x, xmltree::XMLNode::Text(_)))
    {
        vec![xml.name.clone()]
    } else {
        xml.children
            .iter()
            .map(|x| match x {
                xmltree::XMLNode::Element(x) => get_xml_keys(x),
                _ => vec![],
            })
            .flatten()
            .unique()
            .collect::<Vec<_>>()
    }
}

//TODO: Run any script from bash here
fn run_scripts(scripts: &[String]) -> Result<()> {
    todo!("Not implemented")
}

fn get_format(data: &str) -> Option<DataFormat> {
    let data = data.trim().trim_start_matches("\u{feff}");
    match (
        serde_json::from_str::<serde_json::Value>(data),
        xmltree::Element::parse(data.as_bytes()),
    ) {
        (Ok(x), _) => Some(DataFormat::Json(x)),
        (_, Ok(x)) => Some(DataFormat::Xml(x)),
        (_, Err(_)) => None,
    }
}

fn request(
    RequestParam {
        http_verb,
        url,
        body,
    }: RequestParam,
) -> reqwest::blocking::Response {
    let client = ClientBuilder::new()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()
        .context("Error building http client")
        .unwrap();

    match http_verb {
        Verb::GET => client.get(&url),
        Verb::POST => client.post(&url),
        Verb::HEAD => client.head(&url),
    }
    //.danger_accept_invalid_hostnames(true)
    .body(body)
    //.danger_accept_invalid_certs(true)
    .send()
    .map_err(|e| {
        let s = format!("{:#?}", e);
        if s.contains("dns error") {
            RequestError::InvalidHostname(url)
        } else {
            eprintln!("{:#?}", e);
            RequestError::Unkown(e)
        }
    })
    .context("While sending request")
    .unwrap()
}

fn main() -> Result<()> {
    let args = Args::parse();

    let nurl = if !(args.url.starts_with("http://") || args.url.starts_with("https://")) {
        format!("http://{}", &args.url)
    } else {
        args.url.clone()
    };

    let resp = request(RequestParam {
        http_verb: args.verb,
        url: nurl,
        body: args.data.unwrap_or("".into()),
    });

    //let resp = reqwest::blocking::get(nurl)?;
    //let resp = reqwest::blocking::Client::new().get(nurl).send()?;

    let headers = resp.headers().clone();

    let status = resp.status();

    let headers = resp.headers().to_owned();
    let content_type = headers
        .get("Content-Type")
        .map(|x| x.to_str().unwrap_or("null"))
        .unwrap_or("null");

    let mut data: Option<String> = None;

    let len = if resp.content_length().is_some() {
        let len = resp
            .content_length()
            .context("While readig Content-Length")?;
        data = Some(resp.text().context("While reading reponse as text")?);
        len
    } else {
        data = Some(resp.text()?);
        data.as_ref().unwrap().len() as u64
    };

    let data_fmt = get_format(&data.as_ref().unwrap());

    let mut buf = vec![];

    if args.status_code || args.all {
        if status.is_success() {
            buf.push(format!("{}", status.as_u16()).green())
        } else if status.is_server_error() {
            buf.push(format!("{}", status.as_u16()).red())
        } else if status.is_client_error() {
            buf.push(format!("{}", status.as_u16()).yellow())
        } else {
            buf.push(format!("{}", status.as_u16()).black())
        }
    }

    if args.size || args.all {
        buf.push(format!("{}", len).normal());
    }

    if args.all {
        match args.verb {
            Verb::GET => buf.push("get".green()),
            Verb::POST => buf.push("post".blue()),
            Verb::HEAD => buf.push("head".yellow()),
        }
    }

    if args.validate || args.all {
        use DataFormat::*;
        match &data_fmt {
            Some(Json(_)) => buf.push("json".green().bold()),
            Some(Xml(_)) => buf.push("xml".purple().bold()),
            None => buf.push("none".normal()),
        }
    }

    if data_fmt.is_some() && (args.keys || args.all) {
        use DataFormat::*;
        let keys = match data_fmt.unwrap() {
            Json(json) => get_json_keys(&json),
            Xml(xml) => get_xml_keys(&xml),
        };
        buf.push(format!("\"{}\"", keys.join(" ")).white().bold());
    }

    if args.content_type || args.all {
        buf.push(format!("\"{}\"", content_type).normal());
    }

    if args.show_url || args.all {
        buf.push(format!("{}", &args.url).normal());
    }

    if !args.no_body {
        buf.push(
            format!(
                "{}{}",
                if buf.is_empty() { "" } else { "\n" },
                data.unwrap()
            )
            .normal(),
        );
    }

    //TODO: implement this with a pretty output
    //if !args.no_body && args.verb == Verb::HEAD {
    //    buf.push(
    //        headers
    //            .into_iter()
    //            .map(|(k, v)| format!("{:?} {:?}", k, v))
    //            .collect::<Vec<_>>()
    //            .join("\n")
    //            .normal(),
    //    );
    //}

    if !buf.is_empty() {
        println!(
            "{}",
            buf.into_iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }

    if !args.scripts.is_empty() {
        run_scripts(&args.scripts)?;
    }

    //println!(
    //    "{} {} {} \"{}\"",
    //    status.as_u16(),
    //    len,
    //    is_json,
    //    content_type.blue()
    //);
    Ok(())
}
