#![allow(unused_qualifications)]

use crate::core_api::generated::models;

use crate::core_api::generated::header;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CommittedTransaction {
    /// The resultant state version after the txn has been committed. A decimal 64-bit unsigned integer.
    #[serde(rename = "state_version")]
    pub state_version: String,

    #[serde(rename = "notarized_transaction")]
    pub notarized_transaction: models::NotarizedTransaction,

    #[serde(rename = "receipt")]
    pub receipt: models::TransactionReceipt,

}

impl CommittedTransaction {
    pub fn new(state_version: String, notarized_transaction: models::NotarizedTransaction, receipt: models::TransactionReceipt, ) -> CommittedTransaction {
        CommittedTransaction {
            state_version: state_version,
            notarized_transaction: notarized_transaction,
            receipt: receipt,
        }
    }
}

/// Converts the CommittedTransaction value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CommittedTransaction {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("state_version".to_string());
        params.push(self.state_version.to_string());

        // Skipping notarized_transaction in query parameter serialization

        // Skipping receipt in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CommittedTransaction value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CommittedTransaction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub state_version: Vec<String>,
            pub notarized_transaction: Vec<models::NotarizedTransaction>,
            pub receipt: Vec<models::TransactionReceipt>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing CommittedTransaction".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "state_version" => intermediate_rep.state_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "notarized_transaction" => intermediate_rep.notarized_transaction.push(<models::NotarizedTransaction as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "receipt" => intermediate_rep.receipt.push(<models::TransactionReceipt as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing CommittedTransaction".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CommittedTransaction {
            state_version: intermediate_rep.state_version.into_iter().next().ok_or("state_version missing in CommittedTransaction".to_string())?,
            notarized_transaction: intermediate_rep.notarized_transaction.into_iter().next().ok_or("notarized_transaction missing in CommittedTransaction".to_string())?,
            receipt: intermediate_rep.receipt.into_iter().next().ok_or("receipt missing in CommittedTransaction".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CommittedTransaction> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<CommittedTransaction>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<CommittedTransaction>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for CommittedTransaction - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CommittedTransaction> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <CommittedTransaction as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into CommittedTransaction - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// A request to retrieve a sublist of committed transactions from the ledger.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CommittedTransactionsRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// A decimal 64-bit unsigned integer.
    #[serde(rename = "start_state_version")]
    pub start_state_version: String,

    /// The maximum number of transactions that will be returned.
    #[serde(rename = "limit")]
    pub limit: isize,

}

impl CommittedTransactionsRequest {
    pub fn new(network_identifier: models::NetworkIdentifier, start_state_version: String, limit: isize, ) -> CommittedTransactionsRequest {
        CommittedTransactionsRequest {
            network_identifier: network_identifier,
            start_state_version: start_state_version,
            limit: limit,
        }
    }
}

/// Converts the CommittedTransactionsRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CommittedTransactionsRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization


        params.push("start_state_version".to_string());
        params.push(self.start_state_version.to_string());


        params.push("limit".to_string());
        params.push(self.limit.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CommittedTransactionsRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CommittedTransactionsRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub start_state_version: Vec<String>,
            pub limit: Vec<isize>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing CommittedTransactionsRequest".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "start_state_version" => intermediate_rep.start_state_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "limit" => intermediate_rep.limit.push(<isize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing CommittedTransactionsRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CommittedTransactionsRequest {
            network_identifier: intermediate_rep.network_identifier.into_iter().next().ok_or("network_identifier missing in CommittedTransactionsRequest".to_string())?,
            start_state_version: intermediate_rep.start_state_version.into_iter().next().ok_or("start_state_version missing in CommittedTransactionsRequest".to_string())?,
            limit: intermediate_rep.limit.into_iter().next().ok_or("limit missing in CommittedTransactionsRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CommittedTransactionsRequest> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<CommittedTransactionsRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<CommittedTransactionsRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for CommittedTransactionsRequest - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CommittedTransactionsRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <CommittedTransactionsRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into CommittedTransactionsRequest - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CommittedTransactionsResponse {
    /// The first state version returned. A decimal 64-bit unsigned integer.
    #[serde(rename = "start_state_version")]
    pub start_state_version: String,

    /// The maximum state version returned. A decimal 64-bit unsigned integer.
    #[serde(rename = "max_state_version")]
    pub max_state_version: String,

    /// A committed transactions list starting from the `start_state_version_inclusive`.
    #[serde(rename = "transactions")]
    pub transactions: Vec<models::CommittedTransaction>,

}

impl CommittedTransactionsResponse {
    pub fn new(start_state_version: String, max_state_version: String, transactions: Vec<models::CommittedTransaction>, ) -> CommittedTransactionsResponse {
        CommittedTransactionsResponse {
            start_state_version: start_state_version,
            max_state_version: max_state_version,
            transactions: transactions,
        }
    }
}

/// Converts the CommittedTransactionsResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CommittedTransactionsResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("start_state_version".to_string());
        params.push(self.start_state_version.to_string());


        params.push("max_state_version".to_string());
        params.push(self.max_state_version.to_string());

        // Skipping transactions in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CommittedTransactionsResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CommittedTransactionsResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub start_state_version: Vec<String>,
            pub max_state_version: Vec<String>,
            pub transactions: Vec<Vec<models::CommittedTransaction>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing CommittedTransactionsResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "start_state_version" => intermediate_rep.start_state_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "max_state_version" => intermediate_rep.max_state_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "transactions" => return std::result::Result::Err("Parsing a container in this style is not supported in CommittedTransactionsResponse".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing CommittedTransactionsResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CommittedTransactionsResponse {
            start_state_version: intermediate_rep.start_state_version.into_iter().next().ok_or("start_state_version missing in CommittedTransactionsResponse".to_string())?,
            max_state_version: intermediate_rep.max_state_version.into_iter().next().ok_or("max_state_version missing in CommittedTransactionsResponse".to_string())?,
            transactions: intermediate_rep.transactions.into_iter().next().ok_or("transactions missing in CommittedTransactionsResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CommittedTransactionsResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<CommittedTransactionsResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<CommittedTransactionsResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for CommittedTransactionsResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CommittedTransactionsResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <CommittedTransactionsResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into CommittedTransactionsResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ErrorResponse {
    /// A numeric code corresponding to the given error type.
    #[serde(rename = "code")]
    pub code: isize,

    /// A human-readable error message.
    #[serde(rename = "message")]
    pub message: String,

    /// A GUID to be used when reporting errors, to allow correlation with the Core API's error logs, in the case where the Core API details are hidden.
    #[serde(rename = "trace_id")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub trace_id: Option<String>,

}

impl ErrorResponse {
    pub fn new(code: isize, message: String, ) -> ErrorResponse {
        ErrorResponse {
            code: code,
            message: message,
            trace_id: None,
        }
    }
}

/// Converts the ErrorResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ErrorResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("code".to_string());
        params.push(self.code.to_string());


        params.push("message".to_string());
        params.push(self.message.to_string());


        if let Some(ref trace_id) = self.trace_id {
            params.push("trace_id".to_string());
            params.push(trace_id.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ErrorResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ErrorResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub code: Vec<isize>,
            pub message: Vec<String>,
            pub trace_id: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ErrorResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "code" => intermediate_rep.code.push(<isize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "trace_id" => intermediate_rep.trace_id.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ErrorResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ErrorResponse {
            code: intermediate_rep.code.into_iter().next().ok_or("code missing in ErrorResponse".to_string())?,
            message: intermediate_rep.message.into_iter().next().ok_or("message missing in ErrorResponse".to_string())?,
            trace_id: intermediate_rep.trace_id.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ErrorResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<ErrorResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ErrorResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for ErrorResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ErrorResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ErrorResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into ErrorResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// Fees paid
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FeeSummary {
    /// Specifies whether the transaction execution loan has been fully repaid.
    #[serde(rename = "loan_fully_repaid")]
    pub loan_fully_repaid: bool,

    /// Maximum amount of cost units available for the transaction execution. A decimal 32-bit unsigned integer.
    #[serde(rename = "cost_unit_limit")]
    pub cost_unit_limit: String,

    /// The amount of cost units consumed by the transaction execution. A decimal 32-bit unsigned integer.
    #[serde(rename = "cost_unit_consumed")]
    pub cost_unit_consumed: String,

    /// The XRD price of a single cost unit. A fixed-scale 256-bit decimal number.
    #[serde(rename = "cost_unit_price")]
    pub cost_unit_price: String,

    /// The validator tip. A decimal 32-bit unsigned integer, representing the percentage amount (a value of \"1\" corresponds to 1%).
    #[serde(rename = "tip_percentage")]
    pub tip_percentage: String,

    /// The total amount of XRD burned. A fixed-scale 256-bit decimal number.
    #[serde(rename = "xrd_burned")]
    pub xrd_burned: String,

    /// The total amount of XRD tipped to validators. A fixed-scale 256-bit decimal number.
    #[serde(rename = "xrd_tipped")]
    pub xrd_tipped: String,

}

impl FeeSummary {
    pub fn new(loan_fully_repaid: bool, cost_unit_limit: String, cost_unit_consumed: String, cost_unit_price: String, tip_percentage: String, xrd_burned: String, xrd_tipped: String, ) -> FeeSummary {
        FeeSummary {
            loan_fully_repaid: loan_fully_repaid,
            cost_unit_limit: cost_unit_limit,
            cost_unit_consumed: cost_unit_consumed,
            cost_unit_price: cost_unit_price,
            tip_percentage: tip_percentage,
            xrd_burned: xrd_burned,
            xrd_tipped: xrd_tipped,
        }
    }
}

/// Converts the FeeSummary value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for FeeSummary {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("loan_fully_repaid".to_string());
        params.push(self.loan_fully_repaid.to_string());


        params.push("cost_unit_limit".to_string());
        params.push(self.cost_unit_limit.to_string());


        params.push("cost_unit_consumed".to_string());
        params.push(self.cost_unit_consumed.to_string());


        params.push("cost_unit_price".to_string());
        params.push(self.cost_unit_price.to_string());


        params.push("tip_percentage".to_string());
        params.push(self.tip_percentage.to_string());


        params.push("xrd_burned".to_string());
        params.push(self.xrd_burned.to_string());


        params.push("xrd_tipped".to_string());
        params.push(self.xrd_tipped.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FeeSummary value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FeeSummary {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub loan_fully_repaid: Vec<bool>,
            pub cost_unit_limit: Vec<String>,
            pub cost_unit_consumed: Vec<String>,
            pub cost_unit_price: Vec<String>,
            pub tip_percentage: Vec<String>,
            pub xrd_burned: Vec<String>,
            pub xrd_tipped: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing FeeSummary".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "loan_fully_repaid" => intermediate_rep.loan_fully_repaid.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cost_unit_limit" => intermediate_rep.cost_unit_limit.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cost_unit_consumed" => intermediate_rep.cost_unit_consumed.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cost_unit_price" => intermediate_rep.cost_unit_price.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "tip_percentage" => intermediate_rep.tip_percentage.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "xrd_burned" => intermediate_rep.xrd_burned.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "xrd_tipped" => intermediate_rep.xrd_tipped.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing FeeSummary".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FeeSummary {
            loan_fully_repaid: intermediate_rep.loan_fully_repaid.into_iter().next().ok_or("loan_fully_repaid missing in FeeSummary".to_string())?,
            cost_unit_limit: intermediate_rep.cost_unit_limit.into_iter().next().ok_or("cost_unit_limit missing in FeeSummary".to_string())?,
            cost_unit_consumed: intermediate_rep.cost_unit_consumed.into_iter().next().ok_or("cost_unit_consumed missing in FeeSummary".to_string())?,
            cost_unit_price: intermediate_rep.cost_unit_price.into_iter().next().ok_or("cost_unit_price missing in FeeSummary".to_string())?,
            tip_percentage: intermediate_rep.tip_percentage.into_iter().next().ok_or("tip_percentage missing in FeeSummary".to_string())?,
            xrd_burned: intermediate_rep.xrd_burned.into_iter().next().ok_or("xrd_burned missing in FeeSummary".to_string())?,
            xrd_tipped: intermediate_rep.xrd_tipped.into_iter().next().ok_or("xrd_tipped missing in FeeSummary".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FeeSummary> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<FeeSummary>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<FeeSummary>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for FeeSummary - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<FeeSummary> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <FeeSummary as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into FeeSummary - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct IntentSignature {
    /// Signer public key, hex-encoded.
    #[serde(rename = "public_key")]
    pub public_key: String,

    /// The signature, hex-encoded.
    #[serde(rename = "signature")]
    pub signature: String,

}

impl IntentSignature {
    pub fn new(public_key: String, signature: String, ) -> IntentSignature {
        IntentSignature {
            public_key: public_key,
            signature: signature,
        }
    }
}

/// Converts the IntentSignature value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for IntentSignature {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("public_key".to_string());
        params.push(self.public_key.to_string());


        params.push("signature".to_string());
        params.push(self.signature.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a IntentSignature value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for IntentSignature {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub public_key: Vec<String>,
            pub signature: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing IntentSignature".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "public_key" => intermediate_rep.public_key.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "signature" => intermediate_rep.signature.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing IntentSignature".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(IntentSignature {
            public_key: intermediate_rep.public_key.into_iter().next().ok_or("public_key missing in IntentSignature".to_string())?,
            signature: intermediate_rep.signature.into_iter().next().ok_or("signature missing in IntentSignature".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<IntentSignature> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<IntentSignature>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<IntentSignature>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for IntentSignature - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<IntentSignature> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <IntentSignature as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into IntentSignature - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkConfigurationResponse {
    #[serde(rename = "version")]
    pub version: models::NetworkConfigurationResponseVersion,

    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// The network suffix used for bech32 hrps used for addressing.
    #[serde(rename = "network_hrp_suffix")]
    pub network_hrp_suffix: String,

}

impl NetworkConfigurationResponse {
    pub fn new(version: models::NetworkConfigurationResponseVersion, network_identifier: models::NetworkIdentifier, network_hrp_suffix: String, ) -> NetworkConfigurationResponse {
        NetworkConfigurationResponse {
            version: version,
            network_identifier: network_identifier,
            network_hrp_suffix: network_hrp_suffix,
        }
    }
}

/// Converts the NetworkConfigurationResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkConfigurationResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping version in query parameter serialization

        // Skipping network_identifier in query parameter serialization


        params.push("network_hrp_suffix".to_string());
        params.push(self.network_hrp_suffix.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkConfigurationResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkConfigurationResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub version: Vec<models::NetworkConfigurationResponseVersion>,
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub network_hrp_suffix: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkConfigurationResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "version" => intermediate_rep.version.push(<models::NetworkConfigurationResponseVersion as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "network_hrp_suffix" => intermediate_rep.network_hrp_suffix.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkConfigurationResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkConfigurationResponse {
            version: intermediate_rep.version.into_iter().next().ok_or("version missing in NetworkConfigurationResponse".to_string())?,
            network_identifier: intermediate_rep.network_identifier.into_iter().next().ok_or("network_identifier missing in NetworkConfigurationResponse".to_string())?,
            network_hrp_suffix: intermediate_rep.network_hrp_suffix.into_iter().next().ok_or("network_hrp_suffix missing in NetworkConfigurationResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkConfigurationResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkConfigurationResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkConfigurationResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkConfigurationResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkConfigurationResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkConfigurationResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkConfigurationResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// Different versions regarding the node, network and api.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkConfigurationResponseVersion {
    #[serde(rename = "core_version")]
    pub core_version: String,

    #[serde(rename = "api_version")]
    pub api_version: String,

}

impl NetworkConfigurationResponseVersion {
    pub fn new(core_version: String, api_version: String, ) -> NetworkConfigurationResponseVersion {
        NetworkConfigurationResponseVersion {
            core_version: core_version,
            api_version: api_version,
        }
    }
}

/// Converts the NetworkConfigurationResponseVersion value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkConfigurationResponseVersion {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("core_version".to_string());
        params.push(self.core_version.to_string());


        params.push("api_version".to_string());
        params.push(self.api_version.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkConfigurationResponseVersion value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkConfigurationResponseVersion {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub core_version: Vec<String>,
            pub api_version: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkConfigurationResponseVersion".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "core_version" => intermediate_rep.core_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "api_version" => intermediate_rep.api_version.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkConfigurationResponseVersion".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkConfigurationResponseVersion {
            core_version: intermediate_rep.core_version.into_iter().next().ok_or("core_version missing in NetworkConfigurationResponseVersion".to_string())?,
            api_version: intermediate_rep.api_version.into_iter().next().ok_or("api_version missing in NetworkConfigurationResponseVersion".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkConfigurationResponseVersion> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkConfigurationResponseVersion>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkConfigurationResponseVersion>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkConfigurationResponseVersion - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkConfigurationResponseVersion> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkConfigurationResponseVersion as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkConfigurationResponseVersion - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// The name of the network.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NetworkIdentifier {
    #[serde(rename = "network")]
    pub network: String,

}

impl NetworkIdentifier {
    pub fn new(network: String, ) -> NetworkIdentifier {
        NetworkIdentifier {
            network: network,
        }
    }
}

/// Converts the NetworkIdentifier value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkIdentifier {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("network".to_string());
        params.push(self.network.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkIdentifier value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkIdentifier {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkIdentifier".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "network" => intermediate_rep.network.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkIdentifier".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkIdentifier {
            network: intermediate_rep.network.into_iter().next().ok_or("network missing in NetworkIdentifier".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkIdentifier> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkIdentifier>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkIdentifier>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkIdentifier - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkIdentifier> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkIdentifier as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkIdentifier - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct NotarizedTransaction {
    /// The transaction hash, hex-encoded.
    #[serde(rename = "hash")]
    pub hash: String,

    #[serde(rename = "signed_intent")]
    pub signed_intent: models::SignedTransactionIntent,

    /// The notary signature, hex-encoded.
    #[serde(rename = "notary_signature")]
    pub notary_signature: String,

}

impl NotarizedTransaction {
    pub fn new(hash: String, signed_intent: models::SignedTransactionIntent, notary_signature: String, ) -> NotarizedTransaction {
        NotarizedTransaction {
            hash: hash,
            signed_intent: signed_intent,
            notary_signature: notary_signature,
        }
    }
}

/// Converts the NotarizedTransaction value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NotarizedTransaction {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("hash".to_string());
        params.push(self.hash.to_string());

        // Skipping signed_intent in query parameter serialization


        params.push("notary_signature".to_string());
        params.push(self.notary_signature.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NotarizedTransaction value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NotarizedTransaction {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub hash: Vec<String>,
            pub signed_intent: Vec<models::SignedTransactionIntent>,
            pub notary_signature: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NotarizedTransaction".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "hash" => intermediate_rep.hash.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "signed_intent" => intermediate_rep.signed_intent.push(<models::SignedTransactionIntent as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "notary_signature" => intermediate_rep.notary_signature.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing NotarizedTransaction".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NotarizedTransaction {
            hash: intermediate_rep.hash.into_iter().next().ok_or("hash missing in NotarizedTransaction".to_string())?,
            signed_intent: intermediate_rep.signed_intent.into_iter().next().ok_or("signed_intent missing in NotarizedTransaction".to_string())?,
            notary_signature: intermediate_rep.notary_signature.into_iter().next().ok_or("notary_signature missing in NotarizedTransaction".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NotarizedTransaction> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NotarizedTransaction>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NotarizedTransaction>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NotarizedTransaction - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NotarizedTransaction> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NotarizedTransaction as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NotarizedTransaction - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct SignedTransactionIntent {
    /// Signed transaction intent hash, hex-encoded.
    #[serde(rename = "hash")]
    pub hash: String,

    #[serde(rename = "intent")]
    pub intent: models::TransactionIntent,

    #[serde(rename = "intent_signatures")]
    pub intent_signatures: Vec<models::IntentSignature>,

}

impl SignedTransactionIntent {
    pub fn new(hash: String, intent: models::TransactionIntent, intent_signatures: Vec<models::IntentSignature>, ) -> SignedTransactionIntent {
        SignedTransactionIntent {
            hash: hash,
            intent: intent,
            intent_signatures: intent_signatures,
        }
    }
}

/// Converts the SignedTransactionIntent value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SignedTransactionIntent {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("hash".to_string());
        params.push(self.hash.to_string());

        // Skipping intent in query parameter serialization

        // Skipping intent_signatures in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SignedTransactionIntent value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SignedTransactionIntent {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub hash: Vec<String>,
            pub intent: Vec<models::TransactionIntent>,
            pub intent_signatures: Vec<Vec<models::IntentSignature>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing SignedTransactionIntent".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "hash" => intermediate_rep.hash.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "intent" => intermediate_rep.intent.push(<models::TransactionIntent as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "intent_signatures" => return std::result::Result::Err("Parsing a container in this style is not supported in SignedTransactionIntent".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing SignedTransactionIntent".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SignedTransactionIntent {
            hash: intermediate_rep.hash.into_iter().next().ok_or("hash missing in SignedTransactionIntent".to_string())?,
            intent: intermediate_rep.intent.into_iter().next().ok_or("intent missing in SignedTransactionIntent".to_string())?,
            intent_signatures: intermediate_rep.intent_signatures.into_iter().next().ok_or("intent_signatures missing in SignedTransactionIntent".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SignedTransactionIntent> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<SignedTransactionIntent>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<SignedTransactionIntent>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for SignedTransactionIntent - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<SignedTransactionIntent> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <SignedTransactionIntent as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into SignedTransactionIntent - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionHeader {
    #[serde(rename = "version")]
    pub version: isize,

    #[serde(rename = "network_id")]
    pub network_id: isize,

    /// A decimal 64-bit unsigned integer.
    #[serde(rename = "start_epoch_inclusive")]
    pub start_epoch_inclusive: String,

    /// A decimal 64-bit unsigned integer.
    #[serde(rename = "end_epoch_exclusive")]
    pub end_epoch_exclusive: String,

    /// A decimal 64-bit unsigned integer.
    #[serde(rename = "nonce")]
    pub nonce: String,

    /// A hex-encoded public key of a notary.
    #[serde(rename = "notary_public_key")]
    pub notary_public_key: String,

    /// Specifies whether the notary's signature should be included in transaction signers list
    #[serde(rename = "notary_as_signatory")]
    pub notary_as_signatory: bool,

    /// Maximum number of cost units available for transaction execution. A decimal 32-bit unsigned integer.
    #[serde(rename = "cost_unit_limit")]
    pub cost_unit_limit: String,

    /// Specifies the validator tip. A decimal 32-bit unsigned integer, representing the percentage amount (a value of \"1\" corresponds to 1%).
    #[serde(rename = "tip_percentage")]
    pub tip_percentage: String,

}

impl TransactionHeader {
    pub fn new(version: isize, network_id: isize, start_epoch_inclusive: String, end_epoch_exclusive: String, nonce: String, notary_public_key: String, notary_as_signatory: bool, cost_unit_limit: String, tip_percentage: String, ) -> TransactionHeader {
        TransactionHeader {
            version: version,
            network_id: network_id,
            start_epoch_inclusive: start_epoch_inclusive,
            end_epoch_exclusive: end_epoch_exclusive,
            nonce: nonce,
            notary_public_key: notary_public_key,
            notary_as_signatory: notary_as_signatory,
            cost_unit_limit: cost_unit_limit,
            tip_percentage: tip_percentage,
        }
    }
}

/// Converts the TransactionHeader value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionHeader {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("version".to_string());
        params.push(self.version.to_string());


        params.push("network_id".to_string());
        params.push(self.network_id.to_string());


        params.push("start_epoch_inclusive".to_string());
        params.push(self.start_epoch_inclusive.to_string());


        params.push("end_epoch_exclusive".to_string());
        params.push(self.end_epoch_exclusive.to_string());


        params.push("nonce".to_string());
        params.push(self.nonce.to_string());


        params.push("notary_public_key".to_string());
        params.push(self.notary_public_key.to_string());


        params.push("notary_as_signatory".to_string());
        params.push(self.notary_as_signatory.to_string());


        params.push("cost_unit_limit".to_string());
        params.push(self.cost_unit_limit.to_string());


        params.push("tip_percentage".to_string());
        params.push(self.tip_percentage.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionHeader value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionHeader {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub version: Vec<isize>,
            pub network_id: Vec<isize>,
            pub start_epoch_inclusive: Vec<String>,
            pub end_epoch_exclusive: Vec<String>,
            pub nonce: Vec<String>,
            pub notary_public_key: Vec<String>,
            pub notary_as_signatory: Vec<bool>,
            pub cost_unit_limit: Vec<String>,
            pub tip_percentage: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionHeader".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "version" => intermediate_rep.version.push(<isize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "network_id" => intermediate_rep.network_id.push(<isize as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "start_epoch_inclusive" => intermediate_rep.start_epoch_inclusive.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "end_epoch_exclusive" => intermediate_rep.end_epoch_exclusive.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "nonce" => intermediate_rep.nonce.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "notary_public_key" => intermediate_rep.notary_public_key.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "notary_as_signatory" => intermediate_rep.notary_as_signatory.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cost_unit_limit" => intermediate_rep.cost_unit_limit.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "tip_percentage" => intermediate_rep.tip_percentage.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionHeader".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionHeader {
            version: intermediate_rep.version.into_iter().next().ok_or("version missing in TransactionHeader".to_string())?,
            network_id: intermediate_rep.network_id.into_iter().next().ok_or("network_id missing in TransactionHeader".to_string())?,
            start_epoch_inclusive: intermediate_rep.start_epoch_inclusive.into_iter().next().ok_or("start_epoch_inclusive missing in TransactionHeader".to_string())?,
            end_epoch_exclusive: intermediate_rep.end_epoch_exclusive.into_iter().next().ok_or("end_epoch_exclusive missing in TransactionHeader".to_string())?,
            nonce: intermediate_rep.nonce.into_iter().next().ok_or("nonce missing in TransactionHeader".to_string())?,
            notary_public_key: intermediate_rep.notary_public_key.into_iter().next().ok_or("notary_public_key missing in TransactionHeader".to_string())?,
            notary_as_signatory: intermediate_rep.notary_as_signatory.into_iter().next().ok_or("notary_as_signatory missing in TransactionHeader".to_string())?,
            cost_unit_limit: intermediate_rep.cost_unit_limit.into_iter().next().ok_or("cost_unit_limit missing in TransactionHeader".to_string())?,
            tip_percentage: intermediate_rep.tip_percentage.into_iter().next().ok_or("tip_percentage missing in TransactionHeader".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionHeader> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionHeader>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionHeader>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionHeader - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionHeader> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionHeader as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionHeader - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionIntent {
    /// Transaction intent hash, hex-encoded.
    #[serde(rename = "hash")]
    pub hash: String,

    #[serde(rename = "header")]
    pub header: models::TransactionHeader,

    /// Transaction manifest, SBOR-encoded and then hex-encoded.
    #[serde(rename = "manifest")]
    pub manifest: String,

}

impl TransactionIntent {
    pub fn new(hash: String, header: models::TransactionHeader, manifest: String, ) -> TransactionIntent {
        TransactionIntent {
            hash: hash,
            header: header,
            manifest: manifest,
        }
    }
}

/// Converts the TransactionIntent value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionIntent {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("hash".to_string());
        params.push(self.hash.to_string());

        // Skipping header in query parameter serialization


        params.push("manifest".to_string());
        params.push(self.manifest.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionIntent value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionIntent {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub hash: Vec<String>,
            pub header: Vec<models::TransactionHeader>,
            pub manifest: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionIntent".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "hash" => intermediate_rep.hash.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "header" => intermediate_rep.header.push(<models::TransactionHeader as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "manifest" => intermediate_rep.manifest.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionIntent".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionIntent {
            hash: intermediate_rep.hash.into_iter().next().ok_or("hash missing in TransactionIntent".to_string())?,
            header: intermediate_rep.header.into_iter().next().ok_or("header missing in TransactionIntent".to_string())?,
            manifest: intermediate_rep.manifest.into_iter().next().ok_or("manifest missing in TransactionIntent".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionIntent> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionIntent>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionIntent>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionIntent - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionIntent> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionIntent as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionIntent - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionPreviewRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// A transaction manifest. Sbor encoded, and then hex encoded.
    #[serde(rename = "manifest")]
    pub manifest: String,

    /// Maximum number of cost units available for transaction execution. A decimal 32-bit unsigned integer.
    #[serde(rename = "cost_unit_limit")]
    pub cost_unit_limit: String,

    /// The validator tip. A decimal 32-bit unsigned integer, representing the percentage amount (a value of \"1\" corresponds to 1%).
    #[serde(rename = "tip_percentage")]
    pub tip_percentage: String,

    /// The nonce value to use for execution. A decimal 64-bit unsigned integer.
    #[serde(rename = "nonce")]
    pub nonce: String,

    /// A list of public keys to be used as transaction signers, in a compressed format, hex encoded.
    #[serde(rename = "signer_public_keys")]
    pub signer_public_keys: Vec<String>,

    #[serde(rename = "flags")]
    pub flags: models::TransactionPreviewRequestFlags,

}

impl TransactionPreviewRequest {
    pub fn new(network_identifier: models::NetworkIdentifier, manifest: String, cost_unit_limit: String, tip_percentage: String, nonce: String, signer_public_keys: Vec<String>, flags: models::TransactionPreviewRequestFlags, ) -> TransactionPreviewRequest {
        TransactionPreviewRequest {
            network_identifier: network_identifier,
            manifest: manifest,
            cost_unit_limit: cost_unit_limit,
            tip_percentage: tip_percentage,
            nonce: nonce,
            signer_public_keys: signer_public_keys,
            flags: flags,
        }
    }
}

/// Converts the TransactionPreviewRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionPreviewRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization


        params.push("manifest".to_string());
        params.push(self.manifest.to_string());


        params.push("cost_unit_limit".to_string());
        params.push(self.cost_unit_limit.to_string());


        params.push("tip_percentage".to_string());
        params.push(self.tip_percentage.to_string());


        params.push("nonce".to_string());
        params.push(self.nonce.to_string());


        params.push("signer_public_keys".to_string());
        params.push(self.signer_public_keys.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());

        // Skipping flags in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionPreviewRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionPreviewRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub manifest: Vec<String>,
            pub cost_unit_limit: Vec<String>,
            pub tip_percentage: Vec<String>,
            pub nonce: Vec<String>,
            pub signer_public_keys: Vec<Vec<String>>,
            pub flags: Vec<models::TransactionPreviewRequestFlags>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionPreviewRequest".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "manifest" => intermediate_rep.manifest.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cost_unit_limit" => intermediate_rep.cost_unit_limit.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "tip_percentage" => intermediate_rep.tip_percentage.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "nonce" => intermediate_rep.nonce.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "signer_public_keys" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionPreviewRequest".to_string()),
                    "flags" => intermediate_rep.flags.push(<models::TransactionPreviewRequestFlags as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionPreviewRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionPreviewRequest {
            network_identifier: intermediate_rep.network_identifier.into_iter().next().ok_or("network_identifier missing in TransactionPreviewRequest".to_string())?,
            manifest: intermediate_rep.manifest.into_iter().next().ok_or("manifest missing in TransactionPreviewRequest".to_string())?,
            cost_unit_limit: intermediate_rep.cost_unit_limit.into_iter().next().ok_or("cost_unit_limit missing in TransactionPreviewRequest".to_string())?,
            tip_percentage: intermediate_rep.tip_percentage.into_iter().next().ok_or("tip_percentage missing in TransactionPreviewRequest".to_string())?,
            nonce: intermediate_rep.nonce.into_iter().next().ok_or("nonce missing in TransactionPreviewRequest".to_string())?,
            signer_public_keys: intermediate_rep.signer_public_keys.into_iter().next().ok_or("signer_public_keys missing in TransactionPreviewRequest".to_string())?,
            flags: intermediate_rep.flags.into_iter().next().ok_or("flags missing in TransactionPreviewRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionPreviewRequest> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionPreviewRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionPreviewRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionPreviewRequest - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionPreviewRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionPreviewRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionPreviewRequest - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionPreviewRequestFlags {
    #[serde(rename = "unlimited_loan")]
    pub unlimited_loan: bool,

}

impl TransactionPreviewRequestFlags {
    pub fn new(unlimited_loan: bool, ) -> TransactionPreviewRequestFlags {
        TransactionPreviewRequestFlags {
            unlimited_loan: unlimited_loan,
        }
    }
}

/// Converts the TransactionPreviewRequestFlags value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionPreviewRequestFlags {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("unlimited_loan".to_string());
        params.push(self.unlimited_loan.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionPreviewRequestFlags value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionPreviewRequestFlags {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub unlimited_loan: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionPreviewRequestFlags".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "unlimited_loan" => intermediate_rep.unlimited_loan.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionPreviewRequestFlags".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionPreviewRequestFlags {
            unlimited_loan: intermediate_rep.unlimited_loan.into_iter().next().ok_or("unlimited_loan missing in TransactionPreviewRequestFlags".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionPreviewRequestFlags> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionPreviewRequestFlags>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionPreviewRequestFlags>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionPreviewRequestFlags - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionPreviewRequestFlags> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionPreviewRequestFlags as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionPreviewRequestFlags - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionPreviewResponse {
    #[serde(rename = "transaction_status")]
    pub transaction_status: models::TransactionStatus,

    #[serde(rename = "transaction_fee")]
    pub transaction_fee: models::FeeSummary,

    #[serde(rename = "logs")]
    pub logs: Vec<models::TransactionPreviewResponseLogsInner>,

    /// A list of new package addresses, hex-encoded.
    #[serde(rename = "new_package_addresses")]
    pub new_package_addresses: Vec<String>,

    /// A list of new component addresses, hex-encoded.
    #[serde(rename = "new_component_addresses")]
    pub new_component_addresses: Vec<String>,

    /// A list of new resource addresses, hex-encoded.
    #[serde(rename = "new_resource_addresses")]
    pub new_resource_addresses: Vec<String>,

    #[serde(rename = "output")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub output: Option<Vec<String>>,

    #[serde(rename = "error_message")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub error_message: Option<String>,

}

impl TransactionPreviewResponse {
    pub fn new(transaction_status: models::TransactionStatus, transaction_fee: models::FeeSummary, logs: Vec<models::TransactionPreviewResponseLogsInner>, new_package_addresses: Vec<String>, new_component_addresses: Vec<String>, new_resource_addresses: Vec<String>, ) -> TransactionPreviewResponse {
        TransactionPreviewResponse {
            transaction_status: transaction_status,
            transaction_fee: transaction_fee,
            logs: logs,
            new_package_addresses: new_package_addresses,
            new_component_addresses: new_component_addresses,
            new_resource_addresses: new_resource_addresses,
            output: None,
            error_message: None,
        }
    }
}

/// Converts the TransactionPreviewResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionPreviewResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping transaction_status in query parameter serialization

        // Skipping transaction_fee in query parameter serialization

        // Skipping logs in query parameter serialization


        params.push("new_package_addresses".to_string());
        params.push(self.new_package_addresses.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());


        params.push("new_component_addresses".to_string());
        params.push(self.new_component_addresses.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());


        params.push("new_resource_addresses".to_string());
        params.push(self.new_resource_addresses.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());


        if let Some(ref output) = self.output {
            params.push("output".to_string());
            params.push(output.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());
        }


        if let Some(ref error_message) = self.error_message {
            params.push("error_message".to_string());
            params.push(error_message.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionPreviewResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionPreviewResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub transaction_status: Vec<models::TransactionStatus>,
            pub transaction_fee: Vec<models::FeeSummary>,
            pub logs: Vec<Vec<models::TransactionPreviewResponseLogsInner>>,
            pub new_package_addresses: Vec<Vec<String>>,
            pub new_component_addresses: Vec<Vec<String>>,
            pub new_resource_addresses: Vec<Vec<String>>,
            pub output: Vec<Vec<String>>,
            pub error_message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionPreviewResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "transaction_status" => intermediate_rep.transaction_status.push(<models::TransactionStatus as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "transaction_fee" => intermediate_rep.transaction_fee.push(<models::FeeSummary as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "logs" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionPreviewResponse".to_string()),
                    "new_package_addresses" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionPreviewResponse".to_string()),
                    "new_component_addresses" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionPreviewResponse".to_string()),
                    "new_resource_addresses" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionPreviewResponse".to_string()),
                    "output" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionPreviewResponse".to_string()),
                    "error_message" => intermediate_rep.error_message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionPreviewResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionPreviewResponse {
            transaction_status: intermediate_rep.transaction_status.into_iter().next().ok_or("transaction_status missing in TransactionPreviewResponse".to_string())?,
            transaction_fee: intermediate_rep.transaction_fee.into_iter().next().ok_or("transaction_fee missing in TransactionPreviewResponse".to_string())?,
            logs: intermediate_rep.logs.into_iter().next().ok_or("logs missing in TransactionPreviewResponse".to_string())?,
            new_package_addresses: intermediate_rep.new_package_addresses.into_iter().next().ok_or("new_package_addresses missing in TransactionPreviewResponse".to_string())?,
            new_component_addresses: intermediate_rep.new_component_addresses.into_iter().next().ok_or("new_component_addresses missing in TransactionPreviewResponse".to_string())?,
            new_resource_addresses: intermediate_rep.new_resource_addresses.into_iter().next().ok_or("new_resource_addresses missing in TransactionPreviewResponse".to_string())?,
            output: intermediate_rep.output.into_iter().next(),
            error_message: intermediate_rep.error_message.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionPreviewResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionPreviewResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionPreviewResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionPreviewResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionPreviewResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionPreviewResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionPreviewResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionPreviewResponseLogsInner {
    #[serde(rename = "level")]
    pub level: String,

    #[serde(rename = "message")]
    pub message: String,

}

impl TransactionPreviewResponseLogsInner {
    pub fn new(level: String, message: String, ) -> TransactionPreviewResponseLogsInner {
        TransactionPreviewResponseLogsInner {
            level: level,
            message: message,
        }
    }
}

/// Converts the TransactionPreviewResponseLogsInner value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionPreviewResponseLogsInner {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("level".to_string());
        params.push(self.level.to_string());


        params.push("message".to_string());
        params.push(self.message.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionPreviewResponseLogsInner value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionPreviewResponseLogsInner {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub level: Vec<String>,
            pub message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionPreviewResponseLogsInner".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "level" => intermediate_rep.level.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionPreviewResponseLogsInner".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionPreviewResponseLogsInner {
            level: intermediate_rep.level.into_iter().next().ok_or("level missing in TransactionPreviewResponseLogsInner".to_string())?,
            message: intermediate_rep.message.into_iter().next().ok_or("message missing in TransactionPreviewResponseLogsInner".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionPreviewResponseLogsInner> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionPreviewResponseLogsInner>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionPreviewResponseLogsInner>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionPreviewResponseLogsInner - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionPreviewResponseLogsInner> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionPreviewResponseLogsInner as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionPreviewResponseLogsInner - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// The transaction execution receipt
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionReceipt {
    #[serde(rename = "status")]
    pub status: models::TransactionStatus,

    #[serde(rename = "fee_summary")]
    pub fee_summary: models::FeeSummary,

    /// The engine return data (only present if status is succeeded)
    #[serde(rename = "output")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub output: Option<Vec<String>>,

    /// Error message (only present if status is failed or rejected)
    #[serde(rename = "error_message")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub error_message: Option<String>,

}

impl TransactionReceipt {
    pub fn new(status: models::TransactionStatus, fee_summary: models::FeeSummary, ) -> TransactionReceipt {
        TransactionReceipt {
            status: status,
            fee_summary: fee_summary,
            output: None,
            error_message: None,
        }
    }
}

/// Converts the TransactionReceipt value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionReceipt {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping status in query parameter serialization

        // Skipping fee_summary in query parameter serialization


        if let Some(ref output) = self.output {
            params.push("output".to_string());
            params.push(output.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());
        }


        if let Some(ref error_message) = self.error_message {
            params.push("error_message".to_string());
            params.push(error_message.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionReceipt value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionReceipt {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub status: Vec<models::TransactionStatus>,
            pub fee_summary: Vec<models::FeeSummary>,
            pub output: Vec<Vec<String>>,
            pub error_message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionReceipt".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "status" => intermediate_rep.status.push(<models::TransactionStatus as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "fee_summary" => intermediate_rep.fee_summary.push(<models::FeeSummary as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "output" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionReceipt".to_string()),
                    "error_message" => intermediate_rep.error_message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionReceipt".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionReceipt {
            status: intermediate_rep.status.into_iter().next().ok_or("status missing in TransactionReceipt".to_string())?,
            fee_summary: intermediate_rep.fee_summary.into_iter().next().ok_or("fee_summary missing in TransactionReceipt".to_string())?,
            output: intermediate_rep.output.into_iter().next(),
            error_message: intermediate_rep.error_message.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionReceipt> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionReceipt>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionReceipt>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionReceipt - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionReceipt> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionReceipt as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionReceipt - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


/// The status of the transaction
/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum TransactionStatus {
    #[serde(rename = "succeeded")]
    SUCCEEDED,
    #[serde(rename = "failed")]
    FAILED,
    #[serde(rename = "rejected")]
    REJECTED,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            TransactionStatus::SUCCEEDED => write!(f, "succeeded"),
            TransactionStatus::FAILED => write!(f, "failed"),
            TransactionStatus::REJECTED => write!(f, "rejected"),
        }
    }
}

impl std::str::FromStr for TransactionStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "succeeded" => std::result::Result::Ok(TransactionStatus::SUCCEEDED),
            "failed" => std::result::Result::Ok(TransactionStatus::FAILED),
            "rejected" => std::result::Result::Ok(TransactionStatus::REJECTED),
            _ => std::result::Result::Err(format!("Value not valid: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionSubmitRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

    /// A notarized transaction encoded in the Radix transaction format, and then hex encoded.
    #[serde(rename = "notarized_transaction")]
    pub notarized_transaction: String,

}

impl TransactionSubmitRequest {
    pub fn new(network_identifier: models::NetworkIdentifier, notarized_transaction: String, ) -> TransactionSubmitRequest {
        TransactionSubmitRequest {
            network_identifier: network_identifier,
            notarized_transaction: notarized_transaction,
        }
    }
}

/// Converts the TransactionSubmitRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionSubmitRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization


        params.push("notarized_transaction".to_string());
        params.push(self.notarized_transaction.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionSubmitRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionSubmitRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
            pub notarized_transaction: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionSubmitRequest".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "notarized_transaction" => intermediate_rep.notarized_transaction.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionSubmitRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionSubmitRequest {
            network_identifier: intermediate_rep.network_identifier.into_iter().next().ok_or("network_identifier missing in TransactionSubmitRequest".to_string())?,
            notarized_transaction: intermediate_rep.notarized_transaction.into_iter().next().ok_or("notarized_transaction missing in TransactionSubmitRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionSubmitRequest> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionSubmitRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionSubmitRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionSubmitRequest - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionSubmitRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionSubmitRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionSubmitRequest - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct TransactionSubmitResponse {
    /// Is true if the transaction is a duplicate of an existing transaction in the mempool.
    #[serde(rename = "duplicate")]
    pub duplicate: bool,

}

impl TransactionSubmitResponse {
    pub fn new(duplicate: bool, ) -> TransactionSubmitResponse {
        TransactionSubmitResponse {
            duplicate: duplicate,
        }
    }
}

/// Converts the TransactionSubmitResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionSubmitResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("duplicate".to_string());
        params.push(self.duplicate.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionSubmitResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionSubmitResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub duplicate: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionSubmitResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "duplicate" => intermediate_rep.duplicate.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionSubmitResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionSubmitResponse {
            duplicate: intermediate_rep.duplicate.into_iter().next().ok_or("duplicate missing in TransactionSubmitResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionSubmitResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionSubmitResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionSubmitResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionSubmitResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionSubmitResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionSubmitResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionSubmitResponse - {}",
                                value, err))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Unable to convert header: {:?} to string: {}",
                     hdr_value, e))
        }
    }
}

