// IsNotFound 如果指定的错误是由 NewNotFound 创建的，则返回 true。
// 它支持包装的错误，当错误为 nil 时返回 false。
fn is_not_found(err: &dyn std::error::Error) -> bool {
    let (reason, code) = reason_and_code_for_error(err);
    if reason == metav1::StatusReason::NotFound {
        return true;
    }
    if !KNOWN_REASONS.contains_key(&reason) && code == http::StatusCode::NOT_FOUND.as_u16() as i32 {
        return true;
    }
    false
}

fn reason_and_code_for_error(err: &dyn std::error::Error) -> (metav1::StatusReason, i32) {
    if let Some(status) = err.downcast_ref::<dyn APIStatus>() {
        return (status.status().reason, status.status().code);
    }
    
    // 尝试从错误链中查找 APIStatus
    let mut source = err.source();
    while let Some(e) = source {
        if let Some(status) = e.downcast_ref::<dyn APIStatus>() {
            return (status.status().reason, status.status().code);
        }
        source = e.source();
    }
    
    (metav1::StatusReason::Unknown, 0)
}

fn as_error<T: 'static>(err: &dyn std::error::Error, target: &mut &dyn std::any::Any) -> bool {
    if let Some(downcasted) = err.downcast_ref::<T>() {
        *target = downcasted;
        return true;
    }
    
    // 递归检查错误链
    let mut source = err.source();
    while let Some(e) = source {
        if let Some(downcasted) = e.downcast_ref::<T>() {
            *target = downcasted;
            return true;
        }
        source = e.source();
    }
    
    false
}
