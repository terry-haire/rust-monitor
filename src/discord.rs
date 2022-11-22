extern crate reqwest;

/// Print the given msg to discord.
pub fn print_to_discord(channel_id: &str, token: &str, msg: &str) ->
        Result<(), Box<dyn std::error::Error>> {
    let url_str = format!("https://discordapp.com/api/webhooks/{}/{}",
            channel_id,
            token);
    let url = reqwest::Url::parse(&url_str)?;

    // Create JSON.
    let content: String = format!("{{\n\t\"content\": \"{}\"\n}}", msg.to_string());

    let mut headers = reqwest::header::HeaderMap::new();

    // Set content-type.
    headers.insert(reqwest::header::CONTENT_TYPE, "application/json".parse()
        .unwrap());

    let client = reqwest::blocking::Client::new();

    println!("{}", content);

    // Send post request.
    let res = client.post(url)
        .headers(headers)
        .body(content)
        .send()?;

    // TODO: return error.
    if res.status() != reqwest::StatusCode::NO_CONTENT {
        return Err(format!("Unknown status {}", res.status()).into());
    }

    Ok(())
}
