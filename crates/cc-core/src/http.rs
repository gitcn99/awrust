//! HTTP 客户端封装，基于 `reqwest`，提供便捷的微服务间调用能力。
//!
//! # 示例
//!
//! ```rust,no_run
//! use cc_core::http::HttpClient;
//! use std::time::Duration;
//!
//! # async fn example() {
//! let client = HttpClient::builder()
//!     .base_url("https://api.example.com")
//!     .timeout(Duration::from_secs(30))
//!     .default_header("X-Api-Key", "my-key")
//!     .build()
//!     .unwrap();
//!
//! let resp = client.get("/users").send().await.unwrap();
//! # }
//! ```

use std::time::Duration;

use crate::error::{ConfigResult, Error};

/// HTTP 客户端，基于 `reqwest::Client` 的薄封装。
///
/// 支持 `base_url` 自动拼接、超时配置、默认请求头等。
/// 所有请求方法返回 `reqwest::RequestBuilder`，保持 reqwest 原生 API 的灵活性。
#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    base_url: Option<String>,
}

impl HttpClient {
    /// 创建 `HttpClientBuilder`。
    pub fn builder() -> HttpClientBuilder {
        HttpClientBuilder::new()
    }

    /// 获取底层 `reqwest::Client` 引用。
    pub fn inner(&self) -> &reqwest::Client {
        &self.inner
    }

    /// 消费自身，返回底层 `reqwest::Client`。
    pub fn into_inner(self) -> reqwest::Client {
        self.inner
    }

    /// 获取配置的基础 URL。
    pub fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }

    /// 拼接完整 URL：如果配置了 `base_url` 且 `url` 是相对路径，则自动拼接。
    fn resolve_url(&self, url: &str) -> String {
        if let Some(base) = &self.base_url {
            // 如果 url 已经是绝对 URL，直接使用
            if url.starts_with("http://") || url.starts_with("https://") {
                return url.to_string();
            }
            // 拼接 base_url 和路径
            let base = base.trim_end_matches('/');
            let path = url.trim_start_matches('/');
            format!("{base}/{path}")
        } else {
            url.to_string()
        }
    }

    /// 发送 GET 请求。
    pub fn get(&self, url: &str) -> reqwest::RequestBuilder {
        self.inner.get(self.resolve_url(url))
    }

    /// 发送 POST 请求。
    pub fn post(&self, url: &str) -> reqwest::RequestBuilder {
        self.inner.post(self.resolve_url(url))
    }

    /// 发送 PUT 请求。
    pub fn put(&self, url: &str) -> reqwest::RequestBuilder {
        self.inner.put(self.resolve_url(url))
    }

    /// 发送 DELETE 请求。
    pub fn delete(&self, url: &str) -> reqwest::RequestBuilder {
        self.inner.delete(self.resolve_url(url))
    }

    /// 发送 PATCH 请求。
    pub fn patch(&self, url: &str) -> reqwest::RequestBuilder {
        self.inner.patch(self.resolve_url(url))
    }

    /// 发送 HEAD 请求。
    pub fn head(&self, url: &str) -> reqwest::RequestBuilder {
        self.inner.head(self.resolve_url(url))
    }
}

/// HTTP 客户端构建器。
pub struct HttpClientBuilder {
    base_url: Option<String>,
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    default_headers: Vec<(String, String)>,
}

impl Default for HttpClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClientBuilder {
    pub fn new() -> Self {
        Self {
            base_url: None,
            timeout: None,
            connect_timeout: None,
            default_headers: Vec::new(),
        }
    }

    /// 设置基础 URL，后续请求的相对路径将自动拼接。
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// 设置请求超时时间。
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// 设置连接超时时间。
    pub fn connect_timeout(mut self, duration: Duration) -> Self {
        self.connect_timeout = Some(duration);
        self
    }

    /// 添加默认请求头，所有请求都会携带。
    pub fn default_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.push((key.into(), value.into()));
        self
    }

    /// 构造 `HttpClient`。
    pub fn build(self) -> ConfigResult<HttpClient> {
        let mut builder = reqwest::Client::builder();

        if let Some(timeout) = self.timeout {
            builder = builder.timeout(timeout);
        }

        if let Some(connect_timeout) = self.connect_timeout {
            builder = builder.connect_timeout(connect_timeout);
        }

        if !self.default_headers.is_empty() {
            let mut headers = reqwest::header::HeaderMap::new();
            for (key, value) in &self.default_headers {
                let name =
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
                        Error::HttpClientCreate {
                            message: format!("无效的请求头名称 '{key}': {e}"),
                        }
                    })?;
                let val = reqwest::header::HeaderValue::from_str(value).map_err(|e| {
                    Error::HttpClientCreate {
                        message: format!("无效的请求头值 '{value}': {e}"),
                    }
                })?;
                headers.insert(name, val);
            }
            builder = builder.default_headers(headers);
        }

        let inner = builder.build().map_err(|e| Error::HttpClientCreate {
            message: e.to_string(),
        })?;

        tracing::info!(
            base_url = ?self.base_url,
            "HTTP 客户端创建成功"
        );

        Ok(HttpClient {
            inner,
            base_url: self.base_url,
        })
    }
}
