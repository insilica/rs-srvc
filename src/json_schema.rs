use std::collections::HashMap;

use jsonschema::{CompilationOptions, Draft, JSONSchema};
use reqwest::blocking::Client;

use lib_sr::errors::*;

lazy_static! {
    static ref EMBEDDED_DOCUMENTS: HashMap<&'static str, &'static str> = {
        hashmap! {
            "label-answer/boolean-v1" => include_str!("schema/label-answer/boolean-v1.json"),
            "label-answer/boolean-v2" => include_str!("schema/label-answer/boolean-v2.json"),
            "label-answer/string-v1" => include_str!("schema/label-answer/string-v1.json"),
            "label-answer/string-v2" => include_str!("schema/label-answer/string-v2.json"),
        }
    };
}

lazy_static! {
    static ref PARSED_EMBEDDED_DOCUMENTS: HashMap<String, serde_json::Value> = {
        let mut m = HashMap::new();
        for (name, s) in &*EMBEDDED_DOCUMENTS {
            let json: serde_json::Value = serde_json::from_str(s)
                .chain_err(|| "Deserialization failed")
                .expect("Invalid JSON");
            m.insert(
                format!("http://docs.sysrev.com/schema/{}.json", name),
                json.to_owned(),
            );
            m.insert(
                format!("https://docs.sysrev.com/schema/{}.json", name),
                json.to_owned(),
            );
            m.insert(
                format!(
                    "http://raw.githubusercontent.com/insilica/rs-srvc/master/src/schema/{}.json",
                    name
                ),
                json.to_owned(),
            );
            m.insert(
                format!(
                    "https://raw.githubusercontent.com/insilica/rs-srvc/master/src/schema/{}.json",
                    name
                ),
                json,
            );
        }
        m
    };
}

lazy_static! {
    static ref OPTIONS: CompilationOptions = {
        let mut opts = JSONSchema::options();
        opts.with_draft(Draft::Draft7);
        for (url, val) in &*PARSED_EMBEDDED_DOCUMENTS {
            opts.with_document(String::from(url), val.to_owned());
        }
        opts
    };
}

fn validation_error_message(e: jsonschema::ValidationError) -> String {
    // Work around lifetime complications caused by jsonschema's
    // ValidationError referencing the schema data
    format!(
        "Failed to compile JSON schema at {}: {}",
        e.instance_path,
        e.to_string()
    )
}

pub fn compile(schema: &serde_json::Value) -> Result<JSONSchema> {
    match OPTIONS.compile(schema) {
        Ok(jsonschema) => Ok(jsonschema),
        Err(e) => Err(validation_error_message(e).into()),
    }
}

pub fn get_schema_for_url(client: &Client, url: &str) -> Result<serde_json::Value> {
    match PARSED_EMBEDDED_DOCUMENTS.get(url) {
        Some(val) => Ok(val.to_owned()),
        None => {
            let response = client
                .get(url)
                .send()
                .chain_err(|| format!("Error while retrieving json schema at URL: {}", url))?;
            let status = response.status().as_u16();
            let text = response
                .text()
                .chain_err(|| "Error getting response text")?;
            if status == 200 {
                serde_json::from_str(&text).chain_err(|| "Could not parse reponse as JSON")
            } else {
                Err(format!(
                    "Unexpected {} status response at {} ({})",
                    status, &url, text
                )
                .into())
            }
        }
    }
}
