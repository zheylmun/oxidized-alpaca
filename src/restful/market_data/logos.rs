use crate::restful::MarketDataClient;
use reqwest::Method;

impl MarketDataClient {
    /// Get a company logo image as raw bytes (PNG or SVG).
    pub async fn logo(&self, symbol: &str) -> crate::Result<Vec<u8>> {
        let path = format!("v1beta1/logos/{symbol}");
        let request = self.request(Method::GET, &path)?;
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
