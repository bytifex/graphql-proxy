use axum::http::HeaderMap;

#[macro_export]
macro_rules! log_location {
    () => {
        format!("{}:{}", file!(), line!())
    };
}

pub fn move_and_replace_headers(
    dest: &mut HeaderMap,
    src: &mut HeaderMap,
    prohibited_header_names: &[&str],
) {
    for (header_name, header_value) in src.drain() {
        if let Some(header_name) = header_name {
            if !prohibited_header_names.contains(&header_name.as_str()) {
                dest.remove(&header_name);
                dest.append(header_name, header_value);
            }
        }
    }
}
