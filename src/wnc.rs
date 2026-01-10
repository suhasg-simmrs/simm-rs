pub trait WeightsAndCorr {
    fn rw(&self, risk_class: &str, bucket: &str) -> Option<f64>;
    fn rho(&self, risk_class: &str, index1: &str, index2: &str, bucket: Option<&str>) -> Option<f64>;
    fn gamma(&self, risk_class: &str, bucket1: &str, bucket2: &str) -> Option<f64>;
    fn t(&self, risk_class: &str, risk_type: &str, currency: Option<&str>, bucket: Option<&str>) -> Option<f64>;
    fn psi(&self, risk_class1: &str, risk_class2: &str) -> Option<f64>;
}
