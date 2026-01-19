pub trait APIStatus {
    fn status(&self) -> &Status;
}

pub struct Status {
    pub type_meta: TypeMeta,
    pub metadata: Option<ListMeta>,
    pub status: Option<String>,
    pub message: Option<String>,
    pub reason: Option<StatusReason>,
    pub details: Option<StatusDetails>,
    pub code: Option<i32>,
}

pub struct TypeMeta {
    pub kind: Option<String>,
    pub api_version: Option<String>,
}

pub struct ListMeta {
    pub self_link: Option<String>,
    pub resource_version: Option<String>,
    pub continue_token: Option<String>,
    pub remaining_item_count: Option<i64>,
}
pub struct StatusDetails {
    pub name: Option<String>,
    pub group: Option<String>,
    pub kind: Option<String>,
    pub uid: Option<types::UID>,
    pub causes: Option<Vec<StatusCause>>,
    pub retry_after_seconds: Option<i32>,
}

pub struct StatusCause {
    pub type_: Option<CauseType>,
    pub message: Option<String>,
    pub field: Option<String>,
}

pub type UID = String;

pub type StatusReason = String;

pub const STATUS_REASON_UNKNOWN: &str = "";
pub const STATUS_REASON_UNAUTHORIZED: &str = "Unauthorized";
pub const STATUS_REASON_FORBIDDEN: &str = "Forbidden";
pub const STATUS_REASON_NOT_FOUND: &str = "NotFound";
pub const STATUS_REASON_ALREADY_EXISTS: &str = "AlreadyExists";
pub const STATUS_REASON_CONFLICT: &str = "Conflict";
pub const STATUS_REASON_GONE: &str = "Gone";
pub const STATUS_REASON_INVALID: &str = "Invalid";
pub const STATUS_REASON_SERVER_TIMEOUT: &str = "ServerTimeout";
pub const STATUS_REASON_STORE_READ_ERROR: &str = "StorageReadError";
pub const STATUS_REASON_TIMEOUT: &str = "Timeout";
pub const STATUS_REASON_TOO_MANY_REQUESTS: &str = "TooManyRequests";
pub const STATUS_REASON_BAD_REQUEST: &str = "BadRequest";
pub const STATUS_REASON_METHOD_NOT_ALLOWED: &str = "MethodNotAllowed";
pub const STATUS_REASON_NOT_ACCEPTABLE: &str = "NotAcceptable";
pub const STATUS_REASON_REQUEST_ENTITY_TOO_LARGE: &str = "RequestEntityTooLarge";
pub const STATUS_REASON_UNSUPPORTED_MEDIA_TYPE: &str = "UnsupportedMediaType";
pub const STATUS_REASON_INTERNAL_ERROR: &str = "InternalError";
pub const STATUS_REASON_EXPIRED: &str = "Expired";
pub const STATUS_REASON_SERVICE_UNAVAILABLE: &str = "ServiceUnavailable";


pub type CauseType = String;

pub const CAUSE_TYPE_FIELD_VALUE_NOT_FOUND: &str = "FieldValueNotFound";
pub const CAUSE_TYPE_FIELD_VALUE_REQUIRED: &str = "FieldValueRequired";
pub const CAUSE_TYPE_FIELD_VALUE_DUPLICATE: &str = "FieldValueDuplicate";
pub const CAUSE_TYPE_FIELD_VALUE_INVALID: &str = "FieldValueInvalid";
pub const CAUSE_TYPE_FIELD_VALUE_NOT_SUPPORTED: &str = "FieldValueNotSupported";
pub const CAUSE_TYPE_FORBIDDEN: &str = "FieldValueForbidden";
pub const CAUSE_TYPE_TOO_LONG: &str = "FieldValueTooLong";
pub const CAUSE_TYPE_TOO_MANY: &str = "FieldValueTooMany";
pub const CAUSE_TYPE_INTERNAL: &str = "InternalError";
pub const CAUSE_TYPE_TYPE_INVALID: &str = "FieldValueTypeInvalid";
pub const CAUSE_TYPE_UNEXPECTED_SERVER_RESPONSE: &str = "UnexpectedServerResponse";
pub const CAUSE_TYPE_FIELD_MANAGER_CONFLICT: &str = "FieldManagerConflict";
pub const CAUSE_TYPE_RESOURCE_VERSION_TOO_LARGE: &str = "ResourceVersionTooLarge";
