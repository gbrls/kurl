use clap::Parser;
use colored::Colorize;
use reqwest;

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

    /// Display all status
    #[arg(long)]
    all: bool,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let nurl = if !(args.url.starts_with("http://") || args.url.starts_with("https://")) {
        format!("http://{}", args.url)
    } else {
        args.url
    };

    let req = reqwest::blocking::get(nurl)?;

    let status = req.status();

    let headers = req.headers().to_owned();
    let content_type = headers
        .get("Content-Type")
        .map(|x| x.to_str().unwrap_or("null"))
        .unwrap_or("null");

    let mut data: Option<String> = None;

    let len = if req.content_length().is_some() {
        let len = req.content_length().unwrap();
        data = Some(req.text()?);
        len
    } else {
        data = Some(req.text()?);
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

    if args.valid_json || args.all {
        if is_json {
            buf.push(format!("{}", "json").green().bold());
        } else {
            buf.push(format!("{}", "notjson").normal());
        }
    }

    if is_json && (args.keys || args.all) {
        let json = serde_json::from_str::<serde_json::Value>(&data.as_ref().unwrap())?;
        buf.push(format!("\"{}\"", get_keys(&json).join(" ")).white().bold());
    }

    if args.content_type || args.all {
        buf.push(format!("\"{}\"", content_type).normal());
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

    if !buf.is_empty() {
        println!(
            "{}",
            buf.into_iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<_>>()
                .join(" ")
        );
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
