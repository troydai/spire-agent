use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use base64::Engine;
use serde::Serialize;

use crate::rpc::{WorkloadClient, connect_workload_client};
use crate::workload::{JwtBundlesRequest, JwtBundlesResponse, JwtsvidRequest, JwtsvidResponse};

pub async fn fetch_jwt(
    socket_path: &str,
    timeout: Duration,
    audience: Vec<String>,
    spiffe_id: Option<String>,
    output: &str,
) -> Result<()> {
    let mut client = connect_workload_client(socket_path).await?;
    let svids = fetch_jwtsvid(&mut client, timeout, audience, spiffe_id).await?;
    let bundles = fetch_jwt_bundles(&mut client, timeout).await?;

    match output {
        "pretty" => print_pretty(&svids, &bundles),
        "json" => print_json(&svids, &bundles),
        _ => anyhow::bail!("unsupported output format: {output}"),
    }
}

async fn fetch_jwtsvid(
    client: &mut WorkloadClient,
    timeout: Duration,
    audience: Vec<String>,
    spiffe_id: Option<String>,
) -> Result<JwtsvidResponse> {
    let request = tonic::Request::new(JwtsvidRequest {
        audience,
        spiffe_id: spiffe_id.unwrap_or_default(),
    });

    let response = tokio::time::timeout(timeout, client.fetch_jwtsvid(request))
        .await
        .context("request timed out")?
        .context("failed to fetch jwt svid")
        .map(tonic::Response::into_inner)?;

    if response.svids.is_empty() {
        anyhow::bail!("empty response from server");
    }
    Ok(response)
}

async fn fetch_jwt_bundles(
    client: &mut WorkloadClient,
    timeout: Duration,
) -> Result<JwtBundlesResponse> {
    let request = tonic::Request::new(JwtBundlesRequest {});
    let response = tokio::time::timeout(timeout, client.fetch_jwt_bundles(request))
        .await
        .context("request timed out")?
        .context("failed to fetch jwt bundles")?;

    let mut stream = response.into_inner();
    tokio::time::timeout(timeout, stream.message())
        .await
        .context("timed out waiting for response")?
        .context("failed to receive message")?
        .context("empty response from server")
        .map_err(anyhow::Error::from)
}

fn print_pretty(
    jwtsvid_response: &JwtsvidResponse,
    bundles_response: &JwtBundlesResponse,
) -> Result<()> {
    for svid in &jwtsvid_response.svids {
        println!("token({}):", svid.spiffe_id);
        println!("\t\t{}", svid.svid);
    }

    for (trust_domain, bundle) in &bundles_response.bundles {
        println!("bundle({trust_domain}):");
        println!("\t\t{}", format_bundle_pretty(bundle)?);
    }

    Ok(())
}

fn format_bundle_pretty(bundle: &[u8]) -> Result<String> {
    let value: serde_json::Value =
        serde_json::from_slice(bundle).context("failed to parse jwt bundle")?;
    serde_json::to_string_pretty(&value).context("failed to format jwt bundle")
}

fn print_json(
    jwtsvid_response: &JwtsvidResponse,
    bundles_response: &JwtBundlesResponse,
) -> Result<()> {
    let payload = vec![
        JsonEntry::Svids {
            svids: jwtsvid_response
                .svids
                .iter()
                .map(|svid| JsonSvid {
                    hint: svid.hint.clone(),
                    spiffe_id: svid.spiffe_id.clone(),
                    svid: svid.svid.clone(),
                })
                .collect(),
        },
        JsonEntry::Bundles {
            bundles: bundles_response
                .bundles
                .iter()
                .map(|(trust_domain, bundle)| {
                    (
                        trust_domain.clone(),
                        base64::engine::general_purpose::STANDARD.encode(bundle),
                    )
                })
                .collect(),
        },
    ];

    println!("{}", serde_json::to_string(&payload)?);
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct JsonSvid {
    hint: String,
    spiffe_id: String,
    svid: String,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
enum JsonEntry {
    Svids { svids: Vec<JsonSvid> },
    Bundles { bundles: HashMap<String, String> },
}

#[cfg(test)]
mod tests {
    use super::format_bundle_pretty;

    #[test]
    fn format_bundle_pretty_formats_json() {
        let raw = br#"{"keys":[{"kty":"EC"}]}"#;
        let pretty = format_bundle_pretty(raw).unwrap();
        assert!(pretty.contains("\"keys\""));
        assert!(pretty.contains("\n"));
    }
}
