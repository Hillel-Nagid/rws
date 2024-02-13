use std::io::{Error, ErrorKind};

use axum::http::HeaderMap;

pub fn extract_header<'a>(headers: &'a HeaderMap, header_name: &str) -> Option<&'a str> {
    if let Some(header) = headers.get(header_name) {
        if let Ok(header_str) = header.to_str() {
            return Some(header_str);
        }
        return None;
    }
    return None;
}

pub fn check_match(headers: &HeaderMap, etag: String) -> Result<Option<bool>, Error> {
    if let Some(match_etag) = extract_header(headers, "IF-MATCH") {
        if String::from(match_etag) == etag {
            return Ok(Some(true));
        }
        return Ok(Some(false));
    }
    if let Some(match_etag) = extract_header(headers, "IF-NONE-MATCH") {
        if String::from(match_etag) != etag {
            return Ok(Some(true));
        }
        return Ok(Some(false));
    }
    return Ok(None);
}

pub fn check_since(headers: &HeaderMap, last_modified: i64) -> Result<Option<bool>, Error> {
    if let Some(since) = extract_header(headers, "IF-MODIFIED-SINCE") {
        if let Ok(since_i64) = since.parse::<i64>() {
            if since_i64 > last_modified {
                return Ok(Some(true));
            }
            return Ok(Some(false));
        }
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Could'nt parse input timestamp",
        ));
    }
    if let Some(since) = extract_header(headers, "IF-UNMODIFIED-SINCE") {
        if let Ok(since_i64) = since.parse::<i64>() {
            if since_i64 < last_modified {
                return Ok(Some(true));
            }
            return Ok(Some(false));
        }
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Could'nt parse input timestamp",
        ));
    }
    return Ok(None);
}
