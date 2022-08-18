#![allow(unused_qualifications)]

use crate::core_api::generated::models;

use crate::core_api::generated::header;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Bech32Hrps {
    #[serde(rename = "account_hrp")]
    pub account_hrp: String,

    #[serde(rename = "validator_hrp")]
    pub validator_hrp: String,

    #[serde(rename = "node_hrp")]
    pub node_hrp: String,

    #[serde(rename = "resource_hrp_suffix")]
    pub resource_hrp_suffix: String,

}

impl Bech32Hrps {
    pub fn new(account_hrp: String, validator_hrp: String, node_hrp: String, resource_hrp_suffix: String, ) -> Bech32Hrps {
        Bech32Hrps {
            account_hrp: account_hrp,
            validator_hrp: validator_hrp,
            node_hrp: node_hrp,
            resource_hrp_suffix: resource_hrp_suffix,
        }
    }
}

/// Converts the Bech32Hrps value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Bech32Hrps {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("account_hrp".to_string());
        params.push(self.account_hrp.to_string());


        params.push("validator_hrp".to_string());
        params.push(self.validator_hrp.to_string());


        params.push("node_hrp".to_string());
        params.push(self.node_hrp.to_string());


        params.push("resource_hrp_suffix".to_string());
        params.push(self.resource_hrp_suffix.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Bech32Hrps value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Bech32Hrps {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub account_hrp: Vec<String>,
            pub validator_hrp: Vec<String>,
            pub node_hrp: Vec<String>,
            pub resource_hrp_suffix: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Bech32Hrps".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "account_hrp" => intermediate_rep.account_hrp.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "validator_hrp" => intermediate_rep.validator_hrp.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "node_hrp" => intermediate_rep.node_hrp.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "resource_hrp_suffix" => intermediate_rep.resource_hrp_suffix.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Bech32Hrps".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Bech32Hrps {
            account_hrp: intermediate_rep.account_hrp.into_iter().next().ok_or("account_hrp missing in Bech32Hrps".to_string())?,
            validator_hrp: intermediate_rep.validator_hrp.into_iter().next().ok_or("validator_hrp missing in Bech32Hrps".to_string())?,
            node_hrp: intermediate_rep.node_hrp.into_iter().next().ok_or("node_hrp missing in Bech32Hrps".to_string())?,
            resource_hrp_suffix: intermediate_rep.resource_hrp_suffix.into_iter().next().ok_or("resource_hrp_suffix missing in Bech32Hrps".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Bech32Hrps> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<Bech32Hrps>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Bech32Hrps>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for Bech32Hrps - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Bech32Hrps> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Bech32Hrps as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into Bech32Hrps - {}",
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
pub struct CoreErrorDetails {
    #[serde(rename = "type")]
    pub type_: String,

}

impl CoreErrorDetails {
    pub fn new(type_: String, ) -> CoreErrorDetails {
        CoreErrorDetails {
            type_: type_,
        }
    }
}

/// Converts the CoreErrorDetails value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CoreErrorDetails {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CoreErrorDetails value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CoreErrorDetails {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing CoreErrorDetails".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing CoreErrorDetails".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CoreErrorDetails {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in CoreErrorDetails".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CoreErrorDetails> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<CoreErrorDetails>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<CoreErrorDetails>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for CoreErrorDetails - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CoreErrorDetails> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <CoreErrorDetails as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into CoreErrorDetails - {}",
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
    /// A numeric code corresponding to the given error type, roughly aligned with HTTP Status Code semantics (eg 400/404/500).
    #[serde(rename = "code")]
    pub code: isize,

    /// A human-readable error message.
    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "details")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub details: Option<models::CoreErrorDetails>,

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
            details: None,
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

        // Skipping details in query parameter serialization


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
            pub details: Vec<models::CoreErrorDetails>,
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
                    "details" => intermediate_rep.details.push(<models::CoreErrorDetails as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
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
            details: intermediate_rep.details.into_iter().next(),
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


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FeeSummary {
    #[serde(rename = "loan_fully_repaid")]
    pub loan_fully_repaid: bool,

    #[serde(rename = "cost_unit_limit")]
    pub cost_unit_limit: String,

    #[serde(rename = "cost_units_consumed")]
    pub cost_units_consumed: String,

    #[serde(rename = "cost_unit_price")]
    pub cost_unit_price: String,

    #[serde(rename = "tip_percentage")]
    pub tip_percentage: String,

    #[serde(rename = "xrd_burned")]
    pub xrd_burned: String,

    #[serde(rename = "xrd_tipped")]
    pub xrd_tipped: String,

}

impl FeeSummary {
    pub fn new(loan_fully_repaid: bool, cost_unit_limit: String, cost_units_consumed: String, cost_unit_price: String, tip_percentage: String, xrd_burned: String, xrd_tipped: String, ) -> FeeSummary {
        FeeSummary {
            loan_fully_repaid: loan_fully_repaid,
            cost_unit_limit: cost_unit_limit,
            cost_units_consumed: cost_units_consumed,
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


        params.push("cost_units_consumed".to_string());
        params.push(self.cost_units_consumed.to_string());


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
            pub cost_units_consumed: Vec<String>,
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
                    "cost_units_consumed" => intermediate_rep.cost_units_consumed.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
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
            cost_units_consumed: intermediate_rep.cost_units_consumed.into_iter().next().ok_or("cost_units_consumed missing in FeeSummary".to_string())?,
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
pub struct InternalServerError {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "exception")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub exception: Option<String>,

    #[serde(rename = "cause")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub cause: Option<String>,

}

impl InternalServerError {
    pub fn new(type_: String, ) -> InternalServerError {
        InternalServerError {
            type_: type_,
            exception: None,
            cause: None,
        }
    }
}

/// Converts the InternalServerError value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for InternalServerError {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());


        if let Some(ref exception) = self.exception {
            params.push("exception".to_string());
            params.push(exception.to_string());
        }


        if let Some(ref cause) = self.cause {
            params.push("cause".to_string());
            params.push(cause.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InternalServerError value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InternalServerError {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
            pub exception: Vec<String>,
            pub cause: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InternalServerError".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "exception" => intermediate_rep.exception.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cause" => intermediate_rep.cause.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InternalServerError".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InternalServerError {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in InternalServerError".to_string())?,
            exception: intermediate_rep.exception.into_iter().next(),
            cause: intermediate_rep.cause.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InternalServerError> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<InternalServerError>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<InternalServerError>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InternalServerError - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<InternalServerError> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InternalServerError as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InternalServerError - {}",
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
pub struct InternalServerErrorAllOf {
    #[serde(rename = "exception")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub exception: Option<String>,

    #[serde(rename = "cause")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub cause: Option<String>,

}

impl InternalServerErrorAllOf {
    pub fn new() -> InternalServerErrorAllOf {
        InternalServerErrorAllOf {
            exception: None,
            cause: None,
        }
    }
}

/// Converts the InternalServerErrorAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for InternalServerErrorAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref exception) = self.exception {
            params.push("exception".to_string());
            params.push(exception.to_string());
        }


        if let Some(ref cause) = self.cause {
            params.push("cause".to_string());
            params.push(cause.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InternalServerErrorAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InternalServerErrorAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub exception: Vec<String>,
            pub cause: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InternalServerErrorAllOf".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "exception" => intermediate_rep.exception.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cause" => intermediate_rep.cause.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InternalServerErrorAllOf".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InternalServerErrorAllOf {
            exception: intermediate_rep.exception.into_iter().next(),
            cause: intermediate_rep.cause.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InternalServerErrorAllOf> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<InternalServerErrorAllOf>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<InternalServerErrorAllOf>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InternalServerErrorAllOf - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<InternalServerErrorAllOf> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InternalServerErrorAllOf as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InternalServerErrorAllOf - {}",
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
pub struct InvalidHexError {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "invalid_hex")]
    pub invalid_hex: String,

}

impl InvalidHexError {
    pub fn new(type_: String, invalid_hex: String, ) -> InvalidHexError {
        InvalidHexError {
            type_: type_,
            invalid_hex: invalid_hex,
        }
    }
}

/// Converts the InvalidHexError value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for InvalidHexError {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());


        params.push("invalid_hex".to_string());
        params.push(self.invalid_hex.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InvalidHexError value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InvalidHexError {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
            pub invalid_hex: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InvalidHexError".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "invalid_hex" => intermediate_rep.invalid_hex.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InvalidHexError".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InvalidHexError {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in InvalidHexError".to_string())?,
            invalid_hex: intermediate_rep.invalid_hex.into_iter().next().ok_or("invalid_hex missing in InvalidHexError".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InvalidHexError> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<InvalidHexError>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<InvalidHexError>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InvalidHexError - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<InvalidHexError> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InvalidHexError as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InvalidHexError - {}",
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
pub struct InvalidHexErrorAllOf {
    #[serde(rename = "invalid_hex")]
    pub invalid_hex: String,

}

impl InvalidHexErrorAllOf {
    pub fn new(invalid_hex: String, ) -> InvalidHexErrorAllOf {
        InvalidHexErrorAllOf {
            invalid_hex: invalid_hex,
        }
    }
}

/// Converts the InvalidHexErrorAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for InvalidHexErrorAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("invalid_hex".to_string());
        params.push(self.invalid_hex.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InvalidHexErrorAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InvalidHexErrorAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub invalid_hex: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InvalidHexErrorAllOf".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "invalid_hex" => intermediate_rep.invalid_hex.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InvalidHexErrorAllOf".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InvalidHexErrorAllOf {
            invalid_hex: intermediate_rep.invalid_hex.into_iter().next().ok_or("invalid_hex missing in InvalidHexErrorAllOf".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InvalidHexErrorAllOf> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<InvalidHexErrorAllOf>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<InvalidHexErrorAllOf>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InvalidHexErrorAllOf - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<InvalidHexErrorAllOf> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InvalidHexErrorAllOf as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InvalidHexErrorAllOf - {}",
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
pub struct InvalidJsonError {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "cause")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub cause: Option<String>,

}

impl InvalidJsonError {
    pub fn new(type_: String, ) -> InvalidJsonError {
        InvalidJsonError {
            type_: type_,
            cause: None,
        }
    }
}

/// Converts the InvalidJsonError value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for InvalidJsonError {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());


        if let Some(ref cause) = self.cause {
            params.push("cause".to_string());
            params.push(cause.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InvalidJsonError value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InvalidJsonError {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
            pub cause: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InvalidJsonError".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "cause" => intermediate_rep.cause.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InvalidJsonError".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InvalidJsonError {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in InvalidJsonError".to_string())?,
            cause: intermediate_rep.cause.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InvalidJsonError> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<InvalidJsonError>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<InvalidJsonError>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InvalidJsonError - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<InvalidJsonError> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InvalidJsonError as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InvalidJsonError - {}",
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
pub struct InvalidJsonErrorAllOf {
    #[serde(rename = "cause")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub cause: Option<String>,

}

impl InvalidJsonErrorAllOf {
    pub fn new() -> InvalidJsonErrorAllOf {
        InvalidJsonErrorAllOf {
            cause: None,
        }
    }
}

/// Converts the InvalidJsonErrorAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for InvalidJsonErrorAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref cause) = self.cause {
            params.push("cause".to_string());
            params.push(cause.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InvalidJsonErrorAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InvalidJsonErrorAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub cause: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InvalidJsonErrorAllOf".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "cause" => intermediate_rep.cause.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InvalidJsonErrorAllOf".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InvalidJsonErrorAllOf {
            cause: intermediate_rep.cause.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InvalidJsonErrorAllOf> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<InvalidJsonErrorAllOf>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<InvalidJsonErrorAllOf>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InvalidJsonErrorAllOf - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<InvalidJsonErrorAllOf> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InvalidJsonErrorAllOf as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InvalidJsonErrorAllOf - {}",
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
pub struct InvalidTransactionError {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "message")]
    pub message: String,

}

impl InvalidTransactionError {
    pub fn new(type_: String, message: String, ) -> InvalidTransactionError {
        InvalidTransactionError {
            type_: type_,
            message: message,
        }
    }
}

/// Converts the InvalidTransactionError value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for InvalidTransactionError {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());


        params.push("message".to_string());
        params.push(self.message.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InvalidTransactionError value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InvalidTransactionError {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
            pub message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InvalidTransactionError".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InvalidTransactionError".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InvalidTransactionError {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in InvalidTransactionError".to_string())?,
            message: intermediate_rep.message.into_iter().next().ok_or("message missing in InvalidTransactionError".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InvalidTransactionError> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<InvalidTransactionError>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<InvalidTransactionError>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InvalidTransactionError - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<InvalidTransactionError> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InvalidTransactionError as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InvalidTransactionError - {}",
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
pub struct InvalidTransactionErrorAllOf {
    #[serde(rename = "message")]
    pub message: String,

}

impl InvalidTransactionErrorAllOf {
    pub fn new(message: String, ) -> InvalidTransactionErrorAllOf {
        InvalidTransactionErrorAllOf {
            message: message,
        }
    }
}

/// Converts the InvalidTransactionErrorAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for InvalidTransactionErrorAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("message".to_string());
        params.push(self.message.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a InvalidTransactionErrorAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for InvalidTransactionErrorAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing InvalidTransactionErrorAllOf".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing InvalidTransactionErrorAllOf".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(InvalidTransactionErrorAllOf {
            message: intermediate_rep.message.into_iter().next().ok_or("message missing in InvalidTransactionErrorAllOf".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<InvalidTransactionErrorAllOf> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<InvalidTransactionErrorAllOf>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<InvalidTransactionErrorAllOf>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for InvalidTransactionErrorAllOf - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<InvalidTransactionErrorAllOf> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <InvalidTransactionErrorAllOf as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into InvalidTransactionErrorAllOf - {}",
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
pub struct MempoolFullError {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "mempool_transaction_count")]
    pub mempool_transaction_count: i64,

}

impl MempoolFullError {
    pub fn new(type_: String, mempool_transaction_count: i64, ) -> MempoolFullError {
        MempoolFullError {
            type_: type_,
            mempool_transaction_count: mempool_transaction_count,
        }
    }
}

/// Converts the MempoolFullError value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for MempoolFullError {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());


        params.push("mempool_transaction_count".to_string());
        params.push(self.mempool_transaction_count.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MempoolFullError value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MempoolFullError {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
            pub mempool_transaction_count: Vec<i64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing MempoolFullError".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "mempool_transaction_count" => intermediate_rep.mempool_transaction_count.push(<i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing MempoolFullError".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MempoolFullError {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in MempoolFullError".to_string())?,
            mempool_transaction_count: intermediate_rep.mempool_transaction_count.into_iter().next().ok_or("mempool_transaction_count missing in MempoolFullError".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MempoolFullError> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<MempoolFullError>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<MempoolFullError>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for MempoolFullError - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<MempoolFullError> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <MempoolFullError as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into MempoolFullError - {}",
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
pub struct MempoolFullErrorAllOf {
    #[serde(rename = "mempool_transaction_count")]
    pub mempool_transaction_count: i64,

}

impl MempoolFullErrorAllOf {
    pub fn new(mempool_transaction_count: i64, ) -> MempoolFullErrorAllOf {
        MempoolFullErrorAllOf {
            mempool_transaction_count: mempool_transaction_count,
        }
    }
}

/// Converts the MempoolFullErrorAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for MempoolFullErrorAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("mempool_transaction_count".to_string());
        params.push(self.mempool_transaction_count.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MempoolFullErrorAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MempoolFullErrorAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub mempool_transaction_count: Vec<i64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing MempoolFullErrorAllOf".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "mempool_transaction_count" => intermediate_rep.mempool_transaction_count.push(<i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing MempoolFullErrorAllOf".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MempoolFullErrorAllOf {
            mempool_transaction_count: intermediate_rep.mempool_transaction_count.into_iter().next().ok_or("mempool_transaction_count missing in MempoolFullErrorAllOf".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MempoolFullErrorAllOf> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<MempoolFullErrorAllOf>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<MempoolFullErrorAllOf>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for MempoolFullErrorAllOf - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<MempoolFullErrorAllOf> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <MempoolFullErrorAllOf as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into MempoolFullErrorAllOf - {}",
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
pub struct NetworkConfigurationRequest {
    /// Ignore.
    #[serde(rename = "openapi_gen_fix")]
    #[serde(skip_serializing_if="Option::is_none")]
    pub openapi_gen_fix: Option<String>,

}

impl NetworkConfigurationRequest {
    pub fn new() -> NetworkConfigurationRequest {
        NetworkConfigurationRequest {
            openapi_gen_fix: None,
        }
    }
}

/// Converts the NetworkConfigurationRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkConfigurationRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref openapi_gen_fix) = self.openapi_gen_fix {
            params.push("openapi_gen_fix".to_string());
            params.push(openapi_gen_fix.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkConfigurationRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkConfigurationRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub openapi_gen_fix: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkConfigurationRequest".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "openapi_gen_fix" => intermediate_rep.openapi_gen_fix.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkConfigurationRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkConfigurationRequest {
            openapi_gen_fix: intermediate_rep.openapi_gen_fix.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkConfigurationRequest> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkConfigurationRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkConfigurationRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkConfigurationRequest - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkConfigurationRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkConfigurationRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkConfigurationRequest - {}",
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

    #[serde(rename = "bech32_human_readable_parts")]
    pub bech32_human_readable_parts: models::Bech32Hrps,

}

impl NetworkConfigurationResponse {
    pub fn new(version: models::NetworkConfigurationResponseVersion, network_identifier: models::NetworkIdentifier, bech32_human_readable_parts: models::Bech32Hrps, ) -> NetworkConfigurationResponse {
        NetworkConfigurationResponse {
            version: version,
            network_identifier: network_identifier,
            bech32_human_readable_parts: bech32_human_readable_parts,
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

        // Skipping bech32_human_readable_parts in query parameter serialization

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
            pub bech32_human_readable_parts: Vec<models::Bech32Hrps>,
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
                    "bech32_human_readable_parts" => intermediate_rep.bech32_human_readable_parts.push(<models::Bech32Hrps as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
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
            bech32_human_readable_parts: intermediate_rep.bech32_human_readable_parts.into_iter().next().ok_or("bech32_human_readable_parts missing in NetworkConfigurationResponse".to_string())?,
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
pub struct NetworkNotSupportedError {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "supported_networks")]
    pub supported_networks: Vec<models::NetworkIdentifier>,

}

impl NetworkNotSupportedError {
    pub fn new(type_: String, supported_networks: Vec<models::NetworkIdentifier>, ) -> NetworkNotSupportedError {
        NetworkNotSupportedError {
            type_: type_,
            supported_networks: supported_networks,
        }
    }
}

/// Converts the NetworkNotSupportedError value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkNotSupportedError {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());

        // Skipping supported_networks in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkNotSupportedError value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkNotSupportedError {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
            pub supported_networks: Vec<Vec<models::NetworkIdentifier>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkNotSupportedError".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "supported_networks" => return std::result::Result::Err("Parsing a container in this style is not supported in NetworkNotSupportedError".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkNotSupportedError".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkNotSupportedError {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in NetworkNotSupportedError".to_string())?,
            supported_networks: intermediate_rep.supported_networks.into_iter().next().ok_or("supported_networks missing in NetworkNotSupportedError".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkNotSupportedError> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkNotSupportedError>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkNotSupportedError>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkNotSupportedError - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkNotSupportedError> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkNotSupportedError as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkNotSupportedError - {}",
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
pub struct NetworkNotSupportedErrorAllOf {
    #[serde(rename = "supported_networks")]
    pub supported_networks: Vec<models::NetworkIdentifier>,

}

impl NetworkNotSupportedErrorAllOf {
    pub fn new(supported_networks: Vec<models::NetworkIdentifier>, ) -> NetworkNotSupportedErrorAllOf {
        NetworkNotSupportedErrorAllOf {
            supported_networks: supported_networks,
        }
    }
}

/// Converts the NetworkNotSupportedErrorAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkNotSupportedErrorAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping supported_networks in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkNotSupportedErrorAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkNotSupportedErrorAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub supported_networks: Vec<Vec<models::NetworkIdentifier>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkNotSupportedErrorAllOf".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "supported_networks" => return std::result::Result::Err("Parsing a container in this style is not supported in NetworkNotSupportedErrorAllOf".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkNotSupportedErrorAllOf".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkNotSupportedErrorAllOf {
            supported_networks: intermediate_rep.supported_networks.into_iter().next().ok_or("supported_networks missing in NetworkNotSupportedErrorAllOf".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkNotSupportedErrorAllOf> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkNotSupportedErrorAllOf>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkNotSupportedErrorAllOf>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkNotSupportedErrorAllOf - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkNotSupportedErrorAllOf> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkNotSupportedErrorAllOf as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkNotSupportedErrorAllOf - {}",
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
pub struct NetworkSyncStatusRequest {
    #[serde(rename = "network_identifier")]
    pub network_identifier: models::NetworkIdentifier,

}

impl NetworkSyncStatusRequest {
    pub fn new(network_identifier: models::NetworkIdentifier, ) -> NetworkSyncStatusRequest {
        NetworkSyncStatusRequest {
            network_identifier: network_identifier,
        }
    }
}

/// Converts the NetworkSyncStatusRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkSyncStatusRequest {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping network_identifier in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkSyncStatusRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkSyncStatusRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub network_identifier: Vec<models::NetworkIdentifier>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkSyncStatusRequest".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "network_identifier" => intermediate_rep.network_identifier.push(<models::NetworkIdentifier as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkSyncStatusRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkSyncStatusRequest {
            network_identifier: intermediate_rep.network_identifier.into_iter().next().ok_or("network_identifier missing in NetworkSyncStatusRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkSyncStatusRequest> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkSyncStatusRequest>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkSyncStatusRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkSyncStatusRequest - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkSyncStatusRequest> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkSyncStatusRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkSyncStatusRequest - {}",
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
pub struct NetworkSyncStatusResponse {
    #[serde(rename = "sync_status")]
    pub sync_status: models::SyncStatus,

    /// List of peers of the node.
    #[serde(rename = "peers")]
    pub peers: Vec<models::Peer>,

}

impl NetworkSyncStatusResponse {
    pub fn new(sync_status: models::SyncStatus, peers: Vec<models::Peer>, ) -> NetworkSyncStatusResponse {
        NetworkSyncStatusResponse {
            sync_status: sync_status,
            peers: peers,
        }
    }
}

/// Converts the NetworkSyncStatusResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for NetworkSyncStatusResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping sync_status in query parameter serialization

        // Skipping peers in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a NetworkSyncStatusResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for NetworkSyncStatusResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub sync_status: Vec<models::SyncStatus>,
            pub peers: Vec<Vec<models::Peer>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing NetworkSyncStatusResponse".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "sync_status" => intermediate_rep.sync_status.push(<models::SyncStatus as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "peers" => return std::result::Result::Err("Parsing a container in this style is not supported in NetworkSyncStatusResponse".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing NetworkSyncStatusResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(NetworkSyncStatusResponse {
            sync_status: intermediate_rep.sync_status.into_iter().next().ok_or("sync_status missing in NetworkSyncStatusResponse".to_string())?,
            peers: intermediate_rep.peers.into_iter().next().ok_or("peers missing in NetworkSyncStatusResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<NetworkSyncStatusResponse> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<NetworkSyncStatusResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<NetworkSyncStatusResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for NetworkSyncStatusResponse - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<NetworkSyncStatusResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <NetworkSyncStatusResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into NetworkSyncStatusResponse - {}",
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
pub struct Peer {
    #[serde(rename = "peer_id")]
    pub peer_id: String,

}

impl Peer {
    pub fn new(peer_id: String, ) -> Peer {
        Peer {
            peer_id: peer_id,
        }
    }
}

/// Converts the Peer value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Peer {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("peer_id".to_string());
        params.push(self.peer_id.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Peer value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Peer {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub peer_id: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Peer".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "peer_id" => intermediate_rep.peer_id.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Peer".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Peer {
            peer_id: intermediate_rep.peer_id.into_iter().next().ok_or("peer_id missing in Peer".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Peer> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<Peer>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Peer>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for Peer - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Peer> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Peer as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into Peer - {}",
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
pub struct PreviewError {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "message")]
    pub message: String,

}

impl PreviewError {
    pub fn new(type_: String, message: String, ) -> PreviewError {
        PreviewError {
            type_: type_,
            message: message,
        }
    }
}

/// Converts the PreviewError value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for PreviewError {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());


        params.push("message".to_string());
        params.push(self.message.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a PreviewError value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for PreviewError {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
            pub message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing PreviewError".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing PreviewError".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(PreviewError {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in PreviewError".to_string())?,
            message: intermediate_rep.message.into_iter().next().ok_or("message missing in PreviewError".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<PreviewError> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<PreviewError>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<PreviewError>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for PreviewError - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<PreviewError> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <PreviewError as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into PreviewError - {}",
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
pub struct SyncStatus {
    #[serde(rename = "current_state_version")]
    pub current_state_version: i64,

    #[serde(rename = "target_state_version")]
    pub target_state_version: i64,

}

impl SyncStatus {
    pub fn new(current_state_version: i64, target_state_version: i64, ) -> SyncStatus {
        SyncStatus {
            current_state_version: current_state_version,
            target_state_version: target_state_version,
        }
    }
}

/// Converts the SyncStatus value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for SyncStatus {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("current_state_version".to_string());
        params.push(self.current_state_version.to_string());


        params.push("target_state_version".to_string());
        params.push(self.target_state_version.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a SyncStatus value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for SyncStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub current_state_version: Vec<i64>,
            pub target_state_version: Vec<i64>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing SyncStatus".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "current_state_version" => intermediate_rep.current_state_version.push(<i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "target_state_version" => intermediate_rep.target_state_version.push(<i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing SyncStatus".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(SyncStatus {
            current_state_version: intermediate_rep.current_state_version.into_iter().next().ok_or("current_state_version missing in SyncStatus".to_string())?,
            target_state_version: intermediate_rep.target_state_version.into_iter().next().ok_or("target_state_version missing in SyncStatus".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<SyncStatus> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<SyncStatus>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<SyncStatus>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for SyncStatus - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<SyncStatus> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <SyncStatus as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into SyncStatus - {}",
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

    /// A limit of cost units available for execution
    #[serde(rename = "cost_unit_limit")]
    pub cost_unit_limit: i32,

    /// A tip for the validator
    #[serde(rename = "tip_percentage")]
    pub tip_percentage: i32,

    /// A nonce value to use for execution
    #[serde(rename = "nonce")]
    pub nonce: i64,

    /// A list of public keys to be used as transaction signers, in a compressed format, hex encoded.
    #[serde(rename = "signer_public_keys")]
    pub signer_public_keys: Vec<String>,

    #[serde(rename = "flags")]
    pub flags: models::TransactionPreviewRequestFlags,

}

impl TransactionPreviewRequest {
    pub fn new(network_identifier: models::NetworkIdentifier, manifest: String, cost_unit_limit: i32, tip_percentage: i32, nonce: i64, signer_public_keys: Vec<String>, flags: models::TransactionPreviewRequestFlags, ) -> TransactionPreviewRequest {
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
            pub cost_unit_limit: Vec<i32>,
            pub tip_percentage: Vec<i32>,
            pub nonce: Vec<i64>,
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
                    "cost_unit_limit" => intermediate_rep.cost_unit_limit.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "tip_percentage" => intermediate_rep.tip_percentage.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "nonce" => intermediate_rep.nonce.push(<i64 as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
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
    pub logs: Vec<models::TransactionPreviewResponseLogs>,

    /// A list of new package addresses
    #[serde(rename = "new_package_addresses")]
    pub new_package_addresses: Vec<String>,

    /// A list of new component addresses
    #[serde(rename = "new_component_addresses")]
    pub new_component_addresses: Vec<String>,

    /// A list of new resource addresses
    #[serde(rename = "new_resource_addresses")]
    pub new_resource_addresses: Vec<String>,

}

impl TransactionPreviewResponse {
    pub fn new(transaction_status: models::TransactionStatus, transaction_fee: models::FeeSummary, logs: Vec<models::TransactionPreviewResponseLogs>, new_package_addresses: Vec<String>, new_component_addresses: Vec<String>, new_resource_addresses: Vec<String>, ) -> TransactionPreviewResponse {
        TransactionPreviewResponse {
            transaction_status: transaction_status,
            transaction_fee: transaction_fee,
            logs: logs,
            new_package_addresses: new_package_addresses,
            new_component_addresses: new_component_addresses,
            new_resource_addresses: new_resource_addresses,
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
            pub logs: Vec<Vec<models::TransactionPreviewResponseLogs>>,
            pub new_package_addresses: Vec<Vec<String>>,
            pub new_component_addresses: Vec<Vec<String>>,
            pub new_resource_addresses: Vec<Vec<String>>,
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
pub struct TransactionPreviewResponseLogs {
    #[serde(rename = "level")]
    pub level: String,

    #[serde(rename = "message")]
    pub message: String,

}

impl TransactionPreviewResponseLogs {
    pub fn new(level: String, message: String, ) -> TransactionPreviewResponseLogs {
        TransactionPreviewResponseLogs {
            level: level,
            message: message,
        }
    }
}

/// Converts the TransactionPreviewResponseLogs value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionPreviewResponseLogs {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("level".to_string());
        params.push(self.level.to_string());


        params.push("message".to_string());
        params.push(self.message.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionPreviewResponseLogs value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionPreviewResponseLogs {
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
                None => return std::result::Result::Err("Missing value while parsing TransactionPreviewResponseLogs".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "level" => intermediate_rep.level.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionPreviewResponseLogs".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionPreviewResponseLogs {
            level: intermediate_rep.level.into_iter().next().ok_or("level missing in TransactionPreviewResponseLogs".to_string())?,
            message: intermediate_rep.message.into_iter().next().ok_or("message missing in TransactionPreviewResponseLogs".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionPreviewResponseLogs> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionPreviewResponseLogs>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionPreviewResponseLogs>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionPreviewResponseLogs - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionPreviewResponseLogs> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionPreviewResponseLogs as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionPreviewResponseLogs - {}",
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
pub struct TransactionStatus {
    #[serde(rename = "type")]
    pub type_: String,

}

impl TransactionStatus {
    pub fn new(type_: String, ) -> TransactionStatus {
        TransactionStatus {
            type_: type_,
        }
    }
}

/// Converts the TransactionStatus value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionStatus {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionStatus value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionStatus".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionStatus".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionStatus {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in TransactionStatus".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionStatus> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionStatus>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionStatus>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionStatus - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionStatus> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionStatus as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionStatus - {}",
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
pub struct TransactionStatusFailed {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "message")]
    pub message: String,

}

impl TransactionStatusFailed {
    pub fn new(type_: String, message: String, ) -> TransactionStatusFailed {
        TransactionStatusFailed {
            type_: type_,
            message: message,
        }
    }
}

/// Converts the TransactionStatusFailed value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionStatusFailed {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());


        params.push("message".to_string());
        params.push(self.message.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionStatusFailed value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionStatusFailed {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
            pub message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionStatusFailed".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionStatusFailed".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionStatusFailed {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in TransactionStatusFailed".to_string())?,
            message: intermediate_rep.message.into_iter().next().ok_or("message missing in TransactionStatusFailed".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionStatusFailed> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionStatusFailed>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionStatusFailed>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionStatusFailed - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionStatusFailed> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionStatusFailed as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionStatusFailed - {}",
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
pub struct TransactionStatusRejected {
    #[serde(rename = "type")]
    pub type_: String,

}

impl TransactionStatusRejected {
    pub fn new(type_: String, ) -> TransactionStatusRejected {
        TransactionStatusRejected {
            type_: type_,
        }
    }
}

/// Converts the TransactionStatusRejected value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionStatusRejected {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionStatusRejected value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionStatusRejected {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionStatusRejected".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionStatusRejected".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionStatusRejected {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in TransactionStatusRejected".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionStatusRejected> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionStatusRejected>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionStatusRejected>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionStatusRejected - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionStatusRejected> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionStatusRejected as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionStatusRejected - {}",
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
pub struct TransactionStatusSucceeded {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(rename = "output")]
    pub output: Vec<String>,

}

impl TransactionStatusSucceeded {
    pub fn new(type_: String, output: Vec<String>, ) -> TransactionStatusSucceeded {
        TransactionStatusSucceeded {
            type_: type_,
            output: output,
        }
    }
}

/// Converts the TransactionStatusSucceeded value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionStatusSucceeded {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("type".to_string());
        params.push(self.type_.to_string());


        params.push("output".to_string());
        params.push(self.output.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionStatusSucceeded value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionStatusSucceeded {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub type_: Vec<String>,
            pub output: Vec<Vec<String>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionStatusSucceeded".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "type" => intermediate_rep.type_.push(<String as std::str::FromStr>::from_str(val).map_err(|x| format!("{}", x))?),
                    "output" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionStatusSucceeded".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionStatusSucceeded".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionStatusSucceeded {
            type_: intermediate_rep.type_.into_iter().next().ok_or("type missing in TransactionStatusSucceeded".to_string())?,
            output: intermediate_rep.output.into_iter().next().ok_or("output missing in TransactionStatusSucceeded".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionStatusSucceeded> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionStatusSucceeded>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionStatusSucceeded>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionStatusSucceeded - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionStatusSucceeded> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionStatusSucceeded as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionStatusSucceeded - {}",
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
pub struct TransactionStatusSucceededAllOf {
    #[serde(rename = "output")]
    pub output: Vec<String>,

}

impl TransactionStatusSucceededAllOf {
    pub fn new(output: Vec<String>, ) -> TransactionStatusSucceededAllOf {
        TransactionStatusSucceededAllOf {
            output: output,
        }
    }
}

/// Converts the TransactionStatusSucceededAllOf value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for TransactionStatusSucceededAllOf {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("output".to_string());
        params.push(self.output.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",").to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a TransactionStatusSucceededAllOf value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for TransactionStatusSucceededAllOf {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub output: Vec<Vec<String>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing TransactionStatusSucceededAllOf".to_string())
            };

            if let Some(key) = key_result {
                match key {
                    "output" => return std::result::Result::Err("Parsing a container in this style is not supported in TransactionStatusSucceededAllOf".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing TransactionStatusSucceededAllOf".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(TransactionStatusSucceededAllOf {
            output: intermediate_rep.output.into_iter().next().ok_or("output missing in TransactionStatusSucceededAllOf".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<TransactionStatusSucceededAllOf> and hyper::header::HeaderValue


impl std::convert::TryFrom<header::IntoHeaderValue<TransactionStatusSucceededAllOf>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<TransactionStatusSucceededAllOf>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(
                 format!("Invalid header value for TransactionStatusSucceededAllOf - value: {} is invalid {}",
                     hdr_value, e))
        }
    }
}


impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<TransactionStatusSucceededAllOf> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <TransactionStatusSucceededAllOf as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(
                            format!("Unable to convert header value '{}' into TransactionStatusSucceededAllOf - {}",
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

