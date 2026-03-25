use fpt_core::{CommandSpec, RiskLevel};

const UPLOAD_URL_EXAMPLES: &[&str] = &[
    "fpt upload url Shot 123 sg_uploaded_movie movie.mp4 --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt upload url Version 456 sg_uploaded_movie render.mov --content-type video/quicktime --site ...",
    "fpt upload url Version 456 sg_uploaded_movie large_file.mov --multipart --site ...",
];

const DOWNLOAD_URL_EXAMPLES: &[&str] = &[
    "fpt download url Shot 123 sg_uploaded_movie --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt download url Attachment 789 this_file --site ...",
];

const THUMBNAIL_URL_EXAMPLES: &[&str] = &[
    "fpt thumbnail url Asset 55 --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt thumbnail url Shot 100 --site ...",
];

const UPLOAD_NOTES: &[&str] = &[
    "Returns a pre-signed upload URL and upload parameters from ShotGrid",
    "Use the returned URL to PUT/POST the file bytes directly to storage",
    "Pass --multipart for large files to receive a multipart upload URL",
    "content-type defaults to application/octet-stream if not specified",
];

const DOWNLOAD_NOTES: &[&str] = &[
    "Returns a pre-signed download URL for the specified entity field attachment",
    "The URL is time-limited; fetch the file immediately after calling this command",
];

const THUMBNAIL_NOTES: &[&str] = &[
    "Returns the thumbnail image URL for the specified entity record",
    "Uses the REST endpoint GET /entity/{type}/{id}/image",
];

pub const UPLOAD_URL_SPEC: CommandSpec = CommandSpec {
    name: "upload.url",
    summary: "Get a pre-signed upload URL for an entity field attachment",
    risk: RiskLevel::Write,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id + field_name + file_name",
    output: "json upload parameters",
    examples: UPLOAD_URL_EXAMPLES,
    notes: UPLOAD_NOTES,
};

pub const DOWNLOAD_URL_SPEC: CommandSpec = CommandSpec {
    name: "download.url",
    summary: "Get a pre-signed download URL for an entity field attachment",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id + field_name",
    output: "json with download_url",
    examples: DOWNLOAD_URL_EXAMPLES,
    notes: DOWNLOAD_NOTES,
};

pub const THUMBNAIL_URL_SPEC: CommandSpec = CommandSpec {
    name: "thumbnail.url",
    summary: "Get the thumbnail image URL for an entity record",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id",
    output: "json with thumbnail_url",
    examples: THUMBNAIL_URL_EXAMPLES,
    notes: THUMBNAIL_NOTES,
};

const FILMSTRIP_URL_EXAMPLES: &[&str] = &[
    "fpt filmstrip url Version 456 --site ... --auth-mode script --script-name ... --script-key ...",
    "fpt filmstrip url Shot 123 --site ...",
];

const FILMSTRIP_NOTES: &[&str] = &[
    "Returns the filmstrip thumbnail image URL for the specified entity record",
    "Uses the REST endpoint GET /entity/{type}/{id}/filmstrip_image",
    "Filmstrip thumbnails are typically available on Version and Shot entities",
];

pub const FILMSTRIP_URL_SPEC: CommandSpec = CommandSpec {
    name: "filmstrip.url",
    summary: "Get the filmstrip thumbnail image URL for an entity record",
    risk: RiskLevel::Read,
    implemented: true,
    supports_dry_run: false,
    preferred_transport: "rest",
    fallback_transport: None,
    input: "entity + id",
    output: "json with filmstrip_url",
    examples: FILMSTRIP_URL_EXAMPLES,
    notes: FILMSTRIP_NOTES,
};
