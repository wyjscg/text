// IsNotFound 如果指定的错误是由 NewNotFound 创建的，则返回 true。
// 它支持包装的错误，当错误为 nil 时返回 false。
fn is_not_found(err: &dyn std::error::Error) -> bool {
    let (reason, code) = reason_and_code_for_error(err);
    if reason == metav1::StatusReason::NotFound {
        return true;
    }
    if !KNOWN_REASONS.contains_key(&reason) && code == http::StatusCode::NOT_FOUND {
        return true;
    }
    false
}

fn as_error<T>(err: &dyn std::error::Error, target: &mut Option<&T>) -> bool 
where
    T: std::error::Error + 'static,
{
    if let Some(downcasted) = err.downcast_ref::<T>() {
        *target = Some(downcasted);
        return true;
    }
    
    // 递归检查源错误
    if let Some(source) = err.source() {
        return as_error(source, target);
    }
    
    false
}

fn reason_for_error(err: &dyn std::error::Error) -> metav1::StatusReason {
    let mut status: Option<&dyn APIStatus> = None;
    if as_error(err, &mut status) {
        if let Some(s) = status {
            return s.status().reason;
        }
    }
    metav1::StatusReason::Unknown
}
