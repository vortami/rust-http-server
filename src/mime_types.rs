//! Mime type detection based on filename

fn last_two<T, I: Iterator<Item = T>>(mut iter: I) -> (Option<T>, Option<T>) {
    let mut a = None;
    let mut b = None;

    loop {
        let next = iter.next();
        if next.is_some() {
            a = b;
            b = next;
        } else {
            break;
        }
    }

    (a, b)
}

fn get_file_extension(path: &str) -> Option<String> {
    let path = path.split('/').last()?;
    let split = path.split('.');

    let (a, b) = last_two(split);
    a.and(b).map(|s| s.to_string())
}

// should i use a struct or a mod here?
/// util for mime types
pub struct MimeType;

impl MimeType {
    /// get the mime type based on file extension
    /// 
    /// # Examples
    /// ```
    /// # use rust_http_server::mime_types::MimeType;
    /// assert_eq!(MimeType::get_for_path("txt"), "text/plain");
    /// assert_eq!(MimeType::get_for_path("html"), "text/html");
    /// assert_eq!(MimeType::get_for_path("png"), "image/png");
    /// ```
    pub fn get_for_extension(ext: &str) -> String {
        match ext.to_lowercase().as_str() {
            // image
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "svg" => "image/svg+xml",

            // text
            "html" => "text/html",
            "txt" => "text/plain",
            /* fallback */ _ => "text/plain",
        }
        .to_string()
    }
    
    /// get the mime type based on file path
    /// 
    /// # Examples
    /// ```
    /// # use rust_http_server::mime_types::MimeType;
    /// assert_eq!(MimeType::get_for_extension("./relative/file.png"), "image/png");
    /// ```
    pub fn get_for_path(path: &str) -> String {
        let ext = match get_file_extension(path) {
            Some(ext) => ext,
            None => return Self::get_for_extension("txt"),
        };

        Self::get_for_extension(&ext)
    }
}

#[test]
fn test_last_two() {
    let vec = vec![1, 2, 3, 4];
    let out = last_two(vec.iter());

    assert_eq!(out, (Some(&3), Some(&4)));

    let vec = vec![1];
    let out = last_two(vec.iter());
    assert_eq!(out, (None, Some(&1)));
}

#[test]
fn test_file_ext() {
    assert_eq!(
        get_file_extension("/absolute/file.txt"),
        Some("txt".to_string())
    );

    assert_eq!(
        get_file_extension("./relative/file.png"),
        Some("png".to_string())
    );

    assert_eq!(get_file_extension("onlyname.rs"), Some("rs".to_string()));

    assert_eq!(get_file_extension("noextension"), None);
    assert_eq!(get_file_extension("/absolute/noextension"), None);
    assert_eq!(get_file_extension("./relative/noextension"), None);
}
