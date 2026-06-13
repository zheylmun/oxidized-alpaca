use crate::restful::MarketDataClient;
use reqwest::Method;

/// Builder for requesting a company logo image.
#[must_use]
pub struct LogoRequest<'a> {
    client: &'a MarketDataClient,
    symbol: String,
    placeholder: Option<bool>,
}

impl LogoRequest<'_> {
    /// Set whether the API returns a placeholder image when no logo exists
    /// (defaults to `true` server-side). Pass `false` to receive a 404
    /// instead of a placeholder.
    pub fn placeholder(mut self, placeholder: bool) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    fn build_request(&self) -> crate::Result<reqwest::RequestBuilder> {
        let symbol = &self.symbol;
        let path = format!("v1beta1/logos/{symbol}");
        let mut request = self.client.request(Method::GET, &path)?;
        if let Some(placeholder) = self.placeholder {
            request = request.query(&[("placeholder", placeholder)]);
        }
        Ok(request)
    }

    /// Fetch the logo image as raw bytes (PNG or SVG).
    pub async fn execute(self) -> crate::Result<Vec<u8>> {
        let request = self.build_request()?;
        let response = request
            .send()
            .await
            .map_err(|e| crate::Error::ReqwestSend(e.into()))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(crate::Error::ApiError {
                status: status.as_u16(),
                body,
            });
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|e| crate::Error::ReqwestDeserialize(e.into()))?;
        Ok(bytes.to_vec())
    }
}

impl MarketDataClient {
    /// Request a company logo image.
    ///
    /// Returns a builder; call [`LogoRequest::execute`] to fetch the raw
    /// image bytes.
    pub fn logo<'a>(&'a self, symbol: &str) -> LogoRequest<'a> {
        LogoRequest {
            client: self,
            symbol: symbol.to_string(),
            placeholder: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountType;
    use serial_test::serial;
    use std::env;

    fn paper_client() -> MarketDataClient {
        unsafe {
            if env::var("ALPACA_PAPER_API_KEY_ID").is_err() {
                env::set_var("ALPACA_PAPER_API_KEY_ID", "test_key_id");
            }
            if env::var("ALPACA_PAPER_API_SECRET_KEY").is_err() {
                env::set_var("ALPACA_PAPER_API_SECRET_KEY", "test_secret_key");
            }
        }
        MarketDataClient::new(AccountType::Paper).unwrap()
    }

    #[test]
    #[serial]
    fn placeholder_serializes_to_query() {
        let client = paper_client();
        let request = client
            .logo("AAPL")
            .placeholder(false)
            .build_request()
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(request.url().path(), "/v1beta1/logos/AAPL");
        assert_eq!(request.url().query(), Some("placeholder=false"));
    }

    #[test]
    #[serial]
    fn placeholder_omitted_by_default() {
        let client = paper_client();
        let request = client
            .logo("AAPL")
            .build_request()
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(request.url().query(), None);
    }
}
