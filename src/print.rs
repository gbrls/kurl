use anyhow::anyhow;
use strip_ansi_escapes::strip;

use crate::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DataFormat {
    Json(serde_json::Value),
    Xml(xmltree::Element),
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

pub fn log_response(args: &CliArgs, resp: Response, url: &str) -> Result<String> {
    let headers = resp.headers().clone();

    let status = resp.status();
    let filter_status = args.filter_status();

    let u16_status = status.as_u16();

    if filter_status.into_iter().any(|fs| u16_status == fs) {
        return Err(anyhow!("Filtered"));
    }

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
    let mut ret_str = String::new();

    // status_code
    if status.is_success() {
        buf.push(format!("{}", status.as_u16()).green());
    } else if status.is_server_error() {
        buf.push(format!("{}", status.as_u16()).red())
    } else if status.is_client_error() {
        buf.push(format!("{}", status.as_u16()).yellow())
    } else {
        buf.push(format!("{}", status.as_u16()).black())
    }

    // response size
    buf.push(format!("{}", len).normal());

    // http verb
    match args.verb {
        Verb::GET => buf.push("get".green()),
        Verb::POST => buf.push("post".blue()),
        Verb::HEAD => buf.push("head".yellow()),
    }

    // data format
    {
        use DataFormat::*;
        match &data_fmt {
            Some(Json(_)) => buf.push("json".green().bold()),
            Some(Xml(_)) => buf.push("xml".purple().bold()),
            None => buf.push("none".normal()),
        }
    }

    // show keys from json or xml
    if data_fmt.is_some() {
        use DataFormat::*;
        let keys = match data_fmt.unwrap() {
            Json(json) => get_json_keys(&json),
            Xml(xml) => get_xml_keys(&xml),
        };
        buf.push(format!("\"{}\"", keys.join(" ")).white().bold());
    }

    // content-type
    buf.push(format!("\"{}\"", content_type).normal());

    // url
    buf.push(format!("{}", url).normal());

    if args.show_response_body {
        buf.push(
            format!(
                "{}{}",
                if buf.is_empty() { "" } else { "\n" },
                data.unwrap()
            )
            .normal(),
        );
    }

    //TODO: implement header printing

    let final_string = buf
        .into_iter()
        .map(|x| format!("{}", x))
        .collect_vec()
        .join(" ");

    if !final_string.is_empty() {
        println!("{}", final_string);
    }

    let no_colors =
        strip(final_string.as_bytes()).context("while removing ansi symbols from string")?;
    let no_colors = std::str::from_utf8(&no_colors).context("while decoding str to utf-8")?;
    //snailquote::unescape(no_colors).context("while unescaping string")
    Ok(no_colors.to_owned())
}

pub fn write_results(args: &CliArgs, data: Vec<String>) -> Result<()> {
    if args.output.is_none() {
        return Ok(());
    }

    let fname = args.output.clone().unwrap();

    let s = data
        .into_iter()
        .filter(|s| !s.is_empty())
        .sorted_by_key(|s| {
            let len: String = s.split_ascii_whitespace().skip(1).take(1).collect();
            let n: usize = len.parse().unwrap();
            // -n for inverse
            1i64 - (n as i64)
        })
        .join("\n");

    std::fs::write(fname, format!("{}\n", s)).context("while writing to file")
}
