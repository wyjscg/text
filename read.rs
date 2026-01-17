secret
use k8s_openapi::api::core::v1::Secret;
use kube::api::{Api, ListParams};
use kube::Client;

// SecretNamespaceLister 帮助列出和获取 Secrets。
// 此处返回的所有对象必须被视为只读。
pub struct SecretNamespaceLister {
    api: Api<Secret>,
}

impl SecretNamespaceLister {
    pub fn new(client: Client, namespace: &str) -> Self {
        Self {
            api: Api::namespaced(client, namespace),
        }
    }

    // List 列出给定命名空间的索引器中的所有 Secrets。
    // 此处返回的对象必须被视为只读。
    pub async fn list(&self, selector: &str) -> Result<Vec<Secret>, kube::Error> {
        let lp = if selector.is_empty() {
            ListParams::default()
        } else {
            ListParams::default().labels(selector)
        };
        
        let secret_list = self.api.list(&lp).await?;
        Ok(secret_list.items)
    }

    // Get 从给定命名空间和名称的索引器中检索 Secret。
    // 此处返回的对象必须被视为只读。
    pub async fn get(&self, name: &str) -> Result<Secret, kube::Error> {
        self.api.get(name).await
    }
}




---------------------
labels
// Labels 允许你独立于其存储方式来呈现标签。
pub trait Labels {
    // Has 返回提供的标签是否存在。
    fn has(&self, label: &str) -> bool;

    // Get 返回提供的标签的值。
    fn get(&self, label: &str) -> String;

    // Lookup 返回提供的标签的值（如果存在）以及提供的标签是否存在
    fn lookup(&self, label: &str) -> (String, bool);
}



---------------------------------------------------

selector

// Selector 表示一个标签选择器。
pub trait Selector {
    // Matches 如果此选择器匹配给定的标签集，则返回 true。
    fn matches(&self, labels: &dyn Labels) -> bool;

    // Empty 如果此选择器不限制选择空间，则返回 true。
    fn empty(&self) -> bool;

    // String 返回表示此选择器的人类可读字符串。
    fn to_string(&self) -> String;

    // Add 向 Selector 添加要求
    fn add(&self, requirements: &[Requirement]) -> Box<dyn Selector>;

    // Requirements 将此接口转换为 Requirements 以公开更详细的选择信息。
    // 如果有查询参数，它将返回转换后的要求和 selectable=true。
    // 如果此选择器不想选择任何内容，它将返回 selectable=false。
    fn requirements(&self) -> (Vec<Requirement>, bool);

    // Make a deep copy of the selector.
    fn deep_copy_selector(&self) -> Box<dyn Selector>;

    // RequiresExactMatch 允许调用者检查给定的选择器是否需要设置单个特定标签，
    // 如果是，则返回它所需的值。
    fn requires_exact_match(&self, label: &str) -> (String, bool);
}

-----------------------------------------------------
requirement-selector
// Requirement 包含值、键以及关联键和值的运算符。
// Requirement 的零值是无效的。
// Requirement 实现了基于集合的匹配和精确匹配
// 应该通过 NewRequirement 构造函数初始化 Requirement 以创建有效的 Requirement。
// +k8s:deepcopy-gen=true
#[derive(Clone, Debug)]
pub struct Requirement {
    key: String,
    operator: Operator,
    // 在绝大多数情况下，我们这里最多有一个值。
    // 通常对单元素切片的操作比对单元素映射更快，
    // 所以我们这里使用切片。
    str_values: Vec<String>,
}


-----------------------------------------------------
Operator

/*
Copyright 2016 The Kubernetes Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

// Operator 表示键/字段与值的关系。
// 有关更多详细信息，请参见 labels.Requirement 和 fields.Requirement。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operator {
    DoesNotExist,  // "!"
    Equals,        // "="
    DoubleEquals,  // "=="
    In,            // "in"
    NotEquals,     // "!="
    NotIn,         // "notin"
    Exists,        // "exists"
    GreaterThan,   // "gt"
    LessThan,      // "lt"
}

impl Operator {
    pub fn as_str(&self) -> &'static str {
        match self {
            Operator::DoesNotExist => "!",
            Operator::Equals => "=",
            Operator::DoubleEquals => "==",
            Operator::In => "in",
            Operator::NotEquals => "!=",
            Operator::NotIn => "notin",
            Operator::Exists => "exists",
            Operator::GreaterThan => "gt",
            Operator::LessThan => "lt",
        }
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
