use clap::{Parser, ValueEnum};
use colored::Colorize;
use reqwest;

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq)]
#[clap(rename_all = "UPPER")]
enum Verb {
    POST,
    GET,
    HEAD,
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

    #[arg(short = 'j', long)]
    valid_json: bool,

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
}

fn get_keys(json: &serde_json::Value) -> Vec<String> {
    if json.is_object() {
        json.as_object()
            .unwrap()
            .keys()
            .map(|x| x.to_owned())
            .collect::<Vec<String>>()
    } else if json.is_array() {
        let arr = json.as_array().unwrap().to_owned();

        if arr.len() > 0 {
            get_keys(&arr[0])
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

//TODO: Run any script from bash here
fn run_scripts(scripts: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let nurl = if !(args.url.starts_with("http://") || args.url.starts_with("https://")) {
        format!("http://{}", &args.url)
    } else {
        args.url.clone()
    };

    //let resp = reqwest::blocking::get(nurl)?;
    //let resp = reqwest::blocking::Client::new().get(nurl).send()?;
    let client = reqwest::blocking::Client::new();
    let resp = match args.verb {
        Verb::GET => client.get(nurl),
        Verb::POST => client.post(nurl),
        Verb::HEAD => client.head(nurl),
        _ => panic!("Verb {:?} not implemented", args.verb),
    }
    .body(args.data.unwrap_or("".into()))
    .send()?;

    let headers = resp.headers().clone();

    let status = resp.status();

    let headers = resp.headers().to_owned();
    let content_type = headers
        .get("Content-Type")
        .map(|x| x.to_str().unwrap_or("null"))
        .unwrap_or("null");

    let mut data: Option<String> = None;

    let len = if resp.content_length().is_some() {
        let len = resp.content_length().unwrap();
        data = Some(resp.text()?);
        len
    } else {
        data = Some(resp.text()?);
        data.as_ref().unwrap().len() as u64
    };

    let is_json = serde_json::from_str::<serde_json::Value>(&data.as_ref().unwrap()).is_ok();

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

    if args.valid_json || args.all {
        if is_json {
            buf.push("json".green().bold());
        } else {
            buf.push("notjson".normal());
        }
    }

    if is_json && (args.keys || args.all) {
        let json = serde_json::from_str::<serde_json::Value>(&data.as_ref().unwrap())?;
        buf.push(format!("\"{}\"", get_keys(&json).join(" ")).white().bold());
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

    run_scripts(&args.scripts)?;

    //println!(
    //    "{} {} {} \"{}\"",
    //    status.as_u16(),
    //    len,
    //    is_json,
    //    content_type.blue()
    //);
    Ok(())
}
