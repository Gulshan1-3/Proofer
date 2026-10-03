pub mod protocol;
pub mod hover;
pub mod completion;
pub mod definition;

use std::collections::HashMap;
use std::io::{stdin, stdout, BufReader, Write};
use crate::api::{process_proof_request, json_str};
use protocol::{read_message, send_response, send_notification};

/// Run the official Proofer Language Server Protocol (LSP) daemon over stdio.
pub fn run_server() -> std::io::Result<()> {
    let stdin = stdin();
    let mut reader = BufReader::new(stdin.lock());
    let stdout = stdout();
    let mut writer = stdout.lock();

    let mut documents: HashMap<String, String> = HashMap::new();

    while let Ok(Some(msg)) = read_message(&mut reader) {
        handle_lsp_message(&msg, &mut documents, &mut writer)?;
    }

    Ok(())
}

fn handle_lsp_message<W: Write>(
    msg: &str,
    documents: &mut HashMap<String, String>,
    writer: &mut W,
) -> std::io::Result<()> {
    let method = extract_field(msg, "\"method\":");
    let id_opt = extract_raw_id(msg);

    match method.as_deref() {
        Some("initialize") => {
            let capabilities = "{\
                \"capabilities\": {\
                    \"textDocumentSync\": 1,\
                    \"hoverProvider\": true,\
                    \"completionProvider\": {\
                        \"triggerCharacters\": [\" \", \":\", \".\"]\
                    },\
                    \"definitionProvider\": true\
                },\
                \"serverInfo\": {\
                    \"name\": \"proofer-lsp\",\
                    \"version\": \"0.1.0\"\
                }\
            }";
            if let Some(ref id) = id_opt {
                send_response(writer, id, capabilities)?;
            }
        }
        Some("initialized") => {
            // Client is initialized, nothing to respond
        }
        Some("textDocument/didOpen") => {
            if let Some((uri, text)) = parse_did_open(msg) {
                documents.insert(uri.clone(), text.clone());
                publish_diagnostics(writer, &uri, &text)?;
            }
        }
        Some("textDocument/didChange") => {
            if let Some((uri, text)) = parse_did_change(msg) {
                documents.insert(uri.clone(), text.clone());
                publish_diagnostics(writer, &uri, &text)?;
            }
        }
        Some("textDocument/didSave") => {
            if let Some(uri) = parse_uri_only(msg) {
                if let Some(text) = documents.get(&uri) {
                    publish_diagnostics(writer, &uri, text)?;
                }
            }
        }
        Some("textDocument/hover") => {
            if let Some(ref id) = id_opt {
                if let Some((uri, line, character)) = parse_position_params(msg) {
                    if let Some(doc) = documents.get(&uri) {
                        let hover_json = hover::get_hover_markdown(doc, line, character)
                            .unwrap_or_else(|| "null".to_string());
                        send_response(writer, id, &hover_json)?;
                    } else {
                        send_response(writer, id, "null")?;
                    }
                } else {
                    send_response(writer, id, "null")?;
                }
            }
        }
        Some("textDocument/completion") => {
            if let Some(ref id) = id_opt {
                if let Some((uri, line, character)) = parse_position_params(msg) {
                    if let Some(doc) = documents.get(&uri) {
                        let comp_json = completion::get_completions(doc, line, character);
                        send_response(writer, id, &comp_json)?;
                    } else {
                        send_response(writer, id, "{\"isIncomplete\": false, \"items\": []}")?;
                    }
                } else {
                    send_response(writer, id, "{\"isIncomplete\": false, \"items\": []}")?;
                }
            }
        }
        Some("textDocument/definition") => {
            if let Some(ref id) = id_opt {
                if let Some((uri, line, character)) = parse_position_params(msg) {
                    if let Some(doc) = documents.get(&uri) {
                        let def_json = definition::get_definition(doc, line, character, &uri)
                            .unwrap_or_else(|| "null".to_string());
                        send_response(writer, id, &def_json)?;
                    } else {
                        send_response(writer, id, "null")?;
                    }
                } else {
                    send_response(writer, id, "null")?;
                }
            }
        }
        Some("proofer/getWorkstationState") => {
            if let Some(ref id) = id_opt {
                if let Some(uri) = parse_uri_only(msg) {
                    let doc = documents.get(&uri).cloned().unwrap_or_default();
                    let ver = process_proof_request(&doc);
                    send_response(writer, id, &ver)?;
                } else {
                    send_response(writer, id, "null")?;
                }
            }
        }
        Some("proofer/synthesizeStep") => {
            if let Some(ref id) = id_opt {
                if let Some(uri) = parse_uri_only(msg) {
                    let doc = documents.get(&uri).cloned().unwrap_or_default();
                    if let Some(step) = crate::synthesizer::ProofSynthesizer::infill_next_step(&doc) {
                        send_response(writer, id, &step.to_json())?;
                    } else {
                        send_response(writer, id, "null")?;
                    }
                } else {
                    send_response(writer, id, "null")?;
                }
            }
        }
        Some("shutdown") => {
            if let Some(ref id) = id_opt {
                send_response(writer, id, "null")?;
            }
        }
        Some("exit") => {
            std::process::exit(0);
        }
        _ => {
            // Unsupported or notification
            if let Some(ref id) = id_opt {
                send_response(writer, id, "null")?;
            }
        }
    }

    Ok(())
}

fn publish_diagnostics<W: Write>(writer: &mut W, uri: &str, text: &str) -> std::io::Result<()> {
    let ver_json = process_proof_request(text);
    let mut diags = Vec::new();

    // Check if error
    if ver_json.contains("\"status\": \"Error\"") || ver_json.contains("\"status\": \"Rejected\"") {
        let lines: Vec<&str> = text.lines().collect();

        // Extract error strings
        if let Some(errs_pos) = ver_json.find("\"errors\": [") {
            let rest = &ver_json[errs_pos + 11..];
            if let Some(end_bracket) = rest.find(']') {
                let errs_slice = &rest[..end_bracket];
                for part in errs_slice.split("\",") {
                    let clean = part.replace('\"', "").trim().to_string();
                    if !clean.is_empty() {
                        let mut line_no = lines.len().saturating_sub(1);
                        if let Some(idx) = lines.iter().position(|l| l.contains("derive ") || l.contains("suppose ")) {
                            line_no = idx;
                        }

                        diags.push(format!(
                            "{{\"range\": {{\"start\": {{\"line\": {}, \"character\": 0}}, \"end\": {{\"line\": {}, \"character\": 100}}}}, \"severity\": 1, \"source\": \"proofer\", \"message\": {}}}",
                            line_no, line_no, json_str(&clean)
                        ));
                    }
                }
            }
        }
    }

    let params = format!(
        "{{\"uri\": {}, \"diagnostics\": [{}]}}",
        json_str(uri),
        diags.join(", ")
    );
    send_notification(writer, "textDocument/publishDiagnostics", &params)?;

    // Also push custom workstationUpdate notification
    let ws_params = format!(
        "{{\"uri\": {}, \"verification\": {}}}",
        json_str(uri),
        ver_json
    );
    send_notification(writer, "proofer/workstationUpdate", &ws_params)?;

    Ok(())
}

fn extract_field(json: &str, key: &str) -> Option<String> {
    let pos = json.find(key)?;
    let rest = &json[pos + key.len()..];
    let quote_start = rest.find('\"')?;
    let after_quote = &rest[quote_start + 1..];
    let quote_end = after_quote.find('\"')?;
    Some(after_quote[..quote_end].to_string())
}

fn extract_raw_id(json: &str) -> Option<String> {
    let pos = json.find("\"id\":")?;
    let rest = json[pos + 5..].trim_start();
    if rest.starts_with('\"') {
        let after_quote = &rest[1..];
        let quote_end = after_quote.find('\"')?;
        Some(format!("\"{}\"", &after_quote[..quote_end]))
    } else {
        let end = rest.find([',', '}'])?;
        Some(rest[..end].trim().to_string())
    }
}

fn parse_uri_only(json: &str) -> Option<String> {
    extract_field(json, "\"uri\":")
}

fn parse_did_open(json: &str) -> Option<(String, String)> {
    let uri = extract_field(json, "\"uri\":")?;
    let text_pos = json.find("\"text\":")?;
    let rest = &json[text_pos + 7..];
    let start_quote = rest.find('\"')?;
    let rest_quote = &rest[start_quote + 1..];
    let end_quote = rest_quote.rfind('\"')?;
    let text = rest_quote[..end_quote].replace("\\n", "\n").replace("\\\"", "\"").replace("\\\\", "\\");
    Some((uri, text))
}

fn parse_did_change(json: &str) -> Option<(String, String)> {
    parse_did_open(json)
}

fn parse_position_params(json: &str) -> Option<(String, usize, usize)> {
    let uri = extract_field(json, "\"uri\":")?;
    let line_pos = json.find("\"line\":")?;
    let rest_line = &json[line_pos + 7..];
    let end_line = rest_line.find([',', '}'])?;
    let line: usize = rest_line[..end_line].trim().parse().ok()?;

    let char_pos = json.find("\"character\":")?;
    let rest_char = &json[char_pos + 12..];
    let end_char = rest_char.find([',', '}'])?;
    let character: usize = rest_char[..end_char].trim().parse().ok()?;

    Some((uri, line, character))
}
