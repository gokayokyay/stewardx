use std::collections::HashMap;
use url::Url;

pub fn get_qs(from: &str) -> anyhow::Result<HashMap<String, String>> {
    let parsed_url = match Url::parse(from) {
        Ok(url) => url,
        Err(e) => match e {
            url::ParseError::RelativeUrlWithoutBase => {
                return get_qs(&format!("http://0.0.0.0{}", from));
            }
            _ => return Err(anyhow::anyhow!(e)),
        },
    };
    let hash_query: HashMap<_, _> = parsed_url.query_pairs().into_owned().collect();
    return Ok(hash_query);
}
