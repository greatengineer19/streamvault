#[cfg(test)]
mod upload_validation {
    // ── Constants mirrored from routes.rs ─────────────────────────────────────
    // These match the validation rules in upload_init handler

    const MAX_SIZE_BYTES: i64 = 1024 * 1024 * 1024; // 1 GB
    const ALLOWED_MIME_TYPES: &[&str] = &[
        "video/mp4",
        "video/webm",
        "video/ogg",
        "video/quicktime",
    ];

    fn is_valid_mime(mime: &str) -> bool {
        ALLOWED_MIME_TYPES.contains(&mime)
    }

    fn is_valid_size(size: i64) -> bool {
        size <= MAX_SIZE_BYTES
    }

    // ── Mime type tests ───────────────────────────────────────────────────────

    #[test]
    fn accepts_mp4() {
        assert!(is_valid_mime("video/mp4"));
    }

    #[test]
    fn accepts_webm() {
        assert!(is_valid_mime("video/webm"));
    }

    #[test]
    fn accepts_ogg() {
        assert!(is_valid_mime("video/ogg"));
    }

    #[test]
    fn accepts_quicktime_mov() {
        assert!(is_valid_mime("video/quicktime"));
    }

    #[test]
    fn rejects_mkv() {
        assert!(!is_valid_mime("video/x-matroska"));
    }

    #[test]
    fn rejects_avi() {
        assert!(!is_valid_mime("video/x-msvideo"));
    }

    #[test]
    fn rejects_non_video_type() {
        assert!(!is_valid_mime("image/png"));
    }

    #[test]
    fn rejects_empty_mime() {
        assert!(!is_valid_mime(""));
    }

    // ── File size tests ───────────────────────────────────────────────────────

    #[test]
    fn accepts_small_file() {
        let size = 10 * 1024 * 1024; // 10 MB
        assert!(is_valid_size(size));
    }

    #[test]
    fn accepts_exactly_1gb() {
        assert!(is_valid_size(MAX_SIZE_BYTES));
    }

    #[test]
    fn rejects_file_over_1gb() {
        let size = MAX_SIZE_BYTES + 1;
        assert!(!is_valid_size(size));
    }

    #[test]
    fn rejects_2gb_file() {
        let size = 2 * 1024 * 1024 * 1024;
        assert!(!is_valid_size(size));
    }
}