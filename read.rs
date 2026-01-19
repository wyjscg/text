// SecretNamespaceLister 帮助列出和获取 Secrets。
// 这里返回的所有对象必须被视为只读。
trait SecretNamespaceLister {
    // List 列出给定命名空间的索引器中的所有 Secrets。
    // 这里返回的对象必须被视为只读。
    fn list(&self, selector: labels::Selector) -> Result<Vec<&corev1::Secret>, Box<dyn std::error::Error>>;
    // Get 从给定命名空间和名称的索引器中检索 Secret。
    // 这里返回的对象必须被视为只读。
    fn get(&self, name: &str) -> Result<&corev1::Secret, Box<dyn std::error::Error>>;
}
